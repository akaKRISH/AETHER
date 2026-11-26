use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use aether_runner::sha256;
use serde::{Deserialize, Serialize};

use crate::Result;

const MAX_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ROWS: usize = 4096;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DatasetInput {
    train: SplitInput,
    validation: SplitInput,
    independent_test: SplitInput,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SplitInput {
    path: PathBuf,
    sha256: String,
    provenance: String,
    curator: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Split {
    pub sha256: String,
    pub bytes: u64,
    pub rows: u64,
    provenance: String,
    curator: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Dataset {
    pub train: Split,
    pub validation: Split,
    pub independent_test: Split,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Example {
    id: String,
    group: String,
    input: String,
    output: String,
}

#[derive(Default)]
struct SplitCheck<'a> {
    prompts: BTreeSet<String>,
    ids: BTreeSet<String>,
    groups: BTreeMap<String, &'a str>,
}

pub(super) fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn normalized(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn validate_split<'a>(
    bytes: &[u8],
    role: &'a str,
    expected: &str,
    provenance: String,
    curator: String,
    check: &mut SplitCheck<'a>,
) -> Result<Split> {
    if !valid_text(&curator, 120) || !valid_text(&provenance, 300) || bytes.len() as u64 > MAX_BYTES
    {
        return Err("invalid split provenance or size".into());
    }
    let digest = sha256(bytes);
    if digest != expected {
        return Err("dataset split SHA-256 mismatch".into());
    }
    let mut rows = 0;
    for line in std::str::from_utf8(bytes)?.lines() {
        if rows == MAX_ROWS || line.len() > 8192 {
            return Err("dataset row limit exceeded".into());
        }
        let row: Example = serde_json::from_str(line)?;
        if !valid_text(&row.id, 128)
            || !valid_text(&row.group, 128)
            || row.input.trim().is_empty()
            || row.input.len() > 4096
            || row.output.trim().is_empty()
            || row.output.len() > 4096
        {
            return Err("invalid dataset example".into());
        }
        if !check.ids.insert(normalized(&row.id))
            || !check
                .prompts
                .insert(sha256(normalized(&row.input).as_bytes()))
        {
            return Err("dataset contamination: repeated example ID or normalized input".into());
        }
        if check
            .groups
            .insert(normalized(&row.group), role)
            .is_some_and(|old| old != role)
        {
            return Err("dataset contamination: source group crosses splits".into());
        }
        rows += 1;
    }
    if rows == 0 {
        return Err("dataset split is empty".into());
    }
    Ok(Split {
        sha256: digest,
        bytes: bytes.len() as u64,
        rows: rows as u64,
        provenance,
        curator,
    })
}

impl Dataset {
    pub(super) fn read(base: &Path, request: DatasetInput) -> Result<Self> {
        let mut check = SplitCheck::default();
        let mut split = |role, entry: SplitInput| {
            let path = base.join(entry.path);
            if !fs::symlink_metadata(&path)?.is_file() {
                return Err("dataset input must be a regular file".into());
            }
            let mut bytes = Vec::new();
            File::open(path)?
                .take(MAX_BYTES + 1)
                .read_to_end(&mut bytes)?;
            validate_split(
                &bytes,
                role,
                &entry.sha256,
                entry.provenance,
                entry.curator,
                &mut check,
            )
        };
        let dataset = Self {
            train: split("train", request.train)?,
            validation: split("validation", request.validation)?,
            independent_test: split("independent_test", request.independent_test)?,
        };
        let curator = normalized(&dataset.independent_test.curator);
        if curator == normalized(&dataset.train.curator)
            || curator == normalized(&dataset.validation.curator)
        {
            return Err("independent test requires a distinct declared curator".into());
        }
        Ok(dataset)
    }

    pub(super) fn fixture() -> Result<Self> {
        let mut checked = SplitCheck::default();
        let mut make = |role, curator, lines: &str| {
            validate_split(
                lines.as_bytes(),
                role,
                &sha256(lines.as_bytes()),
                "AETHER scripted arithmetic fixture v1".into(),
                curator,
                &mut checked,
            )
        };
        Ok(Self {
            train: make(
                "train",
                "fixture-training".into(),
                concat!(
                    "{\"id\":\"t1\",\"group\":\"sums\",\"input\":\"Compute 2 + 4\",\"output\":\"6\"}\n",
                    "{\"id\":\"t2\",\"group\":\"sums\",\"input\":\"Compute 3 + 9\",\"output\":\"12\"}\n",
                    "{\"id\":\"t3\",\"group\":\"sums\",\"input\":\"Compute 4 + 8\",\"output\":\"12\"}\n",
                    "{\"id\":\"t4\",\"group\":\"sums\",\"input\":\"Compute 6 + 4\",\"output\":\"10\"}\n",
                ),
            )?,
            validation: make(
                "validation",
                "fixture-training".into(),
                r#"{"id":"v1","group":"products","input":"Compute 3 times 6","output":"18"}"#,
            )?,
            independent_test: make(
                "independent_test",
                "fixture-test-stand-in".into(),
                r#"{"id":"e1","group":"quotients","input":"Compute 81 divided by 9","output":"9"}"#,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_normalized_overlap_group_leakage_and_mutation() {
        let mut check = SplitCheck::default();
        let train = br#"{"id":"a","group":"A","input":"Sum 1 and 2","output":"3"}"#;
        let test = |bytes: &[u8], role, pin: &str, check: &mut SplitCheck<'_>| {
            validate_split(
                bytes,
                role,
                pin,
                "fixture-source".into(),
                "fixture-curator".into(),
                check,
            )
        };
        test(train, "train", &sha256(train), &mut check).unwrap();
        for (row, reason) in [
            (
                br#"{"id":"b","group":"B","input":"  SUM  1 and 2","output":"different"}"#
                    .as_slice(),
                "normalized input",
            ),
            (
                br#"{"id":"c","group":" a ","input":"A new prompt","output":"different"}"#
                    .as_slice(),
                "source group",
            ),
        ] {
            assert!(
                test(row, "independent_test", &sha256(row), &mut check)
                    .unwrap_err()
                    .to_string()
                    .contains(reason)
            );
        }
        assert!(
            test(train, "validation", &"0".repeat(64), &mut check)
                .unwrap_err()
                .to_string()
                .contains("SHA-256")
        );
        assert!(test(b"", "validation", &sha256(b""), &mut check).is_err());
    }
}
