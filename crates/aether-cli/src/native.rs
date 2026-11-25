use std::env;
use std::fmt::{self, Write as _};
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aether_core::{Accuracy, EVALUATOR_VERSION, EvalError, FIXTURE_VERSION, evaluate, fixture};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const PROJECT_FILE: &str = "project.json";
const SCHEMA_VERSION: u32 = 1;
const MAX_RECORD_BYTES: u64 = 16 * 1024;
const STAGE_PREFIX: &str = ".stage-";

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum Error {
    Input(String),
    Policy(String),
    Io {
        action: &'static str,
        source: io::Error,
    },
    Integrity(String),
    Evaluation(EvalError),
}

impl Error {
    pub(crate) fn exit_code(&self) -> u8 {
        match self {
            Self::Input(_) => 2,
            Self::Policy(_) => 3,
            Self::Io { .. } | Self::Evaluation(_) => 4,
            Self::Integrity(_) => 5,
        }
    }

    fn io(action: &'static str, source: io::Error) -> Self {
        Self::Io { action, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(message) | Self::Policy(message) | Self::Integrity(message) => {
                f.write_str(message)
            }
            Self::Io { action, source } => write!(f, "{action}: {source}"),
            Self::Evaluation(source) => write!(f, "fixture evaluation failed: {source}"),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Project {
    schema_version: u32,
    project_id: String,
}

impl Project {
    fn campaign_id(&self) -> String {
        format!("campaign-{}", &self.project_id["project-".len()..])
    }

    fn summary(&self) -> String {
        format!(
            "Project: {}\nFixture campaign: {}",
            self.project_id,
            self.campaign_id()
        )
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AttemptStatus {
    Completed,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    project_id: String,
    campaign_id: String,
    run_id: String,
    fixture_version: String,
    evaluator_version: String,
    input_sha256: String,
    correct: usize,
    total: usize,
    status: AttemptStatus,
}

pub(crate) fn init() -> Result<String> {
    let state = state_directory()?;
    create_directory(&state)?;
    let project_path = state.join(PROJECT_FILE);
    if let Some(project) = read_project(&project_path)? {
        return Ok(project.summary());
    }

    for entry in fs::read_dir(&state).map_err(|err| Error::io("inspect .aether", err))? {
        let entry = entry.map_err(|err| Error::io("read .aether entry", err))?;
        let name = entry.file_name();
        if name == "lab" && is_directory(&entry.path())? {
            // The local workbench owns this sibling directory.
            continue;
        }
        if !name
            .to_str()
            .is_some_and(|name| valid_id(name, STAGE_PREFIX))
        {
            // Another initializer may have committed while this process read the directory.
            if let Some(project) = read_project(&project_path)? {
                return Ok(project.summary());
            }
            return Err(Error::Policy(".aether contains unrecognized state".into()));
        }
    }

    let project = Project {
        schema_version: SCHEMA_VERSION,
        project_id: new_id("project-"),
    };
    if publish(&state, &project_path, &encode(&project)?)? {
        Ok(project.summary())
    } else {
        Ok(load_project(&state)?.summary())
    }
}

pub(crate) fn run_fixture() -> Result<String> {
    let state = state_directory()?;
    let project = load_project(&state)?;
    let (score, input_sha256) = fixture_result()?;
    let manifest = Manifest {
        schema_version: SCHEMA_VERSION,
        campaign_id: project.campaign_id(),
        project_id: project.project_id,
        run_id: new_id("run-"),
        fixture_version: FIXTURE_VERSION.into(),
        evaluator_version: EVALUATOR_VERSION.into(),
        input_sha256,
        correct: score.correct,
        total: score.total,
        status: AttemptStatus::Completed,
    };
    let bytes = encode(&manifest)?;
    let digest = digest(&bytes);

    let fixture = state.join("fixture");
    create_directory(&fixture)?;
    let manifests = fixture.join("manifests");
    let runs = fixture.join("runs");
    create_directory(&manifests)?;
    create_directory(&runs)?;

    let blob = manifests.join(format!("{digest}.json"));
    if !publish(&manifests, &blob, &bytes)?
        && read_small(&blob)?.as_deref() != Some(bytes.as_slice())
    {
        return Err(Error::Integrity(
            "existing manifest bytes differ from their digest".into(),
        ));
    }
    if !publish(&runs, &runs.join(&manifest.run_id), digest.as_bytes())? {
        return Err(Error::io(
            "publish run",
            io::Error::new(io::ErrorKind::AlreadyExists, "run identifier collision"),
        ));
    }

    Ok(format!(
        "Run: {}\nCampaign: {}\nAccuracy: {}/{} = {:.2}\nManifest SHA-256: {}",
        manifest.run_id,
        manifest.campaign_id,
        score.correct,
        score.total,
        score.correct as f64 / score.total as f64,
        digest
    ))
}

pub(crate) fn verify(run_id: &str) -> Result<String> {
    if !valid_id(run_id, "run-") {
        return Err(Error::Input("invalid run ID".into()));
    }
    let state = state_directory()?;
    let project = load_project(&state)?;
    let fixture = state.join("fixture");
    let runs = fixture.join("runs");
    if !is_directory(&fixture)? || !is_directory(&runs)? {
        return Err(missing_run());
    }
    let (manifest, _) = read_and_replay(&fixture, &project, run_id)?;
    Ok(format!(
        "Fixture replay passed for {} ({}/{}).",
        manifest.run_id, manifest.correct, manifest.total
    ))
}

pub(crate) fn report(campaign_id: &str) -> Result<String> {
    if !valid_id(campaign_id, "campaign-") {
        return Err(Error::Input("invalid campaign ID".into()));
    }
    let state = state_directory()?;
    let project = load_project(&state)?;
    if campaign_id != project.campaign_id() {
        return Err(Error::Input(
            "campaign ID does not exist in this project".into(),
        ));
    }

    let mut report = format!("Campaign: {campaign_id}");
    let fixture = state.join("fixture");
    let runs = fixture.join("runs");
    let mut run_ids = Vec::new();
    if is_directory(&fixture)? && is_directory(&runs)? {
        for entry in fs::read_dir(&runs).map_err(|err| Error::io("list fixture runs", err))? {
            let name = entry
                .map_err(|err| Error::io("read fixture run entry", err))?
                .file_name()
                .into_string()
                .map_err(|_| Error::Integrity("invalid stored run name".into()))?;
            if valid_id(&name, STAGE_PREFIX) {
                continue;
            }
            if !valid_id(&name, "run-") {
                return Err(Error::Integrity("invalid stored run name".into()));
            }
            run_ids.push(name);
        }
    }

    if run_ids.is_empty() {
        report.push_str("\nNo committed fixture runs.");
    } else {
        run_ids.sort_unstable();
        for run_id in run_ids {
            let (manifest, digest) = match read_and_replay(&fixture, &project, &run_id) {
                Err(Error::Input(_)) => {
                    return Err(Error::Integrity(
                        "listed run pointer disappeared during report".into(),
                    ));
                }
                result => result?,
            };
            write!(
                report,
                "\n{} completed {}/{} manifest={}",
                run_id, manifest.correct, manifest.total, digest
            )
            .expect("writing to String is infallible");
        }
    }
    report.push_str("\nEvidence scope: deterministic fixture replay only.");
    Ok(report)
}

fn read_and_replay(fixture: &Path, project: &Project, run_id: &str) -> Result<(Manifest, String)> {
    let entry = fixture.join("runs").join(run_id);
    let reference = read_small(&entry)?.ok_or_else(missing_run)?;
    let expected = std::str::from_utf8(&reference)
        .ok()
        .filter(|value| valid_hex(value, 64))
        .ok_or_else(|| Error::Integrity("invalid manifest digest reference".into()))?;

    let manifests = fixture.join("manifests");
    if !is_directory(&manifests)? {
        return Err(Error::Integrity(
            "fixture manifests directory is missing".into(),
        ));
    }
    let bytes = read_small(&manifests.join(format!("{expected}.json")))?
        .ok_or_else(|| Error::Integrity("referenced manifest is missing".into()))?;
    if digest(&bytes) != expected {
        return Err(Error::Integrity("manifest digest mismatch".into()));
    }

    let recorded: Manifest = serde_json::from_slice(&bytes)
        .map_err(|_| Error::Integrity("malformed fixture manifest".into()))?;
    if recorded.schema_version != SCHEMA_VERSION
        || recorded.project_id != project.project_id
        || recorded.campaign_id != project.campaign_id()
        || recorded.run_id != run_id
        || recorded.fixture_version != FIXTURE_VERSION
        || recorded.evaluator_version != EVALUATOR_VERSION
    {
        return Err(Error::Integrity(
            "manifest identity or version mismatch".into(),
        ));
    }

    let (score, input_sha256) = fixture_result()?;
    if recorded.input_sha256 != input_sha256
        || recorded.correct != score.correct
        || recorded.total != score.total
    {
        return Err(Error::Integrity(
            "fixture inputs or exact counts do not reproduce".into(),
        ));
    }
    Ok((recorded, expected.to_owned()))
}

fn fixture_result() -> Result<(Accuracy, String)> {
    let (labels, predictions) = fixture();
    let score = evaluate(labels, predictions).map_err(Error::Evaluation)?;
    // Version 1 hashes the compact JSON tuple [[labels...],[predictions...]].
    let inputs = encode(&(labels, predictions))?;
    Ok((score, digest(&inputs)))
}

fn load_project(state: &Path) -> Result<Project> {
    if is_directory(state)?
        && let Some(project) = read_project(&state.join(PROJECT_FILE))?
    {
        return Ok(project);
    }
    Err(Error::Input(
        "project is not initialized, run aether init".into(),
    ))
}

fn read_project(path: &Path) -> Result<Option<Project>> {
    let Some(bytes) = read_small(path)? else {
        return Ok(None);
    };
    let project: Project = serde_json::from_slice(&bytes)
        .map_err(|_| Error::Integrity("malformed project metadata".into()))?;
    if project.schema_version != SCHEMA_VERSION || !valid_id(&project.project_id, "project-") {
        return Err(Error::Integrity(
            "invalid project identity or version".into(),
        ));
    }
    Ok(Some(project))
}

fn state_directory() -> Result<PathBuf> {
    env::current_dir()
        .map(|current| current.join(".aether"))
        .map_err(|err| Error::io("resolve current project directory", err))
}

fn create_directory(path: &Path) -> Result<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists && is_directory(path)? => Ok(()),
        Err(err) => Err(Error::io("create project state directory", err)),
    }
}

fn is_directory(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !is_link(&metadata) => Ok(true),
        Ok(_) => Err(Error::Policy(
            "project state requires an ordinary directory".into(),
        )),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(Error::io("inspect project state directory", err)),
    }
}

fn is_link(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn read_small(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_file() || is_link(&metadata) => {
            return Err(Error::Policy(
                "project records must be ordinary files".into(),
            ));
        }
        Ok(metadata) if metadata.len() > MAX_RECORD_BYTES => {
            return Err(Error::Integrity(
                "project record exceeds the size limit".into(),
            ));
        }
        Ok(_) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(Error::io("inspect project record", err)),
    }
    let file = File::open(path).map_err(|err| Error::io("open project record", err))?;
    let mut bytes = Vec::new();
    file.take(MAX_RECORD_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|err| Error::io("read project record", err))?;
    if bytes.len() as u64 > MAX_RECORD_BYTES {
        return Err(Error::Integrity(
            "project record exceeds the size limit".into(),
        ));
    }
    Ok(Some(bytes))
}

/// Publish an immutable record using an atomic, non-replacing hard link.
fn publish(directory: &Path, destination: &Path, bytes: &[u8]) -> Result<bool> {
    let stage = directory.join(new_id(STAGE_PREFIX));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&stage)
        .map_err(|err| Error::io("create staging record", err))?;

    let result = (|| {
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|err| Error::io("write staging record", err))?;
        drop(file);

        match fs::hard_link(&stage, destination) {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(false),
            Err(err) => Err(Error::io("publish project record", err)),
        }
    })();

    // Only the destination is authoritative, so cleanup failure cannot undo a commit.
    let _ = fs::remove_file(&stage);
    result
}

fn encode<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|err| Error::io("serialize record", io::Error::other(err)))
}

fn valid_id(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|token| valid_hex(token, 32))
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn new_id(prefix: &str) -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    let mut hash = Sha256::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    hash.update(now.as_nanos().to_be_bytes());
    hash.update(std::process::id().to_be_bytes());
    hash.update(SEQUENCE.fetch_add(1, Ordering::Relaxed).to_be_bytes());
    let token = format!("{:x}", hash.finalize());
    format!("{prefix}{}", &token[..32])
}

fn missing_run() -> Error {
    Error::Input("run ID does not exist in this project".into())
}
