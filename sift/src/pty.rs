//! PTY management — spawn real bash via portable-pty, read/write PTY, signal handling.
//!
//! Finds the real bash binary by scanning PATH, excluding our own binary
//! (since baish is installed as `bash` to shadow the real one).

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex as StdMutex};

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, PtySystem};

/// Find the real bash binary, excluding our own path.
fn find_real_bash() -> PathBuf {
    let self_path = std::env::current_exe().ok();
    let path_var = std::env::var("PATH").unwrap_or_default();

    for dir in path_var.split(':') {
        let candidate = PathBuf::from(dir).join("bash");
        if candidate.is_file() {
            if let Ok(canonical) = candidate.canonicalize() {
                if self_path.as_ref().is_some_and(|s| s == &canonical) {
                    continue;
                }
                return canonical;
            }
        }
    }

    for fallback in &["/bin/bash", "/usr/bin/bash", "/usr/local/bin/bash"] {
        let p = PathBuf::from(fallback);
        if p.exists() {
            return p;
        }
    }

    PathBuf::from("/bin/bash")
}

/// A PTY session running a real bash process.
pub struct PtySession {
    reader: Arc<StdMutex<Box<dyn Read + Send>>>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send>,
    _pty_system: Box<dyn PtySystem>,
    line_buf: String,
}

impl PtySession {
    /// Spawn a new bash session in a PTY.
    pub fn spawn() -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to open PTY")?;

        let bash_path = find_real_bash();
        let cmd = CommandBuilder::new(&bash_path);
        let child = pair
            .slave
            .spawn_command(cmd)
            .with_context(|| format!("failed to spawn {}", bash_path.display()))?;

        let reader = Arc::new(StdMutex::new(
            pair.master
                .try_clone_reader()
                .context("failed to get PTY reader")?,
        ));
        let writer = pair.master.take_writer().context("failed to get PTY writer")?;

        Ok(Self {
            reader,
            writer,
            child,
            _pty_system: pty_system,
            line_buf: String::new(),
        })
    }

    /// Send a command and read output with a timeout.
    pub fn send_command(&mut self, cmd: &str) -> Result<Vec<u8>> {
        let full_cmd = format!("{cmd}\n");
        self.writer
            .write_all(full_cmd.as_bytes())
            .context("failed to write to PTY")?;
        self.writer.flush()?;

        let (tx, rx) = mpsc::channel();
        let reader = Arc::clone(&self.reader);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let mut buf = [0u8; 4096];
            let mut guard = reader.lock().unwrap();
            let mut output = Vec::new();
            while output.len() < 1024 * 1024 {
                match guard.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        output.extend_from_slice(&buf[..n]);
                    }
                }
            }
            let _ = tx.send(output);
        });

        rx.recv_timeout(std::time::Duration::from_secs(5))
            .map_or_else(|_| Ok(Vec::new()), Ok)
    }

    /// Read lines from the PTY, handling partial lines at chunk boundaries.
    pub fn read_lines(&mut self) -> Vec<String> {
        let mut guard = self.reader.lock().unwrap();
        let mut buf = [0u8; 4096];
        let mut raw = Vec::new();
        while let Ok(n) = guard.read(&mut buf) {
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&buf[..n]);
        }
        drop(guard);

        if raw.is_empty() {
            return Vec::new();
        }

        let chunk = String::from_utf8_lossy(&raw);
        let mut full = std::mem::take(&mut self.line_buf);
        full.push_str(&chunk);

        let mut lines = Vec::new();
        let mut current = String::new();
        for ch in full.chars() {
            if ch == '\n' {
                lines.push(std::mem::take(&mut current));
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            self.line_buf = current;
        }

        lines
    }

    /// Check if the child process is still running.
    pub fn is_alive(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }

    /// Wait for the child to exit and return the exit code.
    pub fn wait(&mut self) -> Result<i32> {
        let status = self
            .child
            .wait()
            .context("failed to wait for child process")?;
        Ok(status.exit_code().cast_signed())
    }

    /// Send a signal to the child process group.
    #[cfg(unix)]
    pub fn send_signal(&self, signal: nix::sys::signal::Signal) -> Result<()> {
        use nix::sys::signal::killpg;
        use nix::unistd::Pid;
        if let Some(pid) = self.child.process_id() {
            if pid > 0 {
                killpg(Pid::from_raw(pid.cast_signed()), signal)
                    .context("failed to send signal to process group")?;
            }
        }
        Ok(())
    }

    /// Get a reference to the PTY writer.
    pub fn writer(&mut self) -> &mut Box<dyn Write + Send> {
        &mut self.writer
    }

    /// Get a reference to the PTY reader's mutex.
    pub fn reader(&self) -> &Arc<StdMutex<Box<dyn Read + Send>>> {
        &self.reader
    }
}

