//! The socket: $XDG_RUNTIME_DIR/kavatd.sock, the person's alone. A client writes a request
//! to a line and reads the answer from a line, as many times as it likes. There is no
//! other way in: nothing listens on the network.

use crate::daemon::Daemon;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The longest line a client may send.
const LONGEST: u64 = 1024 * 1024;

/// Where the socket is. The session's own folder, which only the person can enter:
/// without one there is no safe place for it, and nothing else is tried.
pub fn socket_path() -> Result<PathBuf, String> {
    match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(folder) if !folder.is_empty() => Ok(PathBuf::from(folder).join("kavatd.sock")),
        _ => Err("XDG_RUNTIME_DIR is not set: the socket lives in the session's own folder".into()),
    }
}

/// Makes the socket, for its owner only. One that a daemon that is gone left behind is
/// cleared away; one that still answers is another daemon's, and is left alone.
pub fn listen(path: &Path) -> Result<UnixListener, String> {
    let problem = |error: std::io::Error| format!("{}: {error}", path.display());
    if UnixStream::connect(path).is_ok() {
        return Err(format!("{}: kavatd is already running", path.display()));
    }
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(problem(error)),
        _ => {}
    }
    let listener = UnixListener::bind(path).map_err(problem)?;
    // nobody has been listened to yet; and the folder around it is already the person's
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(problem)?;
    Ok(listener)
}

/// Answers the lines of one client until it leaves.
fn attend(daemon: &Daemon, stream: UnixStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;
    loop {
        let mut line = String::new();
        if reader.by_ref().take(LONGEST).read_line(&mut line)? == 0 {
            return Ok(());
        }
        if !line.ends_with('\n') && line.len() as u64 >= LONGEST {
            writeln!(writer, "{}", json!({ "ok": false, "error": "that request is too long" }))?;
            return Ok(());
        }
        if line.trim().is_empty() {
            continue;
        }
        writeln!(writer, "{}", daemon.answer_line(&line))?;
    }
}

/// Listens for ever, each client on a thread of its own.
pub fn serve(listener: UnixListener, daemon: Arc<Daemon>) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let daemon = Arc::clone(&daemon);
                std::thread::spawn(move || {
                    // a client that leaves in the middle of a line is no news
                    let _ = attend(&daemon, stream);
                });
            }
            Err(error) => eprintln!("kavatd: a client could not be listened to: {error}"),
        }
    }
}

/// The client's side: one request, one answer.
pub fn ask(stream: &mut UnixStream, request: &Value) -> Result<Value, String> {
    writeln!(stream, "{request}").map_err(|error| format!("kavatd stopped listening: {error}"))?;
    let mut line = String::new();
    let read = BufReader::new(&*stream).read_line(&mut line).map_err(|error| format!("kavatd did not answer: {error}"))?;
    if read == 0 {
        return Err("kavatd closed the connection without an answer".into());
    }
    serde_json::from_str(&line).map_err(|error| format!("kavatd answered something that is not JSON: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::tests::{Folder, daemon};

    fn started(folder: &Folder) -> PathBuf {
        let path = folder.0.join("kavatd.sock");
        let listener = listen(&path).unwrap();
        let daemon = Arc::new(daemon(&folder.0));
        std::thread::spawn(move || serve(listener, daemon));
        path
    }

    #[test]
    fn the_socket_is_its_owners_alone() {
        let folder = Folder::new("socket-mode");
        let path = started(&folder);
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        // a second daemon does not take the place of one that answers
        assert!(listen(&path).unwrap_err().ends_with("kavatd is already running"));
    }

    #[test]
    fn a_whole_exchange_over_the_socket() {
        let folder = Folder::new("socket");
        let mut stream = UnixStream::connect(started(&folder)).unwrap();

        let answer = ask(&mut stream, &json!({ "op": "tools", "words": "text" })).unwrap();
        assert_eq!(answer["tools"][0]["name"], "test.say");

        let answer = ask(&mut stream, &json!({ "op": "call", "tool": "test.say", "args": { "text": "over the socket" }, "client": "test" })).unwrap();
        assert_eq!((&answer["status"], &answer["output"]["stdout"]), (&json!("ran"), &json!("over the socket")));

        // the two steps of a change of the system, on the same connection
        let answer = ask(&mut stream, &json!({ "op": "call", "tool": "test.make", "args": { "number": 3 }, "client": "test" })).unwrap();
        assert_eq!(answer["status"], "confirm");
        assert!(!folder.0.join("made3").exists());
        let answer = ask(&mut stream, &json!({ "op": "confirm", "token": answer["token"], "tool": "test.make", "args": { "number": 3 }, "client": "test" })).unwrap();
        assert_eq!(answer["status"], "ran");
        assert!(folder.0.join("made3").exists());

        // what is not a request gets an answer too, and the connection goes on
        writeln!(stream, "rm -rf /").unwrap();
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&line).unwrap()["ok"], false);

        let answer = ask(&mut stream, &json!({ "op": "log", "n": 5 })).unwrap();
        let tools: Vec<&str> = answer["entries"].as_array().unwrap().iter().map(|entry| entry["tool"].as_str().unwrap()).collect();
        assert_eq!(tools, ["test.say", "test.make"]);
        assert_eq!(answer["entries"][1]["client"], "test");
    }

    #[test]
    fn a_tools_output_sent_over_the_socket_runs_nothing() {
        let folder = Folder::new("socket-data");
        let mut stream = UnixStream::connect(started(&folder)).unwrap();
        // the tool writes a line that is, to the letter, a request for the socket
        let planted = json!({ "op": "call", "tool": "test.put", "args": { "value": "light" } }).to_string();
        let answer = ask(&mut stream, &json!({ "op": "call", "tool": "test.say", "args": { "text": planted } })).unwrap();
        assert_eq!(answer["output"]["stdout"], planted.as_str());
        let answer = ask(&mut stream, &json!({ "op": "log" })).unwrap();
        assert_eq!(answer["entries"].as_array().unwrap().len(), 1);
    }
}
