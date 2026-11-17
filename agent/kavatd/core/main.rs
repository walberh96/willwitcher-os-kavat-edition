//! kavatd: the daemon that runs the agent's tools.

use kavatd::daemon::Daemon;
use kavatd::log::Log;
use kavatd::registry::Registry;
use kavatd::serve;
use std::path::PathBuf;
use std::sync::Arc;

const USAGE: &str = "\
usage: kavatd [--tools <folder>]           listen on $XDG_RUNTIME_DIR/kavatd.sock
       kavatd --check [--tools <folder>]   read the tools and list them, or say what is wrong

The tools are the .toml files of /usr/share/kavat/agent/tools. Every call is written to
~/.local/state/kavat/agent/log.jsonl. kavat-ask is how the daemon is spoken to.";

const TOOLS: &str = "/usr/share/kavat/agent/tools";

fn run(arguments: &[&str]) -> Result<(), String> {
    let (check, folder) = match arguments {
        [] => (false, TOOLS),
        ["--tools", folder] => (false, *folder),
        ["--check"] => (true, TOOLS),
        ["--check", "--tools", folder] | ["--tools", folder, "--check"] => (true, *folder),
        ["--help" | "-h"] => {
            println!("{USAGE}");
            return Ok(());
        }
        _ => return Err(format!("that is not something kavatd takes\n\n{USAGE}")),
    };
    let registry = Registry::load(&PathBuf::from(folder))?;
    if check {
        for tool in registry.tools() {
            println!("{:<18} {:<14} {}", tool.name, tool.level.word(), tool.description);
        }
        return Ok(());
    }
    let log = Log::at(Log::default_path()?);
    log.ready()?;
    let path = serve::socket_path()?;
    let listener = serve::listen(&path)?;
    eprintln!("kavatd: {} tools, listening on {}", registry.tools().count(), path.display());
    serve::serve(listener, Arc::new(Daemon::new(registry, log)));
    Ok(())
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if let Err(problem) = run(&arguments) {
        eprintln!("kavatd: {problem}");
        std::process::exit(1);
    }
}
