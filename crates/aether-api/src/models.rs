pub(crate) mod datasets;
mod evaluation;
mod metadata;
pub(crate) mod training;

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    Result,
    lab::{Lab, valid_digest},
};

const MIB: u64 = 1024 * 1024;
const MAX_FILE_BYTES: u64 = 64 * 1024 * MIB;
const MAX_MANIFESTS: usize = 64;
const MAX_SMALL_FILE: u64 = 64 * 1024;

pub(crate) use evaluation::{compare, evaluate};

#[derive(Deserialize)]
struct Manifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    config: Descriptor,
    layers: Vec<Descriptor>,
}

#[derive(Deserialize)]
struct Descriptor {
    #[serde(rename = "mediaType")]
    media_type: String,
    digest: String,
    size: u64,
}

#[derive(Serialize)]
struct LocalModel {
    alias: String,
    provenance: String,
    manifest_sha256: String,
    weight_sha256: Option<String>,
    license_sha256: Vec<String>,
    checked_config: Option<Value>,
    runtime_layer_bytes: u64,
    inspection: Option<metadata::Inspection>,
    status: &'static str,
    inspection_issue: Option<String>,
}

impl Descriptor {
    fn hash(&self) -> Result<&str> {
        self.digest
            .strip_prefix("sha256:")
            .filter(|hash| valid_digest(hash))
            .ok_or_else(|| "invalid Ollama descriptor digest".into())
    }

    fn path(&self, root: &Path) -> Result<PathBuf> {
        Ok(root.join("blobs").join(format!("sha256-{}", self.hash()?)))
    }
}

