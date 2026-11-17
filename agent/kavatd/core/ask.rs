//! kavat-ask: the terminal's way to the daemon. What `kavat ask` will run.
//!
//! In this skeleton it only carries requests to kavatd and shows the answers. It decides
//! nothing: the tools, their checks and their levels are the daemon's.

use kavatd::serve;
use serde_json::{Map, Value, json};
use std::io::{BufRead, IsTerminal, Write};
use std::os::unix::net::UnixStream;

const USAGE: &str = "\
usage: kavat-ask tools [words]            the tools, or the ones that fit some words
       kavat-ask call <tool> [name=value…]  run a tool; one that changes the system shows
                                          its command first and waits for a yes
       kavat-ask log [number]             the last calls, 20 unless a number says otherwise
       kavat-ask \"<what you want>\"        in your own words: for the model, which is not built yet";

const CLIENT: &str = "kavat-ask";

fn text(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}

/// A command line as a person would type it, for showing only: it is never run from this.
fn shown(command: &Value) -> String {
    let quoted = |argument: &Value| {
        let argument = text(argument);
        let plain = !argument.is_empty() && argument.chars().all(|c| c.is_ascii_alphanumeric() || "-_./=@%+:,".contains(c));
        if plain { argument.to_string() } else { format!("'{}'", argument.replace('\'', "'\\''")) }
    };
    command.as_array().map(|arguments| arguments.iter().map(quoted).collect::<Vec<_>>().join(" ")).unwrap_or_default()
}

/// The answer when the daemon said no; the answer itself when it said yes.
fn accepted(answer: Value) -> Result<Value, String> {
    if answer["ok"] == true { Ok(answer) } else { Err(text(&answer["error"]).to_string()) }
}

/// Arguments as they are typed after `kavat-ask call`.
fn pairs(arguments: &Value) -> String {
    let pair = |(name, value): (&String, &Value)| format!("{name}={}", value.as_str().map_or(value.to_string(), str::to_string));
    arguments.as_object().into_iter().flatten().map(pair).collect::<Vec<_>>().join(" ")
}

/// How a call is undone, when its tool says how.
fn way_back(undo: &Value) -> Option<String> {
    match (undo["tool"].as_str(), undo["say"].as_str()) {
        (Some(tool), _) => Some(format!("to undo it: kavat-ask call {tool} {}", pairs(&undo["args"]))),
        (_, Some(words)) => Some(format!("to undo it: {words}")),
        _ => None,
    }
}

fn tools(stream: &mut UnixStream, words: &[String]) -> Result<(), String> {
    let answer = accepted(serve::ask(stream, &json!({ "op": "tools", "words": words.join(" ") }))?)?;
    let tools = answer["tools"].as_array().cloned().unwrap_or_default();
    if tools.is_empty() {
        return Err("there is no tool for that".into());
    }
    for tool in tools {
        println!("{}  ({})\n    {}", text(&tool["name"]), text(&tool["level"]), text(&tool["description"]));
        let required = tool["parameters"]["required"].as_array().cloned().unwrap_or_default();
        for (name, param) in tool["parameters"]["properties"].as_object().into_iter().flatten() {
            let accepts = match (param["enum"].as_array(), param["minimum"].as_i64(), param["maximum"].as_i64()) {
                (Some(values), _, _) => values.iter().map(text).collect::<Vec<_>>().join(", "),
                (_, Some(min), Some(max)) => format!("{min} to {max}"),
                _ if param["type"] == "boolean" => "true or false".into(),
                _ => "text".into(),
            };
            let optional = if required.iter().any(|required| required == name) { "" } else { ", optional" };
            println!("      {name}: {} ({accepts}{optional})", text(&param["description"]));
        }
    }
    Ok(())
}

/// Shows how a call ended; its exit status is this program's.
fn ended(answer: &Value) -> i32 {
    // what the tool wrote is shown as it came, and nothing in it is acted on
    print!("{}", text(&answer["output"]["stdout"]));
    eprint!("{}", text(&answer["output"]["stderr"]));
    if answer["output"]["truncated"] == true {
        eprintln!("kavat-ask: it wrote more than this; only the first of it is kept");
    }
    if answer["timed_out"] == true {
        eprintln!("kavat-ask: {} took too long and was stopped", text(&answer["tool"]));
        return 124;
    }
    if let Some(way) = way_back(&answer["undo"]) {
        eprintln!("kavat-ask: {way}");
    }
    answer["exit"].as_i64().map_or(1, |exit| exit as i32)
}

