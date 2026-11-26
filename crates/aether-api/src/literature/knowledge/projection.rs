use aether_store::{ClaimState, EvidenceRelation, Snapshot, VerificationVerdict};
use serde_json::{Value, json};

use super::*;
use crate::literature::{Citation, archive::arxiv_alias};

pub(super) fn edge(
    origin: &str,
    to: &str,
    relation: Relation,
    status: Status,
    source: Source,
    rationale: String,
) -> Edge {
    Edge {
        id: String::new(),
        origin: origin.into(),
        target: to.into(),
        origin_revision: String::new(),
        target_revision: String::new(),
        relation,
        status,
        source,
        rationale,
        confidence_bps: None,
    }
}

fn paper_identity(work: &str) -> String {
    format!("catalog:{work}")
}

fn passage_identity(work: &str, document: &str, start: usize, end: usize) -> String {
    format!("passage:{work}:{document}:{start}:{end}")
}

fn passage_label(quote: &str) -> String {
    quote
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(96)
        .collect()
}

fn resolve_citation(
    db: &Connection,
    link: &CitationLink,
    target: &str,
) -> Result<(), LiteratureError> {
    if link.origin != target {
        save_edge(
            db,
            edge(
                &link.origin,
                target,
                link.relation,
                Status::Reported,
                link.source.clone(),
                link.target_identifier.clone(),
            ),
        )?;
    }
    db.execute(
        "UPDATE knowledge_citation SET resolved = ?1 WHERE id = ?2",
        params![target, link.id],
    )?;
    Ok(())
}

fn target_paper(db: &Connection, alias: &str) -> Result<Option<String>, LiteratureError> {
    let work: Option<String> = db
        .query_row(
            "SELECT doi FROM citations WHERE doi = COALESCE(
            (SELECT work FROM citation_aliases WHERE alias = ?1), ?1)",
            [alias],
            |r| r.get(0),
        )
        .optional()?;
    let Some(work) = work else { return Ok(None) };
    let id = node_id(Kind::Paper, &paper_identity(&work));
    Ok(db
        .query_row("SELECT id FROM knowledge_node WHERE id = ?1", [&id], |r| {
            r.get(0)
        })
        .optional()?)
}

pub(in crate::literature) fn snapshot(
    db: &Connection,
    work: &str,
    sha256: &str,
    citation: &Citation,
) -> Result<String, LiteratureError> {
    let source = Source::Metadata {
        paper: work.into(),
        snapshot_sha256: sha256.into(),
    };
    let id = save_node(
        db,
        Kind::Paper,
        &paper_identity(work),
        Observation {
            label: citation.title.clone(),
            statement: citation.source_url(),
            status: Status::Reported,
            source: source.clone(),
            confidence_bps: None,
        },
    )?;
    for (relation, target) in citation
        .references
        .iter()
        .flatten()
        .map(|id| (Relation::Cites, id))
        .chain(
            citation
                .related_doi
                .iter()
                .map(|id| (Relation::RelatedPublication, id)),
        )
    {
        let alias = arxiv_alias(target);
        let mut link = CitationLink {
            id: String::new(),
            origin: id.clone(),
            target_identifier: target.clone(),
            relation,
            source: source.clone(),
        };
        link.id = digest(&link)?;
        let inserted = db.execute(
            "INSERT OR IGNORE INTO knowledge_citation(id, origin, alias, relation, record) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![link.id, link.origin, alias, relation.key(), serde_json::to_string(&link)?],
        )?;
        if inserted != 0
            && let Some(target) = target_paper(db, &alias)?
        {
            resolve_citation(db, &link, &target)?;
        }
    }
    for alias in std::iter::once(work.to_owned()).chain(citation.aliases()) {
        // Read one bounded batch at a time to avoid materializing every pending citation.
        loop {
            let links: Vec<CitationLink> = records(
                db,
                "SELECT record FROM knowledge_citation WHERE alias = ?1 AND resolved IS NULL LIMIT 128",
                [&alias],
            )?;
            if links.is_empty() {
                break;
            }
            for link in links {
                resolve_citation(db, &link, &id)?;
            }
        }
    }
    Ok(id)
}

