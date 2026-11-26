mod cases;

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use aether_runner::sha256;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use self::cases::{Checked, Recipe};
use super::Registry;
use crate::{
    Result,
    lab::{Lab, read_json_file, valid_id},
};

const SCHEMA: &str = "aether.development-dataset/1";
const MAX_CANDIDATES: usize = 128;
const DEFAULT_SEED: u32 = 602;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    name: String,
    seed: u32,
    candidates: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    recipe: Recipe,
    #[serde(default)]
    variant: u8,
    submission: Option<Submission>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Submission {
    answer: Value,
    #[serde(default)]
    human_authored_attested: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Rejection {
    MalformedSample,
    InvalidCase,
    MalformedAnswer,
    IncorrectAnswer,
    Provenance,
    ExactDuplicate,
    NearDuplicate,
    ConflictingAnswer,
}

#[derive(Debug, Serialize)]
struct Rejected {
    candidate: usize,
    reason: Rejection,
}

#[derive(Serialize)]
struct Instruction<'a> {
    id: String,
    group: &'a str,
    input: &'a str,
    output: &'a str,
}

fn normalized(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn shingles(input: &str) -> BTreeSet<[&str; 3]> {
    let tokens = input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    tokens.windows(3).map(|w| [w[0], w[1], w[2]]).collect()
}

fn near(left: &BTreeSet<[&str; 3]>, right: &BTreeSet<[&str; 3]>) -> bool {
    let shared = left.intersection(right).count();
    let union = left.len() + right.len() - shared;
    shared >= 3 && shared * 5 >= union * 4
}

fn find(parent: &mut [usize], mut index: usize) -> usize {
    while index != parent[index] {
        parent[index] = parent[parent[index]];
        index = parent[index];
    }
    index
}

fn same_source(a: &Checked, b: &Checked) -> bool {
    a.evidence
        .as_ref()
        .zip(b.evidence.as_ref())
        .is_some_and(|(a, b)| {
            a.paper == b.paper || a.document == b.document || a.source_sha256 == b.source_sha256
        })
}

fn components(
    rows: &[(usize, Checked)],
    rejected: &mut Vec<Rejected>,
) -> BTreeMap<String, Vec<usize>> {
    let labels = rows
        .iter()
        .map(|(_, row)| normalized(&row.input))
        .collect::<Vec<_>>();
    let grams = labels
        .iter()
        .map(|label| shingles(label))
        .collect::<Vec<_>>();
    let mut parent = (0..rows.len()).collect::<Vec<_>>();
    let mut flags = vec![None; rows.len()];
    // ponytail: compare at most 128 candidates. Index shingles if that cap grows.
    for (i, (_, current)) in rows.iter().enumerate() {
        for (j, (_, other)) in rows[..i].iter().enumerate() {
            let exact = labels[i] == labels[j];
            let similar = near(&grams[i], &grams[j]);
            if exact
                || similar
                || current.family == other.family
                || current.case == other.case
                || same_source(current, other)
            {
                let left = find(&mut parent, i);
                let right = find(&mut parent, j);
                parent[left.max(right)] = left.min(right);
            }
            if exact && current.output != other.output {
                flags[i] = Some(Rejection::ConflictingAnswer);
                flags[j] = Some(Rejection::ConflictingAnswer);
            } else if exact && !matches!(flags[i], Some(Rejection::ConflictingAnswer)) {
                flags[i] = Some(Rejection::ExactDuplicate);
            } else if similar
                && current.case == other.case
                && current.output == other.output
                && flags[i].is_none()
            {
                flags[i] = Some(Rejection::NearDuplicate);
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, (index, _)) in rows.iter().enumerate() {
        if let Some(reason) = flags[i] {
            rejected.push(Rejected {
                candidate: *index,
                reason,
            });
        } else {
            groups.entry(find(&mut parent, i)).or_default().push(i);
        }
    }
    groups
        .into_values()
        .map(|indices| {
            let mut members = BTreeSet::new();
            for &i in &indices {
                let row = &rows[i].1;
                members.insert(&row.family);
                members.insert(&row.case);
            }
            let group = sha256(
                serde_json::to_vec(&members)
                    .expect("string keys serialize")
                    .as_slice(),
            );
            (group, indices)
        })
        .collect()
}

fn validation_groups(groups: &BTreeMap<String, Vec<usize>>, seed: u32) -> BTreeSet<&str> {
    if groups.len() < 2 {
        return BTreeSet::new();
    }
    let target = groups.values().map(Vec::len).sum::<usize>().div_ceil(4);
    let mut order = groups
        .iter()
        .map(|(group, rows)| {
            (
                sha256(format!("aether.development-split/1/{seed}/{group}").as_bytes()),
                group.as_str(),
                rows.len(),
            )
        })
        .collect::<Vec<_>>();
    order.sort_unstable();
    let mut count = 0;
    order
        .iter()
        .take(order.len() - 1)
        .take_while(|(_, _, size)| {
            let include = count < target;
            count += size;
            include
        })
        .map(|(_, group, _)| *group)
        .collect()
}

#[derive(Default, Serialize)]
struct Coverage {
    rows: usize,
    groups: usize,
    topics: BTreeMap<&'static str, usize>,
    origins: BTreeMap<&'static str, usize>,
    verification: BTreeMap<&'static str, usize>,
}

impl Coverage {
    fn count(&mut self, row: &Checked) {
        self.rows += 1;
        *self.topics.entry(row.topic).or_default() += 1;
        *self.origins.entry(row.origin).or_default() += 1;
        *self.verification.entry(row.verification).or_default() += 1;
    }
}

fn line(bytes: &mut Vec<u8>, row: &impl Serialize) -> Result<()> {
    serde_json::to_writer(&mut *bytes, row)?;
    bytes.push(b'\n');
    Ok(())
}

fn part(lab: &Lab, bytes: &[u8], records: usize, format: &str) -> Result<Value> {
    Ok(
        json!({"sha256":lab.record_development_bytes(bytes)?,"bytes":bytes.len(),"records":records,"format":format}),
    )
}

pub(crate) fn build(lab: &Lab) -> Result<Value> {
    qualify(
        lab,
        Request {
            name: "apollo-development".into(),
            seed: DEFAULT_SEED,
            candidates: cases::default_candidates(),
        },
    )
}

pub(crate) fn qualify_file(lab: &Lab, path: &Path) -> Result<Value> {
    qualify(lab, read_json_file(path)?)
}

pub(crate) fn qualify(lab: &Lab, request: Request) -> Result<Value> {
    if !valid_id(&request.name) || !(1..=MAX_CANDIDATES).contains(&request.candidates.len()) {
        return Err("expected a dataset name and 1–128 candidate records".into());
    }
    let total = request.candidates.len();
    let mut rows = Vec::new();
    let mut rejected = Vec::new();
    let mut sources = None;
    for (index, value) in request.candidates.into_iter().enumerate() {
        let Ok(candidate) = serde_json::from_value::<Candidate>(value) else {
            rejected.push(Rejected {
                candidate: index,
                reason: Rejection::MalformedSample,
            });
            continue;
        };
        if candidate.recipe.selects_reviewed_source() && sources.is_none() {
            sources = Some(cases::reviewed_sources(lab)?);
        }
        match cases::check(candidate, request.seed, sources.as_deref().unwrap_or(&[]))? {
            Ok(row) => rows.push((index, row)),
            Err(reason) => rejected.push(Rejected {
                candidate: index,
                reason,
            }),
        }
    }
    let groups = components(&rows, &mut rejected);
    rejected.sort_by_key(|entry| entry.candidate);
    let validation = validation_groups(&groups, request.seed);
    let qualified = !validation.is_empty();
    let mut train = Vec::new();
    let mut dev = Vec::new();
    let mut annotations = Vec::new();
    let mut licensed_sources = BTreeMap::new();
    let mut train_coverage = Coverage::default();
    let mut val_coverage = Coverage::default();
    if qualified {
        for (group, members) in &groups {
            let (split, bytes, coverage) = if validation.contains(group.as_str()) {
                ("validation", &mut dev, &mut val_coverage)
            } else {
                ("train", &mut train, &mut train_coverage)
            };
            coverage.groups += 1;
            for &index in members {
                let row = &rows[index].1;
                let id = sha256(normalized(&row.input).as_bytes());
                line(
                    bytes,
                    &Instruction {
                        id: id.clone(),
                        group,
                        input: &row.input,
                        output: &row.output,
                    },
                )?;
                line(
                    &mut annotations,
                    &json!({
                        "id":id,"split":split,"group":group,"family":row.family,"related_case":row.case,
                        "topic":row.topic,"answer_origin":row.origin,"answer_verification":row.verification,
                        "evidence":row.evidence
                    }),
                )?;
                coverage.count(row);
                if let Some(evidence) = &row.evidence {
                    licensed_sources.insert(&evidence.document, evidence);
                }
            }
        }
    }
    let rejection_bytes =
        serde_json::to_vec(&json!({"schema":"aether.dataset-rejections/1","rejections":rejected}))?;
    let mut parts = BTreeMap::from([(
        "rejections",
        part(
            lab,
            &rejection_bytes,
            rejected.len(),
            "aether.dataset-rejections/1",
        )?,
    )]);
    if qualified {
        super::training::validate_development_exports(&train, &dev)?;
        parts.insert(
            "train",
            part(
                lab,
                &train,
                train_coverage.rows,
                "aether.instruction-jsonl/1",
            )?,
        );
        parts.insert(
            "validation",
            part(lab, &dev, val_coverage.rows, "aether.instruction-jsonl/1")?,
        );
        parts.insert(
            "annotations",
            part(
                lab,
                &annotations,
                train_coverage.rows + val_coverage.rows,
                "aether.instruction-annotations-jsonl/1",
            )?,
        );
    }
    let manifest = json!({
        "schema":SCHEMA,"name":request.name,"seed":request.seed,"scope":"development",
        "qualification":if qualified {"development_only"} else {"insufficient_disjoint_groups"},
        "independent_test":"external_custody_required","candidates":total,"retained_after_filtering":total - rejected.len(),
        "rejected":rejected.len(),"coverage":{"train":train_coverage,"validation":val_coverage},
        "source_documents":licensed_sources.into_values().collect::<Vec<_>>(),"parts":parts,
        "group_policy":"Connected components of task families, related cases, source identities and exact or near prompts. Lexical 80% trigram Jaccard.",
        "generator":"aether.development-recipes/1",
        "evaluator_sha256":sha256(concat!(include_str!("datasets/cases.rs"),
            include_str!("../../../aether-runner/src/lib.rs"),
            include_str!("../../../aether-runner/src/optimization.rs")).replace("\r\n","\n").as_bytes())
    });
    let digest = lab.record_development_artifact(&manifest)?;
    let output = json!({"manifest_sha256":digest,"manifest":manifest});
    let registry = lab.model_registry()?;
    Registry::save(&registry.0, "dataset", &request.name, &digest, &output)?;
    Ok(output)
}

pub(crate) fn list(lab: &Lab) -> Result<Value> {
    let registry = lab.model_registry()?;
    let mut query = registry.0.prepare(
        "SELECT record FROM model_registry WHERE kind='dataset' ORDER BY rowid DESC LIMIT 33",
    )?;
    let mut datasets = query
        .query_map([], |r| r.get::<_, String>(0))?
        .map(|row| Ok(serde_json::from_str::<Value>(&row?)?))
        .collect::<Result<Vec<_>>>()?;
    let truncated = datasets.len() > 32;
    datasets.truncate(32);
    Ok(json!({"schema":"aether.dataset-registry/1","datasets":datasets,"truncated":truncated}))
}

pub(crate) fn inspect(lab: &Lab, digest: &str) -> Result<Value> {
    let bytes = lab.artifact(digest)?;
    let manifest: Value = serde_json::from_slice(&bytes)?;
    if manifest["schema"] != SCHEMA || sha256(&bytes) != digest {
        return Err("not an intact development dataset manifest".into());
    }
    Ok(json!({"manifest_sha256":digest,"manifest":manifest}))
}

pub(crate) fn export(lab: &Lab, digest: &str, part: &str) -> Result<Vec<u8>> {
    let inspected = inspect(lab, digest)?;
    let descriptor = &inspected["manifest"]["parts"][part];
    let pin = descriptor["sha256"]
        .as_str()
        .ok_or("dataset part is unavailable")?;
    let bytes = lab.artifact(pin)?;
    if sha256(&bytes) != pin || descriptor["bytes"].as_u64() != Some(bytes.len() as u64) {
        return Err("dataset part failed integrity validation".into());
    }
    Ok(bytes)
}
