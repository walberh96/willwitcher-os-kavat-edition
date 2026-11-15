//! Firmware, as one more step of an update: fwupd is asked what there is for the
//! motherboard, the disks and what is plugged in, and offers it. fwupd is not part of the
//! system: when it is not installed, or cannot reach its servers, there is no such step.

use crate::system::{has, run, someone_there, yes};
use std::process::{Command, Stdio};

/// What fwupd answers with, read as far as kav needs it.
#[derive(Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(fields) => fields.iter().find(|(name, _)| name == key).map(|(_, value)| value),
            _ => None,
        }
    }

    fn text(&self) -> Option<&str> {
        match self {
            Json::Text(text) => Some(text),
            _ => None,
        }
    }

    fn list(&self) -> &[Json] {
        match self {
            Json::List(items) => items,
            _ => &[],
        }
    }
}

/// Reads one value at the start of the text, and gives it with what follows it.
fn value(text: &str) -> Option<(Json, &str)> {
    let text = text.trim_start();
    let first = text.chars().next()?;
    match first {
        '{' | '[' => {
            let (mut rest, mut fields, mut items) = (text[1..].trim_start(), Vec::new(), Vec::new());
            let close = if first == '{' { '}' } else { ']' };
            loop {
                if let Some(after) = rest.strip_prefix(close) {
                    return Some((if first == '{' { Json::Object(fields) } else { Json::List(items) }, after));
                }
                if first == '{' {
                    let (Json::Text(key), after) = value(rest)? else { return None };
                    let (item, after) = value(after.trim_start().strip_prefix(':')?)?;
                    fields.push((key, item));
                    rest = after.trim_start();
                } else {
                    let (item, after) = value(rest)?;
                    items.push(item);
                    rest = after.trim_start();
                }
                rest = match rest.strip_prefix(',') {
                    Some(after) => after.trim_start(),
                    None if rest.starts_with(close) => rest,
                    None => return None,
                };
            }
        }
        '"' => {
            let (mut out, mut letters) = (String::new(), text[1..].char_indices());
            while let Some((at, letter)) = letters.next() {
                match letter {
                    '"' => return Some((Json::Text(out), &text[at + 2..])),
                    '\\' => match letters.next()?.1 {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'u' => {
                            let code: String = letters.by_ref().take(4).map(|(_, digit)| digit).collect();
                            out.push(u32::from_str_radix(&code, 16).ok().and_then(char::from_u32).unwrap_or('?'));
                        }
                        'r' | 'b' | 'f' => {}
                        other => out.push(other),
                    },
                    _ => out.push(letter),
                }
            }
            None
        }
        _ => {
            let end = text.find(|c: char| !(c.is_ascii_alphanumeric() || "+-.".contains(c))).unwrap_or(text.len());
            let word = &text[..end];
            let found = match word {
                "null" => Json::Null,
                "true" => Json::Bool(true),
                "false" => Json::Bool(false),
                _ => Json::Number(word.parse().ok()?),
            };
            Some((found, &text[end..]))
        }
    }
}

/// One piece of firmware that can be brought up to date.
#[derive(Debug, PartialEq)]
pub struct Firmware {
    pub device: String,
    pub from: String,
    pub to: String,
    /// whether the machine restarts, or is switched off and on, to put it in
    pub restart: bool,
}

/// What `fwupdmgr get-updates --json` answered, as the firmware it offers.
pub fn offered(answer: &str) -> Vec<Firmware> {
    let Some((all, _)) = value(answer) else { return Vec::new() };
    let mut found = Vec::new();
    for device in all.get("Devices").map_or(&[][..], Json::list) {
        let text_of = |from: &Json, key: &str| from.get(key).and_then(Json::text).unwrap_or("").to_string();
        // the first release listed is the newest
        let Some(release) = device.get("Releases").and_then(|releases| releases.list().first()) else { continue };
        let flags = device.get("Flags").map_or(&[][..], Json::list);
        found.push(Firmware {
            device: text_of(device, "Name"),
            from: text_of(device, "Version"),
            to: text_of(release, "Version"),
            restart: flags.iter().filter_map(Json::text).any(|flag| flag == "needs-reboot" || flag == "needs-shutdown"),
        });
    }
    found
}