fn bounded_read(path: &Path) -> Result<Vec<u8>> {
    if !fs::symlink_metadata(path)?.is_file() {
        return Err("expected a regular metadata file".into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_SMALL_FILE + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_SMALL_FILE {
        return Err("metadata file exceeds 64 KiB".into());
    }
    Ok(bytes)
}

fn read_manifest(path: &Path) -> Result<(String, Manifest)> {
    let bytes = bounded_read(path)?;
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    if manifest.schema_version != 2 || manifest.layers.len() > 32 {
        return Err("unsupported Ollama manifest".into());
    }
    for d in std::iter::once(&manifest.config).chain(&manifest.layers) {
        d.hash()?;
        if d.size > MAX_FILE_BYTES {
            return Err("Ollama blob exceeds the model file limit".into());
        }
    }
    Ok((aether_runner::sha256(&bytes), manifest))
}

fn open_blob(path: &Path) -> Result<(File, u64)> {
    if !fs::symlink_metadata(path)?.is_file() {
        return Err("expected a regular model blob".into());
    }
    let file = File::open(path)?;
    let bytes = file.metadata()?.len();
    if !(8..=MAX_FILE_BYTES).contains(&bytes) {
        return Err("model file size falls outside the inspection limit".into());
    }
    Ok((file, bytes))
}

fn inspect_file(path: &Path) -> Result<metadata::Inspection> {
    let (mut file, bytes) = open_blob(path)?;
    metadata::inspect(&mut file, bytes)
}

fn full_digest(file: &mut File, bytes: u64) -> Result<String> {
    let mut digest = Sha256::new();
    file.seek(SeekFrom::Start(0))?;
    let mut reader = file.take(bytes + 1);
    let mut buffer = vec![0; MIB as usize];
    let mut read = 0;
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        read += count as u64;
        digest.update(&buffer[..count]);
    }
    if read != bytes {
        return Err("model file length changed during inspection".into());
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn inspect_model(
    root: &Path,
    path: &Path,
    alias: String,
    provenance: String,
) -> Result<LocalModel> {
    let (manifest_sha256, manifest) = read_manifest(path)?;
    let mut model = LocalModel {
        alias,
        provenance,
        manifest_sha256,
        weight_sha256: None,
        runtime_layer_bytes: 0,
        inspection: None,
        license_sha256: Vec::new(),
        checked_config: None,
        status: "unavailable",
        inspection_issue: None,
    };
    if model.alias.ends_with(":cloud")
        || manifest
            .layers
            .iter()
            .any(|d| d.media_type.contains("remote"))
    {
        model.status = "remote_excluded";
        return Ok(model);
    }
    let weights: Vec<_> = manifest
        .layers
        .iter()
        .filter(|d| d.media_type == "application/vnd.ollama.image.model")
        .collect();
    if let [weight] = weights.as_slice() {
        if let Ok(bytes) = bounded_read(&manifest.config.path(root)?)
            && bytes.len() as u64 == manifest.config.size
            && aether_runner::sha256(&bytes) == manifest.config.hash()?
            && let Ok(config) = serde_json::from_slice::<Value>(&bytes)
        {
            model.checked_config = Some(json!({
                "sha256":manifest.config.hash()?, "format":config["model_format"].as_str(),
                "architecture":config["model_family"].as_str(), "parameter_label":config["model_type"].as_str(),
                "quantization":config["file_type"].as_str(),
            }));
        }
        model.weight_sha256 = Some(weight.hash()?.into());
        model.runtime_layer_bytes = manifest.layers.iter().try_fold(0u64, |total, d| {
            total
                .checked_add(d.size)
                .ok_or("Ollama layer sizes overflow")
        })?;
        for license in manifest
            .layers
            .iter()
            .filter(|d| d.media_type == "application/vnd.ollama.image.license")
        {
            let hash = license.hash()?;
            if let Ok(bytes) = bounded_read(&license.path(root)?)
                && bytes.len() as u64 == license.size
                && aether_runner::sha256(&bytes) == hash
            {
                model.license_sha256.push(hash.into());
            }
        }
        let weight_path = weight.path(root)?;
        let inspection = inspect_file(&weight_path).and_then(|data| {
            if fs::metadata(&weight_path)?.len() != weight.size {
                return Err("weight size disagrees with its Ollama manifest".into());
            }
            Ok(data)
        });
        match inspection {
            Ok(data) => {
                model.inspection = Some(data);
                model.status = "metadata_inspected";
            }
            Err(error) => model.inspection_issue = Some(error.to_string()),
        }
    } else {
        model.inspection_issue = Some("expected one primary weight layer".into());
    }
    Ok(model)
}

fn manifests(root: &Path, depth: usize, remaining: &mut usize) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if !fs::symlink_metadata(root)?.is_dir() {
        return Err("expected a local Ollama manifest directory".into());
    }
    for entry in fs::read_dir(root)? {
        *remaining = remaining
            .checked_sub(1)
            .ok_or("Ollama discovery entry limit exceeded")?;
        let entry = entry?;
        let kind = entry.file_type()?;
        if depth == 0 && kind.is_file() {
            paths.push(entry.path());
        } else if depth > 0 && kind.is_dir() {
            paths.extend(manifests(&entry.path(), depth - 1, remaining)?);
        }
        if paths.len() > MAX_MANIFESTS {
            return Err("too many Ollama manifests for one discovery pass".into());
        }
    }
    paths.sort_unstable();
    Ok(paths)
}

pub(crate) struct Registry(Connection);

impl Registry {
    pub(crate) fn open(path: &Path) -> rusqlite::Result<Self> {
        let db = Connection::open(path)?;
        db.busy_timeout(std::time::Duration::from_secs(3))?;
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS model_registry (
                 kind TEXT NOT NULL, key TEXT NOT NULL, sha256 TEXT NOT NULL, record TEXT NOT NULL,
                 PRIMARY KEY (kind, key, sha256)
             ) STRICT;",
        )?;
        Ok(Self(db))
    }

    fn save(
        db: &Connection,
        kind: &str,
        key: &str,
        sha: &str,
        record: &impl Serialize,
    ) -> Result<()> {
        db.execute(
            "INSERT INTO model_registry VALUES (?1,?2,?3,?4)
                    ON CONFLICT(kind,key,sha256) DO UPDATE SET record=excluded.record",
            params![kind, key, sha, serde_json::to_string(record)?],
        )?;
        Ok(())
    }

    pub(crate) fn discover(&mut self, root: &Path) -> Result<Value> {
        let base = root.join("manifests");
        let mut models = Vec::new();
        let mut issues = Vec::new();
        for path in manifests(&base, 3, &mut 256)? {
            let relative = path.strip_prefix(&base)?;
            let parts: Vec<_> = relative
                .components()
                .filter_map(|part| part.as_os_str().to_str())
                .collect();
            let [registry, namespace, name, tag] = parts.as_slice() else {
                continue;
            };
            let provenance = format!("{registry}/{namespace}/{name}:{tag}");
            if provenance.len() > 240 || !provenance.is_ascii() {
                continue;
            }
            match inspect_model(root, &path, format!("{name}:{tag}"), provenance.clone()) {
                Ok(model) => models.push(model),
                Err(error) => {
                    issues.push(json!({"provenance":provenance,"issue":error.to_string()}))
                }
            }
        }
        let tx = self.0.transaction()?;
        for model in &models {
            Self::save(
                &tx,
                "ollama",
                &model.provenance,
                &model.manifest_sha256,
                model,
            )?;
        }
        tx.commit()?;
        Ok(
            json!({"models":models,"issues":issues,"weight_integrity":"unverified_until_full_inspection"}),
        )
    }

    pub(crate) fn inspect(&self, path: &Path, expected: Option<&str>) -> Result<Value> {
        if expected.is_some_and(|digest| !valid_digest(digest)) {
            return Err("expected a lowercase SHA-256 digest".into());
        }
        let (mut file, bytes) = open_blob(path)?;
        let before = file.metadata()?.modified()?;
        let inspection = metadata::inspect(&mut file, bytes)?;
        let sha256 = full_digest(&mut file, bytes)?;
        if before != file.metadata()?.modified()? || expected.is_some_and(|pin| pin != sha256) {
            return Err("model content failed its integrity check".into());
        }
        let record = json!({
            "identity_sha256":sha256,"basis":"local_file","file_name":path.file_name().and_then(|name| name.to_str()),
            "minimum_weight_bytes":bytes, "integrity": if expected.is_some() { "expected_digest_matched" } else { "content_hashed" },
            "inspection":inspection
        });
        Self::save(&self.0, "file", &sha256, &sha256, &record)?;
        Ok(record)
    }

    pub(crate) fn list(&self) -> Result<Value> {
        let mut stmt = self.0.prepare(
            "SELECT kind, record FROM model_registry WHERE kind IN ('file','ollama','report')
             ORDER BY kind, rowid DESC LIMIT 65",
        )?;
        let mut rows = stmt.query([])?;
        let mut entries = Vec::new();
        let mut reports = Vec::new();
        while let Some(row) = rows.next()? {
            if entries.len() + reports.len() == 64 {
                return Ok(
                    json!({"schema":"aether.model-registry/1","entries":entries,"reports":reports,"truncated":true}),
                );
            }
            let kind: String = row.get(0)?;
            let record: String = row.get(1)?;
            let value = serde_json::from_str::<Value>(&record)?;
            if kind == "report" {
                reports.push(value);
            } else {
                entries.push(value);
            }
        }
        Ok(
            json!({"schema":"aether.model-registry/1","entries":entries,"reports":reports,"truncated":false}),
        )
    }

    fn record(&self, lab: &Lab, report: &Value) -> Result<Value> {
        let hash = lab.record_development_artifact(report)?;
        let entry = json!({"artifact_sha256":hash,"provider":report["manifest"]["provider"],
            "suite_sha256":report["manifest"]["suite_sha256"],"model_sha256":report["manifest"]["model_sha256"],
            "manifest_sha256":report["manifest_sha256"],"complete":report["complete"],"summary":report["summary"]});
        Self::save(&self.0, "report", &hash, &hash, &entry)?;
        Ok(json!({"artifact_sha256":hash,"report":report}))
    }
}

pub(crate) fn local_root() -> Result<PathBuf> {
    std::env::var_os("OLLAMA_MODELS")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .or_else(|| std::env::var_os("HOME"))
                .map(|base| PathBuf::from(base).join(".ollama/models"))
        })
        .ok_or_else(|| "local Ollama model directory is unknown".into())
}
