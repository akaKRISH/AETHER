use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "aether-fixture-test-{}-{nanos}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn command(&self, arguments: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_aether"));
        command.current_dir(&self.0).args(arguments);
        command
    }

    fn run(&self, arguments: &[&str]) -> Output {
        self.command(arguments).output().unwrap()
    }

    fn state(&self) -> PathBuf {
        self.0.join(".aether")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn field<'a>(text: &'a str, label: &str) -> &'a str {
    text.lines()
        .find_map(|line| line.strip_prefix(label))
        .unwrap()
}

fn assert_error(output: Output, code: i32, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains(diagnostic)
    );
}

fn manifest_path(state: &Path, digest: &str) -> PathBuf {
    state
        .join("fixture/manifests")
        .join(format!("{digest}.json"))
}

#[test]
fn fresh_commands_preserve_metadata_and_replay_distinct_runs() {
    let env = Scratch::new();
    let init = stdout(env.run(&["init"]));
    let metadata = fs::read(env.state().join("project.json")).unwrap();
    assert_eq!(stdout(env.run(&["init"])), init);
    assert_eq!(
        fs::read(env.state().join("project.json")).unwrap(),
        metadata
    );
    let campaign = field(&init, "Fixture campaign: ");

    let first = stdout(env.run(&["run-fixture"]));
    let second = stdout(env.run(&["run-fixture"]));
    let first_id = field(&first, "Run: ");
    let second_id = field(&second, "Run: ");
    assert_ne!(first_id, second_id);
    for run in [&first, &second] {
        assert!(run.contains("Accuracy: 3/4 = 0.75"));
        assert_eq!(field(run, "Campaign: "), campaign);
        let run_id = field(run, "Run: ");
        let digest = field(run, "Manifest SHA-256: ");
        let bytes = fs::read(manifest_path(&env.state(), digest)).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), digest);
        assert_eq!(
            fs::read(env.state().join("fixture/runs").join(run_id)).unwrap(),
            digest.as_bytes()
        );
        assert!(
            stdout(env.run(&["verify", run_id]))
                .contains(&format!("Fixture replay passed for {run_id} (3/4)."))
        );
    }

    let report = stdout(env.run(&["report", campaign]));
    let mut ordered = [first_id, second_id];
    ordered.sort_unstable();
    assert!(report.find(ordered[0]).unwrap() < report.find(ordered[1]).unwrap());
    assert_eq!(report.matches("completed 3/4").count(), 2);
    assert!(report.contains("deterministic fixture replay only"));
}

#[test]
fn concurrent_init_uses_one_identity() {
    let env = Scratch::new();
    let mut first = env.command(&["init"]);
    let mut second = env.command(&["init"]);
    let first = first
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let second = second
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let one = stdout(first.wait_with_output().unwrap());
    let two = stdout(second.wait_with_output().unwrap());
    assert_eq!(one, two);
}

#[test]
fn native_state_coexists_with_the_workbench_namespace() {
    let env = Scratch::new();
    let lab = env.state().join("lab");
    fs::create_dir_all(&lab).unwrap();
    let existing = lab.join("workbench-owned");
    fs::write(&existing, b"preserve").unwrap();
    let init = stdout(env.run(&["init"]));
    assert!(init.contains("Fixture campaign: "));
    assert!(stdout(env.run(&["run-fixture"])).contains("Accuracy: 3/4 = 0.75"));
    assert_eq!(fs::read(existing).unwrap(), b"preserve");
}

#[test]
fn abandoned_staging_and_unreferenced_blob_are_not_complete() {
    let env = Scratch::new();
    let init = stdout(env.run(&["init"]));
    let run = stdout(env.run(&["run-fixture"]));
    let campaign = field(&init, "Fixture campaign: ");
    let run_id = field(&run, "Run: ");
    let digest = field(&run, "Manifest SHA-256: ");
    let runs = env.state().join("fixture/runs");
    fs::rename(
        runs.join(run_id),
        runs.join(".stage-00000000000000000000000000000000"),
    )
    .unwrap();

    assert!(manifest_path(&env.state(), digest).exists());
    assert!(stdout(env.run(&["report", campaign])).contains("No committed fixture runs."));
    assert_error(env.run(&["verify", run_id]), 2, "does not exist");
    let retry = stdout(env.run(&["run-fixture"]));
    assert_ne!(field(&retry, "Run: "), run_id);
    assert!(stdout(env.run(&["verify", field(&retry, "Run: ")])).contains("replay passed"));
}

#[test]
fn invalid_and_tampered_input_cannot_produce_verified_output() {
    let env = Scratch::new();
    assert_error(env.run(&["verify", "../project.json"]), 2, "invalid run ID");
    assert!(!env.state().exists());
    assert_error(env.run(&["run-fixture"]), 2, "not initialized");

    let campaign = stdout(env.run(&["init"]));
    let campaign = field(&campaign, "Fixture campaign: ");
    let intact = stdout(env.run(&["run-fixture"]));
    let modified = stdout(env.run(&["run-fixture"]));
    let digest = field(&modified, "Manifest SHA-256: ");
    let path = manifest_path(&env.state(), digest);
    let mut bytes = fs::read(&path).unwrap();
    bytes[0] ^= 1;
    fs::write(path, bytes).unwrap();

    assert_error(
        env.run(&["verify", field(&modified, "Run: ")]),
        5,
        "digest mismatch",
    );
    assert_error(env.run(&["report", campaign]), 5, "digest mismatch");
    assert!(stdout(env.run(&["verify", field(&intact, "Run: ")])).contains("replay passed"));
}

#[test]
fn init_leaves_conflicting_files_intact() {
    let env = Scratch::new();
    fs::create_dir(env.state()).unwrap();
    let foreign = env.state().join("foreign");
    fs::write(&foreign, b"original").unwrap();

    assert_error(env.run(&["init"]), 3, "unrecognized state");
    assert_eq!(fs::read(foreign).unwrap(), b"original");
    assert!(!env.state().join("project.json").exists());
}

#[test]
fn help_lists_commands_and_integrity_exit_code() {
    let env = Scratch::new();
    let help = stdout(env.run(&["--help"]));
    for command in [
        "init",
        "run-fixture",
        "verify <run-id>",
        "report <campaign-id>",
    ] {
        assert!(help.contains(command));
    }
    assert!(help.contains("5 integrity failure"));
}
