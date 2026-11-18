//! Bounded system queries without pipe-reader threads or inherited-pipe waits.
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_millis(500);
const MAX_OUTPUT: u64 = 64 * 1024;

struct Capture {
    file: Option<File>,
    path: PathBuf,
}
impl Capture {
    fn new() -> std::io::Result<Self> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        for _ in 0..32 {
            let path = std::env::temp_dir().join(format!(
                "egui-native-look-{}-{}.stdout",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt as _;
                options.mode(0o600);
            }
            match options.open(&path) {
                Ok(file) => {
                    return Ok(Self {
                        file: Some(file),
                        path,
                    })
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "system query capture names exhausted",
        ))
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(target_os = "linux")]
pub(super) fn query(program: &str, args: &[&str]) -> Option<Vec<u8>> {
    let mut command = Command::new(program);
    command.args(args);
    run(command, TIMEOUT, MAX_OUTPUT)
}

#[cfg(any(target_os = "linux", test))]
fn run(command: Command, timeout: Duration, limit: u64) -> Option<Vec<u8>> {
    run_inner(command, timeout, limit, false)
}

#[cfg(target_os = "macos")]
pub(super) fn query_allow_failure(program: &str, args: &[&str]) -> Option<Vec<u8>> {
    let mut command = Command::new(program);
    command.args(args);
    run_inner(command, TIMEOUT, MAX_OUTPUT, true)
}

fn run_inner(
    mut command: Command,
    timeout: Duration,
    limit: u64,
    allow_failure: bool,
) -> Option<Vec<u8>> {
    let start = Instant::now();
    let mut capture = Capture::new().ok()?;
    // A regular file cannot block waiting for EOF from an inherited stdout pipe.
    let output = capture.file.as_ref()?.try_clone().ok()?;
    command
        .stdin(Stdio::null())
        .stdout(output)
        .stderr(Stdio::null());
    let mut child = command.spawn().ok()?;
    drop(command);
    let status = loop {
        let size = capture.file.as_ref()?.metadata().map(|m| m.len());
        let done = child.try_wait();
        if start.elapsed() >= timeout
            || size.as_ref().map_or(true, |size| *size > limit)
            || done.is_err()
        {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        if let Some(status) = done.ok().flatten() {
            break status;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    if !allow_failure && !status.success() {
        return None;
    }
    let file = capture.file.as_mut()?;
    file.seek(SeekFrom::Start(0)).ok()?;
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= limit).then_some(bytes)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    fn shell(script: &str) -> Command {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", script]);
        command
    }
    #[test]
    fn captures_success_and_rejects_failed_commands() {
        // macOS `defaults` uses a failing status with empty stdout for light mode.
        assert_eq!(
            run_inner(shell("exit 1"), Duration::from_secs(1), 16, true),
            Some(Vec::new())
        );
        assert_eq!(
            run(shell("printf blue"), Duration::from_secs(1), 16),
            Some(b"blue".to_vec())
        );
        assert!(run(shell("printf blue; exit 1"), Duration::from_secs(1), 16).is_none());
    }
    #[test]
    fn bounds_output_and_reaps_a_timed_out_child() {
        assert!(run(shell("printf 123456789"), Duration::from_secs(1), 8).is_none());
        let start = Instant::now();
        assert!(run(shell("exec sleep 10"), Duration::from_millis(30), 16).is_none());
        assert!(start.elapsed() < Duration::from_secs(2));
    }
    #[test]
    fn inherited_stdout_does_not_extend_the_deadline() {
        let start = Instant::now();
        assert_eq!(
            run(shell("sleep 2 & printf ok"), Duration::from_millis(100), 16),
            Some(b"ok".to_vec())
        );
        assert!(start.elapsed() < Duration::from_secs(1));
    }
}
