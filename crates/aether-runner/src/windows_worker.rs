use std::io::{self, Read, Write};
use std::os::windows::{io::AsRawHandle, process::CommandExt};
use std::process::{Child, Command, Stdio};
use win32job::{ExtendedLimitInfo, Job};

const ADMITTED: u8 = b'A';
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

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
    let mut limits = ExtendedLimitInfo::new();
    limits.limit_kill_on_job_close();
    let job = Job::create_with_limit_info(&limits).map_err(io::Error::other)?;
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
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use std::time::{Duration, Instant};

    const FIXTURE: &str = "AETHER_WINDOWS_JOB_FIXTURE";

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
            let (mut parent, job) = spawn(&mut command)?;
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
