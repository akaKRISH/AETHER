use std::io::{self, Read, Write};
use std::os::windows::{io::AsRawHandle, process::CommandExt};
use std::process::{Child, Command, Stdio};
use win32job::Job;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::{
    JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_JOB_TIME,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_TIME,
    JOBOBJECT_BASIC_LIMIT_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectExtendedLimitInformation, SetInformationJobObject,
};

const ADMITTED: u8 = b'A';
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const JOB_CPU_TICKS: i64 = 10_000_000; // One second of user-mode CPU in 100 ns units.
const JOB_COMMIT_BYTES: usize = 64 * 1024 * 1024;

#[allow(unsafe_code)]
fn limited_job(active_processes: u32) -> io::Result<Job> {
    let job = Job::create().map_err(io::Error::other)?;
    let limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
            PerProcessUserTimeLimit: JOB_CPU_TICKS,
            PerJobUserTimeLimit: JOB_CPU_TICKS,
            ActiveProcessLimit: active_processes,
            LimitFlags: JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                | JOB_OBJECT_LIMIT_JOB_MEMORY
                | JOB_OBJECT_LIMIT_JOB_TIME
                | JOB_OBJECT_LIMIT_PROCESS_TIME
                | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            ..Default::default()
        },
        JobMemoryLimit: JOB_COMMIT_BYTES,
        ..Default::default()
    };
    // SAFETY: job owns a live handle, and Win32 reads this matching structure during the call.
    unsafe {
        SetInformationJobObject(
            HANDLE(job.handle() as _),
            JobObjectExtendedLimitInformation,
            std::ptr::from_ref(&limits).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    }
    .map_err(io::Error::other)?;
    Ok(job)
}

pub fn wait_for_admission() -> io::Result<()> {
    let mut gate = [0];
    io::stdin().lock().read_exact(&mut gate)?;
    if gate != [ADMITTED] {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "reviewed worker admission rejected",
        ));
    }
    Ok(())
}

pub(crate) fn spawn(command: &mut Command) -> io::Result<(Child, Job)> {
    let job = limited_job(1)?;
    spawn_in_job(command, job)
}