#[derive(Deserialize)]
struct Document {
    id: String,
    source_sha256: String,
    source: DocumentSource,
    passages: Vec<Paragraph>,
}

#[derive(Deserialize)]
struct DocumentSource {
    paper_id: String,
    source_url: String,
    license: String,
}

#[derive(Deserialize)]
struct Paragraph {
    kind: String,
    start_byte: usize,
    end_byte: usize,
    quote: String,
}

pub(in crate::literature) fn document(
    db: &Connection,
    work: &str,
    record: Value,
    body: &str,
) -> Result<Vec<String>, LiteratureError> {
    let document: Document = serde_json::from_value(record)?;
    if !valid_hash(&document.id) || sha256(body.as_bytes()) != document.source_sha256 {
        return Err(LiteratureError::InvalidText);
    }
    let paper = node_id(Kind::Paper, &paper_identity(work));
    let mut ids = Vec::new();
    for passage in document.passages {
        if body.get(passage.start_byte..passage.end_byte) != Some(passage.quote.as_str()) {
            return Err(LiteratureError::InvalidText);
        }
        let kind = match passage.kind.as_str() {
            "research_question" => Kind::ResearchQuestion,
            "method" => Kind::Method,
            "claim" | "result" => Kind::Claim,
            "limitation" => Kind::Limitation,
            _ => return Err(LiteratureError::InvalidText),
        };
        let source = Source::Passage {
            paper: work.into(),
            paper_version: document.source.paper_id.clone(),
            document: document.id.clone(),
            source_sha256: document.source_sha256.clone(),
            source_url: document.source.source_url.clone(),
            license: document.source.license.clone(),
            start_byte: passage.start_byte,
            end_byte: passage.end_byte,
            section: passage.kind,
        };
        let label = passage_label(&passage.quote);
        let id = save_node(
            db,
            kind,
            &passage_identity(work, &document.id, passage.start_byte, passage.end_byte),
            Observation {
                label,
                statement: passage.quote,
                status: Status::Extracted,
                source: source.clone(),
                confidence_bps: None,
            },
        )?;
        save_edge(
            db,
            edge(
                &paper,
                &id,
                Relation::Reports,
                Status::Extracted,
                source,
                String::new(),
            ),
        )?;
        ids.push(id);
    }
    Ok(ids)
}

impl CitationCatalog {
    pub(crate) fn dataset_passage_ref(
        &self,
        work: &str,
        document: &str,
        span: std::ops::Range<usize>,
        quote: &str,
        limitation: bool,
    ) -> Result<(String, String), LiteratureError> {
        let kind = if limitation {
            Kind::Limitation
        } else {
            Kind::Claim
        };
        let id = node_id(
            kind,
            &passage_identity(work, document, span.start, span.end),
        );
        let graph = self.graph_export(&id, 0)?.graph;
        let revision = graph.root_history.iter().find(|revision| {
            let observed = &revision.observation;
            observed.statement == quote && observed.status == Status::Extracted
                && matches!(&observed.source, Source::Passage { paper, document: saved, start_byte, end_byte, .. }
                    if paper == work && saved == document && *start_byte == span.start && *end_byte == span.end)
        }).ok_or(LiteratureError::InvalidText)?;
        Ok((id, revision.sha256.clone()))
    }

