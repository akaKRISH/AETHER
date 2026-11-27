pub(super) mod hypotheses;
pub(super) mod projection;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::{CitationCatalog, LiteratureError};
use aether_store::artifact_sha256 as sha256;

const NODE_LIMIT: u32 = 4_096;
const EDGE_LIMIT: u32 = 16_384;
const REVISION_LIMIT: u32 = 32;
const NEIGHBOR_LIMIT: usize = 64;
const EDGE_PAGE: usize = 192;

pub(super) fn initialize(db: &Connection) -> Result<(), LiteratureError> {
    db.execute_batch(include_str!("knowledge.sql"))?;
    Ok(())
}

macro_rules! graph_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
        pub enum $name { $(#[serde(rename = $value)] $variant),+ }
        impl $name {
            fn key(self) -> &'static str {
                match self { $(Self::$variant => $value),+ }
            }
        }
    }
}

graph_enum!(Kind {
    Paper => "paper", Claim => "claim", Method => "method", Dataset => "dataset",
    Metric => "metric", Experiment => "experiment", Model => "model", Evidence => "evidence",
    Limitation => "limitation", Hypothesis => "hypothesis", ResearchQuestion => "research_question",
    Campaign => "campaign",
});
graph_enum!(Relation {
    Cites => "cites", RelatedPublication => "related_publication", Reports => "reports",
    DependsOn => "depends_on", Supports => "supports", Contradicts => "contradicts",
    UsesMethod => "uses_method", UsesDataset => "uses_dataset", UsesModel => "uses_model",
    Measures => "measures", Tests => "tests", Records => "records",
    Includes => "includes", Evaluates => "evaluates", DerivedFrom => "derived_from",
});