/// The step of kav update: names the firmware there is and, with a person at the
/// terminal, offers to put it in with fwupd's own command, which asks its own questions.
pub fn step() {
    if !has("fwupdmgr") {
        return;
    }
    // what fwupd knows is fetched again; without a network this fails, and so does the step
    let quiet = |arguments: &[&str]| Command::new("fwupdmgr").args(arguments).stdin(Stdio::null()).stderr(Stdio::null()).output().ok();
    let _ = quiet(&["refresh"]);
    // fwupd ends with an error when nothing is offered: what it wrote says which it was
    let Some(answer) = quiet(&["get-updates", "--json"]) else { return };
    let all = offered(&String::from_utf8_lossy(&answer.stdout));
    if all.is_empty() {
        return;
    }
    println!("\nkav: firmware that can be brought up to date:");
    for firmware in &all {
        println!("  {:<40} {} -> {}{}", firmware.device, firmware.from, firmware.to, if firmware.restart { "  · restarts the machine" } else { "" });
    }
    if !someone_there() {
        println!("kav: `fwupdmgr update` puts it in.");
        return;
    }
    if all.iter().any(|firmware| firmware.restart) {
        println!("Some of it is put in while the machine restarts: fwupd asks before it restarts.\nKeep the power plugged in until it is done.");
    }
    if yes("Put the firmware in now?") {
        if let Err(problem) = run(Command::new("fwupdmgr").arg("update")) {
            println!("kav: the firmware was not put in: {problem}. `fwupdmgr update` tries it again.");
        }
    } else {
        println!("kav: the firmware stays as it is. `fwupdmgr update` puts it in.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANSWER: &str = r#"{
  "Devices" : [
    {
      "Name" : "System Firmware",
      "DeviceId" : "a45df35ac0e948ee180fe216a5f703f32dda163f",
      "Plugin" : "uefi_capsule",
      "Flags" : [ "internal", "updatable", "require-ac", "needs-reboot", "usable-during-update" ],
      "Version" : "1.20",
      "VersionRaw" : 65556,
      "Releases" : [
        { "Version" : "1.24", "Description" : "<p>Fixes \"S3\" resume.\nAnd more é.</p>", "Size" : 16777216, "Flags" : [ "is-upgrade" ] },
        { "Version" : "1.22", "Size" : 16777216 }
      ]
    },
    {
      "Name" : "USB Receiver",
      "Flags" : [ "updatable" ],
      "Version" : "RQR24.07_B0030",
      "Releases" : [ { "Version" : "RQR24.11_B0036", "Trusted" : true, "Urgency" : null } ]
    },
    { "Name" : "A disk with nothing new", "Version" : "2B2QGXA7", "Releases" : [ ] }
  ]
}"#;

    #[test]
    fn what_fwupd_answers_becomes_the_firmware_it_offers() {
        let all = offered(ANSWER);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], Firmware { device: "System Firmware".into(), from: "1.20".into(), to: "1.24".into(), restart: true });
        assert_eq!((all[1].device.as_str(), all[1].to.as_str(), all[1].restart), ("USB Receiver", "RQR24.11_B0036", false));
    }

    #[test]
    fn an_answer_with_nothing_in_it_offers_nothing() {
        assert!(offered("{ \"Devices\" : [] }").is_empty());
        assert!(offered("").is_empty());
        assert!(offered("No updatable devices").is_empty());
        assert!(offered("{ \"Devices\" : [ { \"Name\" : ").is_empty());
    }

    #[test]
    fn text_is_read_with_its_escapes() {
        let (found, rest) = value(r#"{"a":["x\"y\\", -1.5e2, true, null], "b":{}} after"#).unwrap();
        assert_eq!(found.get("a").unwrap().list(), [Json::Text("x\"y\\".into()), Json::Number(-150.0), Json::Bool(true), Json::Null]);
        assert_eq!(found.get("b"), Some(&Json::Object(vec![])));
        assert_eq!(rest, " after");
    }
}