    pub fn graph_report_claim(
        &mut self,
        paper: &str,
        document_id: &str,
        start: usize,
        end: usize,
    ) -> Result<Value, LiteratureError> {
        if !valid_hash(document_id) {
            return Err(LiteratureError::InvalidGraph("invalid source document ID"));
        }
        self.freeze(paper)?;
        let (requested, work) = self.resolve_work(paper)?;
        let saved = self.source_text(paper, document_id)?;
        let body = saved["text"].as_str().ok_or(LiteratureError::InvalidText)?;
        let quote = body
            .get(start..end)
            .filter(|text| !text.trim().is_empty() && text.len() <= 4096)
            .ok_or(LiteratureError::InvalidGraph(
                "expected an exact UTF-8 passage up to 4096 bytes",
            ))?;
        let document: Document = serde_json::from_value(saved["record"].clone())?;
        if requested != arxiv_alias(&requested) && requested != document.source.paper_id {
            return Err(LiteratureError::MissingCitation);
        }
        let source = Source::Passage {
            paper: work.clone(),
            paper_version: document.source.paper_id,
            document: document.id.clone(),
            source_sha256: document.source_sha256,
            source_url: document.source.source_url,
            license: document.source.license,
            start_byte: start,
            end_byte: end,
            section: "operator_selected_span".into(),
        };
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let paper = node_id(Kind::Paper, &paper_identity(&work));
        let id = save_node(
            &tx,
            Kind::Claim,
            &passage_identity(&work, &document.id, start, end),
            Observation {
                label: passage_label(quote),
                statement: quote.into(),
                status: Status::Reported,
                source: source.clone(),
                confidence_bps: None,
            },
        )?;
        save_edge(
            &tx,
            edge(
                &paper,
                &id,
                Relation::Reports,
                Status::Reported,
                source,
                String::new(),
            ),
        )?;
        enforce_graph_bounds(&tx)?;
        tx.commit()?;
        Ok(json!({"claim": id}))
    }