impl Relation {
    fn allows(self, from: Kind, to: Kind) -> bool {
        use Kind::*;
        match self {
            Self::Cites | Self::RelatedPublication => (from, to) == (Paper, Paper),
            Self::Reports => {
                from == Paper
                    && matches!(to, Claim | Method | Metric | Limitation | ResearchQuestion)
            }
            Self::DependsOn => {
                matches!(from, Claim | Hypothesis) && matches!(to, Claim | Hypothesis)
            }
            Self::Supports | Self::Contradicts => {
                matches!(from, Claim | Evidence | Metric | Limitation)
                    && matches!(to, Claim | Hypothesis)
            }
            Self::UsesMethod => matches!(from, Paper | Experiment) && to == Method,
            Self::UsesDataset => matches!(from, Paper | Experiment) && to == Dataset,
            Self::UsesModel => matches!(from, Paper | Experiment) && to == Model,
            Self::Measures => (from, to) == (Experiment, Metric),
            Self::Tests => (from, to) == (Experiment, Hypothesis),
            Self::Records => (from, to) == (Experiment, Evidence),
            Self::Evaluates => (from, to) == (Evidence, Claim),
            Self::DerivedFrom => from == Metric && matches!(to, Claim | Evidence),
            Self::Includes => {
                from == Campaign
                    && matches!(to, Claim | Hypothesis | Experiment | Metric | Evidence)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Proposed,
    Reported,
    Extracted,
    Inferred,
    Observed,
    IndependentlyVerified,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Source {
    Operator,
    Metadata {
        paper: String,
        snapshot_sha256: String,
    },
    Passage {
        paper: String,
        paper_version: String,
        document: String,
        source_sha256: String,
        source_url: String,
        license: String,
        start_byte: usize,
        end_byte: usize,
        section: String,
    },
    Campaign {
        campaign: String,
        protocol_sha256: String,
        record: String,
    },
    Archive {
        artifact: String,
        artifact_sha256: String,
        pointer: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Observation {
    label: String,
    statement: String,
    status: Status,
    source: Source,
    // Subjective input, in basis points. It confers no epistemic authority.
    confidence_bps: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Revision {
    sha256: String,
    #[serde(flatten)]
    observation: Observation,
    source_group: String,
    content_sha256: Option<String>,
}

#[derive(Debug, Serialize)]
struct Node {
    id: String,
    kind: Kind,
    identity: String,
    current: Revision,
}

#[derive(Debug, Serialize, Deserialize)]
struct Edge {
    id: String,
    origin: String,
    target: String,
    origin_revision: String,
    target_revision: String,
    relation: Relation,
    status: Status,
    source: Source,
    rationale: String,
    confidence_bps: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CitationLink {
    id: String,
    origin: String,
    target_identifier: String,
    relation: Relation,
    source: Source,
}

#[derive(Debug, Serialize)]
pub struct GraphSlice {
    root: String,
    depth: u8,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    root_history: Vec<Revision>,
    linked_history: Vec<LinkedRevision>,
    unresolved_citations: Vec<CitationLink>,
    source_group_peers: Vec<String>,
    exact_duplicate_peers: Vec<String>,
    truncated: bool,
}

#[derive(Debug, Serialize)]
struct LinkedRevision {
    node: String,
    revision: Revision,
}

#[derive(Debug, Serialize)]
pub struct GraphExport {
    schema: &'static str,
    graph_sha256: String,
    graph: GraphSlice,
}

impl GraphExport {
    pub(crate) fn digest_valid(&self) -> bool {
        digest(&self.graph).is_ok_and(|actual| actual == self.graph_sha256)
    }

    pub(crate) fn graph_sha256(&self) -> &str {
        &self.graph_sha256
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeInput {
    kind: Kind,
    key: String,
    label: String,
    #[serde(default)]
    statement: String,
    confidence_bps: Option<u16>,
}

#[derive(Deserialize)]
#[serde(tag = "by", rename_all = "snake_case", deny_unknown_fields)]
enum Endpoint {
    Local { kind: Kind, key: String },
    Id { id: String },
}

impl Endpoint {
    fn id(self) -> Result<String, LiteratureError> {
        match self {
            Self::Local { kind, key } => Ok(node_id(kind, &operator_key(&key)?)),
            Self::Id { id } if valid_hash(&id) => Ok(id),
            _ => Err(LiteratureError::InvalidGraph("invalid graph endpoint")),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EdgeInput {
    origin: Endpoint,
    target: Endpoint,
    relation: Relation,
    #[serde(default)]
    rationale: String,
    confidence_bps: Option<u16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphBatch {
    #[serde(default)]
    nodes: Vec<NodeInput>,
    #[serde(default)]
    edges: Vec<EdgeInput>,
}

fn digest(value: &impl Serialize) -> Result<String, LiteratureError> {
    Ok(sha256(&serde_json::to_vec(value)?))
}

fn node_id(kind: Kind, identity: &str) -> String {
    sha256(format!("aether.graph.node/1\n{}\n{identity}", kind.key()).as_bytes())
}

fn valid_hash(id: &str) -> bool {
    id.len() == 64
        && id
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn operator_key(key: &str) -> Result<String, LiteratureError> {
    let key = key.trim();
    if key.is_empty() || key.len() > 180 || key.chars().any(char::is_control) {
        return Err(LiteratureError::InvalidGraph("invalid local entity key"));
    }
    Ok(format!("local:{key}"))
}

fn valid_text(value: &str, max: usize) -> bool {
    value.len() <= max
        && !value
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
}

fn save_node(
    db: &Connection,
    kind: Kind,
    identity: &str,
    observation: Observation,
) -> Result<String, LiteratureError> {
    if observation.label.trim().is_empty()
        || !valid_text(&observation.label, 1024)
        || !valid_text(&observation.statement, 4096)
        || observation
            .confidence_bps
            .is_some_and(|score| score > 10_000)
    {
        return Err(LiteratureError::InvalidGraph("invalid graph observation"));
    }
    let id = node_id(kind, identity);
    db.execute(
        "INSERT OR IGNORE INTO knowledge_node VALUES (?1, ?2, ?3, ?4)",
        params![id, kind.key(), identity, observation.label],
    )?;
    let (source_group, content_sha256) = match &observation.source {
        Source::Metadata { paper, .. } => (format!("paper:{paper}"), None),
        Source::Passage { paper, .. } => (
            format!("paper:{paper}"),
            Some(sha256(observation.statement.as_bytes())),
        ),
        Source::Campaign { campaign, .. } => (format!("campaign:{campaign}"), None),
        Source::Archive {
            artifact_sha256, ..
        } => (format!("archive:{artifact_sha256}"), None),
        Source::Operator => (format!("operator:{id}"), None),
    };
    let revision = Revision {
        sha256: digest(&observation)?,
        observation,
        source_group,
        content_sha256,
    };
    let inserted = db.execute(
        "INSERT OR IGNORE INTO knowledge_revision(node, digest, record, source_group, content_sha256)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, revision.sha256, serde_json::to_string(&revision)?, revision.source_group, revision.content_sha256],
    )?;
    if inserted != 0 {
        db.execute(
            "UPDATE knowledge_node SET label = ?1 WHERE id = ?2",
            params![revision.observation.label, id],
        )?;
        let versions: u32 = db.query_row(
            "SELECT count(*) FROM knowledge_revision WHERE node = ?1",
            [&id],
            |r| r.get(0),
        )?;
        if versions > REVISION_LIMIT {
            return Err(LiteratureError::ArchiveLimit);
        }
    }
    Ok(id)
}

pub(super) fn enforce_graph_bounds(db: &Connection) -> Result<(), LiteratureError> {
    let (nodes, edges, citations): (u32, u32, u32) = db.query_row(
        "SELECT (SELECT count(*) FROM knowledge_node), (SELECT count(*) FROM knowledge_edge),
                (SELECT count(*) FROM knowledge_citation)",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    if nodes > NODE_LIMIT || edges > EDGE_LIMIT || citations > EDGE_LIMIT {
        return Err(LiteratureError::ArchiveLimit);
    }
    Ok(())
}

fn save_edge(db: &Connection, mut edge: Edge) -> Result<String, LiteratureError> {
    let origin = current_node(db, &edge.origin)?;
    let target = current_node(db, &edge.target)?;
    if edge.origin == edge.target
        || !edge.relation.allows(origin.kind, target.kind)
        || !valid_text(&edge.rationale, 1024)
        || edge.confidence_bps.is_some_and(|score| score > 10_000)
    {
        return Err(LiteratureError::InvalidGraph(
            "relationship cannot connect these endpoints",
        ));
    }
    if edge.relation == Relation::DependsOn {
        // UNION visits each prerequisite once, even when paths converge.
        let cycle: bool = db.query_row(
            "WITH RECURSIVE prerequisite(id) AS (
                SELECT ?1 UNION SELECT e.target FROM knowledge_edge e JOIN prerequisite p ON p.id = e.origin
                WHERE e.relation = 'depends_on'
             ) SELECT EXISTS(SELECT 1 FROM prerequisite WHERE id = ?2)",
            params![edge.target, edge.origin], |r| r.get(0),
        )?;
        if cycle {
            return Err(LiteratureError::InvalidGraph("claim dependency cycle"));
        }
    }
    edge.origin_revision = origin.current.sha256;
    edge.target_revision = target.current.sha256;
    edge.id = digest(&edge)?;
    db.execute(
        "INSERT OR IGNORE INTO knowledge_edge VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            edge.id,
            edge.origin,
            edge.relation.key(),
            edge.target,
            edge.origin_revision,
            edge.target_revision,
            serde_json::to_string(&edge)?
        ],
    )?;
    Ok(edge.id)
}

fn records<T: DeserializeOwned, P: rusqlite::Params>(
    db: &Connection,
    sql: &str,
    args: P,
) -> Result<Vec<T>, LiteratureError> {
    db.prepare(sql)?
        .query_map(args, |r| r.get::<_, String>(0))?
        .map(|r| Ok(serde_json::from_str(&r?)?))
        .collect()
}

fn current_node(db: &Connection, id: &str) -> Result<Node, LiteratureError> {
    let (kind, identity, record): (String, String, String) = db.query_row(
        "SELECT n.kind, n.identity, r.record FROM knowledge_node n JOIN knowledge_revision r ON r.sequence =
            (SELECT sequence FROM knowledge_revision WHERE node = n.id ORDER BY sequence DESC LIMIT 1)
         WHERE n.id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).optional()?.ok_or(LiteratureError::MissingGraphNode)?;
    Ok(Node {
        id: id.into(),
        kind: serde_json::from_value(serde_json::Value::String(kind))?,
        identity,
        current: serde_json::from_str(&record)?,
    })
}

fn ids<P: rusqlite::Params>(
    db: &Connection,
    sql: &str,
    args: P,
) -> Result<Vec<String>, LiteratureError> {
    Ok(db
        .prepare(sql)?
        .query_map(args, |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}

impl CitationCatalog {
    pub fn graph_apply(&mut self, batch: GraphBatch) -> Result<serde_json::Value, LiteratureError> {
        if batch.nodes.len() > 64 || batch.edges.len() > 128 {
            return Err(LiteratureError::InvalidGraph(
                "graph batch exceeds its limit",
            ));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        for node in batch.nodes {
            nodes.push(save_node(
                &tx,
                node.kind,
                &operator_key(&node.key)?,
                Observation {
                    label: node.label,
                    statement: node.statement,
                    status: Status::Proposed,
                    source: Source::Operator,
                    confidence_bps: node.confidence_bps,
                },
            )?);
        }
        for edge in batch.edges {
            edges.push(save_edge(
                &tx,
                Edge {
                    id: String::new(),
                    origin: edge.origin.id()?,
                    target: edge.target.id()?,
                    origin_revision: String::new(),
                    target_revision: String::new(),
                    relation: edge.relation,
                    status: Status::Inferred,
                    source: Source::Operator,
                    rationale: edge.rationale,
                    confidence_bps: edge.confidence_bps,
                },
            )?);
        }
        enforce_graph_bounds(&tx)?;
        tx.commit()?;
        Ok(serde_json::json!({"node_ids": nodes, "edge_ids": edges}))
    }

    pub fn graph_search(
        &self,
        query: &str,
        kind: Option<Kind>,
    ) -> Result<serde_json::Value, LiteratureError> {
        if !valid_text(query, 128) {
            return Err(LiteratureError::InvalidGraph("invalid graph search"));
        }
        let query = query.trim();
        let matching = if valid_hash(query) {
            ids(
                &self.db,
                "SELECT id FROM knowledge_node WHERE id = ?1 AND (?2 IS NULL OR kind = ?2)",
                params![query, kind.map(Kind::key)],
            )?
        } else {
            let prefix = format!(
                "{}%",
                query
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
            );
            match kind {
                Some(kind) => ids(
                    &self.db,
                    "SELECT id FROM knowledge_node WHERE kind = ?1 AND label LIKE ?2 ESCAPE '\\' ORDER BY label, id LIMIT 33",
                    params![kind.key(), prefix],
                )?,
                None => ids(
                    &self.db,
                    "SELECT id FROM knowledge_node WHERE label LIKE ?1 ESCAPE '\\' ORDER BY label, id LIMIT 33",
                    [prefix],
                )?,
            }
        };
        let nodes = matching
            .iter()
            .take(32)
            .map(|id| current_node(&self.db, id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(serde_json::json!({"nodes": nodes, "truncated": matching.len() > 32}))
    }

    pub fn graph_export(&self, root: &str, depth: u8) -> Result<GraphExport, LiteratureError> {
        if !valid_hash(root) || depth > 3 {
            return Err(LiteratureError::InvalidGraph(
                "expected a graph ID and depth from zero to three",
            ));
        }
        let tx = self.db.unchecked_transaction()?;
        let root_node = current_node(&tx, root)?;
        let mut visited = BTreeSet::from([root.to_owned()]);
        let mut queue = VecDeque::from([(root.to_owned(), 0)]);
        let mut edges = BTreeMap::new();
        let mut truncated = false;
        while let Some((id, distance)) = queue.pop_front() {
            if distance == depth {
                continue;
            }
            let adjacent: Vec<Edge> = records(&tx,
                "SELECT record FROM (
                    SELECT id, record FROM knowledge_edge WHERE origin = ?1 ORDER BY relation, target, id LIMIT 193
                 ) UNION ALL SELECT record FROM (
                    SELECT id, record FROM knowledge_edge WHERE target = ?1 ORDER BY relation, origin, id LIMIT 193
                 )", [&id])?;
            for edge in adjacent {
                if edges.contains_key(&edge.id) {
                    continue;
                }
                let peer = if edge.origin == id {
                    &edge.target
                } else {
                    &edge.origin
                };
                if edges.len() >= EDGE_PAGE
                    || (!visited.contains(peer) && visited.len() >= NEIGHBOR_LIMIT)
                {
                    truncated = true;
                    continue;
                }
                if visited.insert(peer.clone()) {
                    queue.push_back((peer.clone(), distance + 1));
                }
                edges.insert(edge.id.clone(), edge);
            }
        }
        let nodes = visited
            .iter()
            .map(|id| current_node(&tx, id))
            .collect::<Result<Vec<_>, _>>()?;
        let root_history: Vec<Revision> = records(
            &tx,
            "SELECT record FROM knowledge_revision WHERE node = ?1 ORDER BY sequence",
            [root],
        )?;
        let present: BTreeSet<_> = nodes
            .iter()
            .map(|node| (node.id.as_str(), node.current.sha256.as_str()))
            .chain(
                root_history
                    .iter()
                    .map(|revision| (root, revision.sha256.as_str())),
            )
            .collect();
        let historical: BTreeSet<_> = edges
            .values()
            .flat_map(|edge| {
                [
                    (edge.origin.as_str(), edge.origin_revision.as_str()),
                    (edge.target.as_str(), edge.target_revision.as_str()),
                ]
            })
            .filter(|key| !present.contains(key))
            .collect();
        let mut linked_history = Vec::new();
        for (node, digest) in historical {
            let record: String = tx.query_row(
                "SELECT record FROM knowledge_revision WHERE node = ?1 AND digest = ?2",
                params![node, digest],
                |r| r.get(0),
            )?;
            linked_history.push(LinkedRevision {
                node: node.into(),
                revision: serde_json::from_str(&record)?,
            });
        }
        let mut unresolved_citations = records(
            &tx,
            "SELECT record FROM knowledge_citation WHERE origin = ?1 AND resolved IS NULL ORDER BY id LIMIT 65",
            [root],
        )?;
        let mut source_group_peers = ids(
            &tx,
            "SELECT DISTINCT node FROM knowledge_revision WHERE source_group = ?1 AND node <> ?2 ORDER BY node LIMIT 65",
            params![root_node.current.source_group, root],
        )?;
        let mut exact_duplicate_peers = if let Some(digest) = root_node.current.content_sha256 {
            ids(
                &tx,
                "SELECT DISTINCT node FROM knowledge_revision WHERE content_sha256 = ?1 AND node <> ?2 ORDER BY node LIMIT 65",
                params![digest, root],
            )?
        } else {
            Vec::new()
        };
        truncated |= unresolved_citations.len() > 64
            || source_group_peers.len() > 64
            || exact_duplicate_peers.len() > 64;
        unresolved_citations.truncate(64);
        source_group_peers.truncate(64);
        exact_duplicate_peers.truncate(64);
        tx.commit()?;
        let graph = GraphSlice {
            root: root.into(),
            depth,
            nodes,
            edges: edges.into_values().collect(),
            root_history,
            linked_history,
            unresolved_citations,
            source_group_peers,
            exact_duplicate_peers,
            truncated,
        };
        Ok(GraphExport {
            schema: "aether.knowledge/1",
            graph_sha256: digest(&graph)?,
            graph,
        })
    }
}