/// Execute a command through a one-shot PTY session.
pub fn execute_command(cmd: &str) -> Result<i32> {
    let mut session = PtySession::spawn()?;
    let full_cmd = format!("{cmd}; exit $?\n");
    session
        .writer
        .write_all(full_cmd.as_bytes())
        .context("failed to write to PTY")?;
    session.writer.flush()?;
    let exit_code = session.wait()?;
    Ok(exit_code)
}

/// Split a byte chunk into lines, handling partial lines at boundaries.
#[must_use]
pub fn split_lines(chunk: &[u8], partial: &str) -> (Vec<String>, String) {
    let mut text = partial.to_string();
    text.push_str(&String::from_utf8_lossy(chunk));

    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch == '\n' {
            lines.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    (lines, current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_real_bash() {
        let bash = find_real_bash();
        assert!(bash.exists(), "real bash should exist at {bash:?}");
    }

    #[test]
    fn test_pty_session_spawn() {
        let mut session = PtySession::spawn().expect("failed to spawn PTY session");
        assert!(session.is_alive(), "bash should be running");
    }

    #[test]
    fn test_pty_exit_code() {
        let mut session = PtySession::spawn().expect("failed to spawn PTY session");
        session
            .send_command("exit 42")
            .expect("failed to send exit command");
        let code = session.wait().expect("failed to wait");
        assert_eq!(code, 42, "exit code should be 42, got {code}");
    }

    #[test]
    fn test_read_lines() {
        let mut session = PtySession::spawn().expect("failed to spawn PTY session");
        // Send a command that produces output, then exit
        session
            .writer()
            .write_all(b"echo hello\nexit 42\n")
            .unwrap();
        session.writer().flush().unwrap();
        let lines = session.read_lines();
        assert!(!lines.is_empty(), "should have read some lines");
        let code = session.wait().expect("failed to wait");
        assert_eq!(code, 42, "exit code should be 42, got {code}");
    }

    #[test]
    fn test_split_lines_complete() {
        let (lines, partial) = split_lines(b"hello\nworld\n", "");
        assert_eq!(lines, vec!["hello", "world"]);
        assert!(partial.is_empty());
    }

    #[test]
    fn test_split_lines_partial() {
        let (lines, partial) = split_lines(b"hello\nworld", "");
        assert_eq!(lines, vec!["hello"]);
        assert_eq!(partial, "world");
    }

    #[test]
    fn test_split_lines_with_existing_partial() {
        let (lines, partial) = split_lines(b" world\n", "hello");
        assert_eq!(lines, vec!["hello world"]);
        assert!(partial.is_empty());
    }

    #[test]
    fn test_split_lines_multiple_chunks() {
        let (lines, partial) = split_lines(b"hello\nwor", "");
        assert_eq!(lines, vec!["hello"]);
        assert_eq!(partial, "wor");

        let (lines, partial) = split_lines(b"ld\n", &partial);
        assert_eq!(lines, vec!["world"]);
        assert!(partial.is_empty());
    }

    #[test]
    fn test_split_lines_empty() {
        let (lines, partial) = split_lines(b"", "");
        assert!(lines.is_empty());
        assert!(partial.is_empty());
    }
}