    pub fn graph_index_paper(&mut self, id: &str) -> Result<Value, LiteratureError> {
        let archive = self.archive_work(id)?;
        let work = &archive.id;
        let mut documents = Vec::new();
        for doc in archive.documents {
            let id = doc["id"].as_str().ok_or(LiteratureError::InvalidText)?;
            let verified = self.source_text(work, id)?;
            let body = verified["text"]
                .as_str()
                .ok_or(LiteratureError::InvalidText)?;
            documents.push((doc, body.to_owned()));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let paper = node_id(Kind::Paper, &paper_identity(work));
        if archive.snapshots.is_empty() {
            super::super::archive::save_snapshot(&tx, work, &archive.citation, None)?;
        } else {
            for observed in archive.snapshots {
                snapshot(&tx, work, &observed.sha256, &observed.citation)?;
            }
        }
        let mut passages = Vec::new();
        for (record, body) in documents {
            passages.extend(document(&tx, work, record, &body)?);
        }
        enforce_graph_bounds(&tx)?;
        tx.commit()?;
        Ok(json!({ "paper": paper, "passage_ids": passages }))
    }

    pub fn graph_index_campaign(&mut self, data: &Snapshot) -> Result<Value, LiteratureError> {
        let campaign = &data.campaign.spec;
        let source = |record: &str| Source::Campaign {
            campaign: campaign.id.clone(),
            protocol_sha256: campaign.protocol_hash.clone(),
            record: record.into(),
        };
        let key = |record: &str| format!("campaign:{}:{record}", campaign.id);
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let observation = |record: &str, label: String, statement: String, status| Observation {
            label,
            statement,
            status,
            source: source(record),
            confidence_bps: None,
        };
        let root = save_node(
            &tx,
            Kind::Campaign,
            &key(""),
            observation(
                &campaign.id,
                campaign.title.clone(),
                campaign.objective.clone(),
                Status::Observed,
            ),
        )?;
        let save = |kind, record: &str, label: String, statement: String, status| {
            let id = save_node(
                &tx,
                kind,
                &key(record),
                observation(record, label, statement, status),
            )?;
            save_edge(
                &tx,
                edge(
                    &root,
                    &id,
                    Relation::Includes,
                    Status::Observed,
                    source(record),
                    String::new(),
                ),
            )?;
            Ok::<_, LiteratureError>(id)
        };
        for claim in &data.claims {
            save(
                Kind::Claim,
                &claim.spec.id,
                claim.spec.id.clone(),
                claim.spec.statement.clone(),
                Status::Observed,
            )?;
        }
        for hypothesis in &data.hypotheses {
            save(
                Kind::Hypothesis,
                &hypothesis.id,
                hypothesis.id.clone(),
                hypothesis.statement.clone(),
                Status::Observed,
            )?;
        }
        for experiment in &data.experiments {
            let id = save(
                Kind::Experiment,
                &experiment.id,
                experiment.task_key.clone(),
                experiment.definition_hash.clone(),
                Status::Observed,
            )?;
            if let Some(hypothesis) = &experiment.hypothesis_id {
                save_edge(
                    &tx,
                    edge(
                        &id,
                        &node_id(Kind::Hypothesis, &key(hypothesis)),
                        Relation::Tests,
                        Status::Observed,
                        source(&experiment.id),
                        String::new(),
                    ),
                )?;
            }
        }
        for metric in &data.metrics {
            save(
                Kind::Metric,
                &metric.data.id,
                metric.data.name.clone(),
                serde_json::to_string(metric)?,
                Status::Observed,
            )?;
        }
        for dependency in &data.dependencies {
            save_edge(
                &tx,
                edge(
                    &node_id(Kind::Claim, &key(&dependency.claim_id)),
                    &node_id(Kind::Claim, &key(&dependency.prerequisite_id)),
                    Relation::DependsOn,
                    Status::Observed,
                    source(&dependency.claim_id),
                    String::new(),
                ),
            )?;
        }
        for evidence in &data.evidence {
            let item = &evidence.data;
            let id = save(
                Kind::Evidence,
                &item.id,
                item.id.clone(),
                serde_json::to_string(evidence)?,
                Status::Observed,
            )?;
            let relation = match item.relation {
                EvidenceRelation::Supports if !evidence.invalidated => Some(Relation::Supports),
                EvidenceRelation::Contradicts | EvidenceRelation::Invalidates => {
                    Some(Relation::Contradicts)
                }
                _ => None,
            };
            if let Some(relation) = relation {
                save_edge(
                    &tx,
                    edge(
                        &id,
                        &node_id(Kind::Claim, &key(&item.claim_id)),
                        relation,
                        Status::Observed,
                        source(&item.id),
                        String::new(),
                    ),
                )?;
            }
        }
        for receipt in &data.verifications {
            let replay = receipt.replay_attempt_id.as_ref().filter(|id| {
                data.replications
                    .iter()
                    .any(|r| r.passed && &r.replay_attempt_id == *id)
            });
            let verified = receipt.verdict == VerificationVerdict::Verified
                && receipt.rule_hash == campaign.protocol_hash
                && receipt.evaluator_hash == campaign.evaluator_hash
                && replay.is_some();
            let current_claim_state = data
                .claims
                .iter()
                .find(|claim| claim.spec.id == receipt.claim_id)
                .map(|claim| claim.state);
            let status = if verified && current_claim_state == Some(ClaimState::Verified) {
                Status::IndependentlyVerified
            } else {
                Status::Observed
            };
            let receipt_node = save(
                Kind::Evidence,
                &format!("receipt:{}", receipt.id),
                format!("Verification {}", receipt.id),
                serde_json::to_string(
                    &json!({"receipt": receipt, "claim_state_at_capture": current_claim_state}),
                )?,
                status,
            )?;
            save_edge(
                &tx,
                edge(
                    &receipt_node,
                    &node_id(Kind::Claim, &key(&receipt.claim_id)),
                    Relation::Evaluates,
                    Status::Observed,
                    source(&receipt.id),
                    String::new(),
                ),
            )?;
        }
        for contradiction in &data.contradictions {
            save_edge(
                &tx,
                edge(
                    &node_id(Kind::Evidence, &key(&contradiction.evidence_id)),
                    &node_id(Kind::Claim, &key(&contradiction.claim_id)),
                    Relation::Contradicts,
                    Status::Observed,
                    source(&contradiction.id),
                    contradiction.reason.clone(),
                ),
            )?;
        }
        enforce_graph_bounds(&tx)?;
        tx.commit()?;
        Ok(json!({"campaign": root}))
    }
}