fn call(stream: &mut UnixStream, arguments: &[String]) -> Result<i32, String> {
    let Some((tool, pairs)) = arguments.split_first() else {
        return Err("call takes a tool and its arguments: kavat-ask call theme.set mode=dark".into());
    };
    let mut given = Map::new();
    for pair in pairs {
        let Some((name, value)) = pair.split_once('=') else {
            return Err(format!("{pair}: an argument is written name=value"));
        };
        given.insert(name.to_string(), value.into());
    }
    let answer = accepted(serve::ask(stream, &json!({ "op": "call", "tool": tool, "args": given, "client": CLIENT }))?)?;
    if answer["status"] != "confirm" {
        return Ok(ended(&answer));
    }
    // the daemon holds the call until the person has seen what would run and said yes
    eprintln!("{} ({}) would run:\n\n    {}\n\nIt waits for you because {}.", text(&answer["tool"]), text(&answer["level"]), shown(&answer["command"]), text(&answer["why"]));
    if !std::io::stdin().is_terminal() {
        return Err("nobody is at a terminal to say yes: nothing was run".into());
    }
    eprint!("Run it? [y/N] ");
    let _ = std::io::stderr().flush();
    let mut said = String::new();
    let _ = std::io::stdin().lock().read_line(&mut said);
    if !matches!(said.trim(), "y" | "Y" | "yes") {
        return Err("nothing was run".into());
    }
    let confirm = json!({ "op": "confirm", "token": answer["token"], "tool": answer["tool"], "args": answer["args"], "client": CLIENT });
    Ok(ended(&accepted(serve::ask(stream, &confirm)?)?))
}

fn log(stream: &mut UnixStream, count: Option<&String>) -> Result<(), String> {
    let count = match count {
        None => 20,
        Some(count) => count.parse::<usize>().map_err(|_| "log takes how many calls to show: kavat-ask log 50".to_string())?,
    };
    let answer = accepted(serve::ask(stream, &json!({ "op": "log", "n": count }))?)?;
    for entry in answer["entries"].as_array().into_iter().flatten() {
        let ended = match (text(&entry["result"]), entry["exit"].as_i64()) {
            ("ran", Some(0)) => "done".to_string(),
            ("ran", Some(exit)) => format!("failed ({exit})"),
            (other, _) => other.to_string(),
        };
        let confirmed = if entry["confirmed"] == true { ", confirmed" } else { "" };
        println!("{}  {}  {} {}  [{}{confirmed}]  {ended}", text(&entry["time"]), text(&entry["client"]), text(&entry["tool"]), pairs(&entry["args"]), text(&entry["level"]));
        if let Some(way) = way_back(&entry["undo"]) {
            println!("    {way}");
        }
    }
    Ok(())
}

fn run(arguments: &[String]) -> Result<i32, String> {
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    if matches!(words.as_slice(), [] | ["--help" | "-h" | "help"]) {
        println!("{USAGE}");
        return Ok(0);
    }
    let path = serve::socket_path()?;
    let mut stream = UnixStream::connect(&path).map_err(|error| format!("kavatd is not listening on {}: {error}", path.display()))?;
    match words.as_slice() {
        ["tools", ..] => tools(&mut stream, &arguments[1..]).map(|()| 0),
        ["call", ..] => call(&mut stream, &arguments[1..]),
        ["log"] | ["log", _] => log(&mut stream, arguments.get(1)).map(|()| 0),
        // anything else is what the person wants, in their own words
        _ => {
            let answer = accepted(serve::ask(&mut stream, &json!({ "op": "ask", "text": arguments.join(" "), "client": CLIENT }))?)?;
            Ok(ended(&answer))
        }
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(status) => std::process::exit(status),
        Err(problem) => {
            eprintln!("kavat-ask: {problem}");
            std::process::exit(1);
        }
    }
}
