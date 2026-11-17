//! Running a tool's command: no shell, no input, a time limit and a limit to what is kept.

use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The most that is kept of what a command writes, of each of its two outputs.
pub const LIMIT: usize = 64 * 1024;

#[derive(Debug)]
pub struct Outcome {
    /// The exit status; None when it was stopped, or never started.
    pub exit: Option<i32>,
    pub timed_out: bool,
    pub stdout: String,
    pub stderr: String,
    /// Whether it wrote more than is kept.
    pub truncated: bool,
    /// Why it could not be started, when it could not.
    pub problem: Option<String>,
}

// The one thing asked of the C library: a signal for a whole group of processes, which
// std cannot send. A command that is out of time is stopped with everything it started.
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}
const SIGKILL: i32 = 9;

/// Reads an output to its end, keeping its first `limit` bytes: the rest is read and
/// dropped, so that the command is never left waiting for someone to listen.
fn listen(mut from: impl Read + Send + 'static, limit: usize) -> mpsc::Receiver<(Vec<u8>, bool)> {
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        let mut truncated = false;
        let mut buffer = [0u8; 8192];
        while let Ok(count) = from.read(&mut buffer) {
            if count == 0 {
                break;
            }
            let room = limit - kept.len();
            kept.extend_from_slice(&buffer[..count.min(room)]);
            truncated |= count > room;
        }
        let _ = send.send((kept, truncated));
    });
    receive
}

/// Runs a command line as it is and waits for it, at most `timeout`.
pub fn run(line: &[String], timeout: Duration, limit: usize) -> Outcome {
    let mut outcome = Outcome { exit: None, timed_out: false, stdout: String::new(), stderr: String::new(), truncated: false, problem: None };
    let Some((program, arguments)) = line.split_first() else {
        outcome.problem = Some("there is no command to run".into());
        return outcome;
    };
    let mut command = Command::new(program);
    // a group of its own, so that it can be stopped whole; and nothing to read: a
    // command that stops to ask a question gets no answer and ends
    command.args(arguments).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).process_group(0);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            outcome.problem = Some(format!("{program}: {error}"));
            return outcome;
        }
    };
    let stdout = listen(child.stdout.take().unwrap(), limit);
    let stderr = listen(child.stderr.take().unwrap(), limit);
    let deadline = Instant::now() + timeout;
    let mut pause = Duration::from_millis(2);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                outcome.exit = status.code();
                break;
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(pause);
                pause = (pause * 2).min(Duration::from_millis(50));
            }
            _ => {
                outcome.timed_out = true;
                // SAFETY: kill only takes two numbers; the group is the child's own, made
                // above, and the child has not been waited for, so its number is still its
                unsafe { kill(-(child.id() as i32), SIGKILL) };
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
        }
    }
    // something the command left behind may still hold an output open: it is not waited for
    let patience = Duration::from_secs(1);
    for (output, kept) in [(stdout, &mut outcome.stdout), (stderr, &mut outcome.stderr)] {
        if let Ok((bytes, truncated)) = output.recv_timeout(patience) {
            *kept = String::from_utf8_lossy(&bytes).into_owned();
            outcome.truncated |= truncated;
        }
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| word.to_string()).collect()
    }

    #[test]
    fn what_a_command_writes_and_how_it_ends_are_kept() {
        let outcome = run(&line(&["/usr/bin/printf", "%s\\n", "one", "two"]), Duration::from_secs(5), LIMIT);
        assert_eq!((outcome.exit, outcome.stdout.as_str(), outcome.timed_out), (Some(0), "one\ntwo\n", false));
        assert_eq!(run(&line(&["/usr/bin/false"]), Duration::from_secs(5), LIMIT).exit, Some(1));
    }

    #[test]
    fn no_shell_reads_an_argument() {
        // a shell would run rm and expand $HOME; printf gets the one argument, as written
        let outcome = run(&line(&["/usr/bin/printf", "%s", "x; rm -rf $HOME `id` $(id) | cat > /tmp/x"]), Duration::from_secs(5), LIMIT);
        assert_eq!(outcome.stdout, "x; rm -rf $HOME `id` $(id) | cat > /tmp/x");
    }

    #[test]
    fn a_command_that_takes_too_long_is_stopped() {
        let started = Instant::now();
        let outcome = run(&line(&["/usr/bin/sleep", "30"]), Duration::from_millis(100), LIMIT);
        assert!(outcome.timed_out && outcome.exit.is_none());
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn only_the_first_of_a_long_output_is_kept() {
        let outcome = run(&line(&["/usr/bin/seq", "1", "100000"]), Duration::from_secs(10), 1000);
        assert_eq!((outcome.exit, outcome.stdout.len(), outcome.truncated), (Some(0), 1000, true));
    }

    #[test]
    fn a_command_that_waits_for_an_answer_gets_none() {
        let outcome = run(&line(&["/usr/bin/cat"]), Duration::from_secs(5), LIMIT);
        assert_eq!((outcome.exit, outcome.timed_out), (Some(0), false));
    }

    #[test]
    fn a_program_that_is_not_there_is_a_problem_not_a_crash() {
        let outcome = run(&line(&["/usr/bin/kavat-no-such-program"]), Duration::from_secs(5), LIMIT);
        assert!(outcome.problem.is_some() && outcome.exit.is_none());
    }
}