fn spawn_in_job(command: &mut Command, job: Job) -> io::Result<(Child, Job)> {
    let mut child = command
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::piped())
        .spawn()?;
    let admission = job
        .assign_process(child.as_raw_handle() as isize)
        .map_err(io::Error::other)
        .and_then(|()| {
            child
                .stdin
                .take()
                .ok_or_else(|| io::Error::other("worker admission pipe unavailable"))?
                .write_all(&[ADMITTED])
        });
    if let Err(error) = admission {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    Ok((child, job))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ATTEMPT_DIRECTORY, Error, collect_worker, worker_command};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};
    use windows::Win32::Foundation::{ERROR_NOT_ENOUGH_QUOTA, STATUS_QUOTA_EXCEEDED};

    const FIXTURE: &str = "AETHER_WINDOWS_JOB_FIXTURE";
    const SECRET: &str = "AETHER_TEST_SECRET_CANARY";

    fn fixture(name: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args([
            "--ignored",
            "--exact",
            &format!("windows_worker::tests::{name}"),
        ]);
        command.env_clear();
        command
    }

    fn assert_fixture(name: &str) {
        let (mut child, _job) = spawn(&mut fixture(name)).unwrap();
        assert!(child.wait().unwrap().success(), "{name} failed");
    }

    #[test]
    #[ignore = "run by the CPU limit test"]
    fn cpu_fixture() {
        wait_for_admission().unwrap();
        let mut value = 1_u64;
        loop {
            value = std::hint::black_box(value.wrapping_mul(1_664_525).wrapping_add(1));
        }
    }

    #[test]
    fn job_cpu_limit_terminates_busy_worker() {
        let start = Instant::now();
        let (mut child, _job) = spawn(&mut fixture("cpu_fixture")).unwrap();
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(
                    [
                        Some(STATUS_QUOTA_EXCEEDED.0),
                        Some(ERROR_NOT_ENOUGH_QUOTA.0 as i32)
                    ]
                    .contains(&status.code()),
                    "{status}"
                );
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(12),
                "CPU fixture did not exit"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    #[ignore = "run by the job commit limit test"]
    fn memory_fixture() {
        wait_for_admission().unwrap();
        let mut bytes = Vec::<u8>::new();
        assert!(bytes.try_reserve_exact(1024 * 1024).is_ok());
        assert!(bytes.try_reserve_exact(JOB_COMMIT_BYTES + 4096).is_err());
    }

    #[test]
    fn job_commit_limit_refuses_oversized_allocation() {
        assert_fixture("memory_fixture");
    }

    #[test]
    #[ignore = "run by the process limit test"]
    fn no_child_fixture() {
        wait_for_admission().unwrap();
        assert!(fixture("uncontained_fixture").status().is_err());
    }

    #[test]
    #[ignore = "child denied by the process limit"]
    fn uncontained_fixture() {}

    #[test]
    fn job_process_limit_prevents_child_creation() {
        assert_fixture("no_child_fixture");
    }

    #[test]
    #[ignore = "run by the environment isolation test"]
    fn secret_parent_fixture() {
        assert_eq!(std::env::var(SECRET).as_deref(), Ok("synthetic-secret"));
        let mut command = worker_command(
            &std::env::current_exe().unwrap(),
            PathBuf::from(".").as_path(),
        );
        command.args([
            "--ignored",
            "--exact",
            "windows_worker::tests::secret_child_fixture",
        ]);
        assert!(
            collect_worker(
                &mut command,
                Duration::from_secs(3),
                &AtomicBool::new(false)
            )
            .is_ok()
        );
    }

    #[test]
    #[ignore = "run by the environment isolation test"]
    fn secret_child_fixture() {
        wait_for_admission().unwrap();
        assert!(std::env::var_os(SECRET).is_none());
    }

    #[test]
    fn worker_does_not_inherit_parent_secret() {
        assert!(
            fixture("secret_parent_fixture")
                .env(SECRET, "synthetic-secret")
                .status()
                .unwrap()
                .success()
        );
    }

    #[test]
    #[ignore = "run by the stop tests"]
    fn sleeping_fixture() {
        wait_for_admission().unwrap();
        fs::write("ready", b"ready").unwrap();
        thread::sleep(Duration::from_secs(2));
        fs::write("survived", b"survived").unwrap();
    }

    fn check_stop(cancel: bool) {
        let root = std::env::temp_dir().join(format!(
            "aether-stop-{}-{}",
            std::process::id(),
            ATTEMPT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let flag = AtomicBool::new(false);
        let mut command = worker_command(&std::env::current_exe().unwrap(), &root);
        command.args([
            "--ignored",
            "--exact",
            "windows_worker::tests::sleeping_fixture",
        ]);
        let started = Instant::now();
        let result = thread::scope(|scope| {
            if cancel {
                scope.spawn(|| {
                    while !root.join("ready").is_file()
                        && started.elapsed() < Duration::from_secs(2)
                    {
                        thread::sleep(Duration::from_millis(5));
                    }
                    flag.store(true, Ordering::Release);
                });
            }
            let timeout = if cancel {
                Duration::from_secs(3)
            } else {
                Duration::from_secs(1)
            };
            collect_worker(&mut command, timeout, &flag)
        });
        let ready = root.join("ready").is_file();
        thread::sleep(Duration::from_millis(2100));
        let survived = root.join("survived").is_file();
        for name in ["ready", "survived"] {
            let file = root.join(name);
            if file.is_file() {
                fs::remove_file(file).unwrap();
            }
        }
        fs::remove_dir(root).unwrap();
        assert!(ready, "worker did not enter its working directory");
        assert!(!survived, "worker survived stop");
        if cancel {
            assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
        } else {
            assert!(matches!(result, Err(Error::Timeout)), "{result:?}");
        }
    }

    #[test]
    fn active_worker_cancellation_reaps_worker() {
        check_stop(true);
    }

    #[test]
    fn wall_timeout_reaps_worker() {
        check_stop(false);
    }

    #[test]
    #[ignore = "spawned by the job containment test"]
    fn parent_fixture() {
        wait_for_admission().unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "windows_worker::tests::descendant_fixture",
            ])
            .spawn()
            .unwrap();
        child.wait().unwrap();
    }

    #[test]
    #[ignore = "spawned by the job containment test"]
    fn descendant_fixture() {
        let root = PathBuf::from(std::env::var_os(FIXTURE).unwrap());
        fs::write(root.join("ready"), b"ready").unwrap();
        thread::sleep(Duration::from_secs(1));
        fs::write(root.join("survived"), b"survived").unwrap();
    }

    #[test]
    fn dropping_the_job_stops_a_running_descendant() {
        let root = std::env::temp_dir().join(format!("aether-job-test-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let result = (|| -> io::Result<()> {
            let mut command = Command::new(std::env::current_exe()?);
            command
                .args([
                    "--ignored",
                    "--exact",
                    "windows_worker::tests::parent_fixture",
                ])
                .env_clear()
                .env(FIXTURE, &root)
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            // This fixture permits one descendant to exercise job-close cleanup.
            let (mut parent, job) = spawn_in_job(&mut command, limited_job(2)?)?;
            let deadline = Instant::now() + Duration::from_secs(3);
            while !root.join("ready").exists() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            let running = root.join("ready").exists() && parent.try_wait()?.is_none();
            drop(job);
            parent.wait()?;
            if !running {
                return Err(io::Error::other(
                    "reviewed fixture did not start with a live descendant",
                ));
            }
            thread::sleep(Duration::from_millis(1100));
            if root.join("survived").exists() {
                return Err(io::Error::other("descendant survived the job close"));
            }
            Ok(())
        })();
        for name in ["ready", "survived"] {
            let path = root.join(name);
            if path.is_file() {
                fs::remove_file(path).unwrap();
            }
        }
        fs::remove_dir(root).unwrap();
        result.unwrap();
    }
}
