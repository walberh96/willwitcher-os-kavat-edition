//! The tools: what each one is called, what it takes, its level and its command line.
//!
//! A tool is declared in a TOML file, one file to an area:
//!
//! ```toml
//! [[tool]]
//! name = "theme.set"
//! description = "Change the look of the system."      # what the model reads
//! level = "reversible"                 # read, reversible, with-snapshot or privileged
//! timeout = 20                         # seconds; 30 when it is not said
//! command = ["/usr/bin/kavat-theme", "set", ["mode", "{mode}"]]
//!
//! [tool.params.mode]
//! type = "enum"                        # string, integer, boolean or enum
//! description = "Dark or light."
//! values = ["dark", "light"]
//! optional = true
//! ```
//!
//! The command is a list of arguments, never a line for a shell. `{name}` is where a
//! parameter's value goes, and `{{` and `}}` are plain braces. An argument whose parameter
//! was not given is left out, and a list inside the list is left out whole when one of its
//! parameters was not given: that is how an option and its value go together. A string can
//! only be an argument by itself; only what has known values, an integer, a boolean or an
//! enum, can be a part of a longer one.
//!
//! A string has a `form`, text, words, name or path, and `max`, the longest it can be. An
//! integer has `min` and `max`. An enum has `values`. A boolean has `yes` and `no`: what
//! the command is given for true and for false.

use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

/// The permission levels of the design, from the least to the most.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    /// The state of the system. Runs at once.
    Read,
    /// What can be put back. Runs at once and stays in the log.
    Reversible,
    /// Packages and the system's configuration. One confirmation; a snapshot first.
    WithSnapshot,
    /// Disks, users, the network, services. A confirmation with the password.
    Privileged,
}

impl Level {
    pub fn word(self) -> &'static str {
        match self {
            Level::Read => "read",
            Level::Reversible => "reversible",
            Level::WithSnapshot => "with-snapshot",
            Level::Privileged => "privileged",
        }
    }

    /// Whether a call at this level waits for the person to say yes.
    pub fn asks(self) -> bool {
        self >= Level::WithSnapshot
    }
}

/// What a string may hold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Form {
    /// Any text on one line.
    Text,
    /// Words, each one an argument of its own.
    Words,
    /// The name of a package, of a palette, of a mode: letters, digits and @ . _ + -
    Name,
    /// A file, by its whole path.
    Path,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Text { form: Form, max: usize },
    Whole { min: i64, max: i64 },
    OneOf(Vec<String>),
    /// True or false, with what the command is given for each.
    Flag { yes: String, no: String },
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub description: String,
    pub kind: Kind,
    pub optional: bool,
}

#[derive(Debug, Clone)]
enum Part {
    Fixed(String),
    Hole(String),
}

/// One argument of the command line, with the places its parameters go.
#[derive(Debug, Clone)]
struct Argument(Vec<Part>);

#[derive(Debug, Clone)]
enum Slot {
    One(Argument),
    Group(Vec<Argument>),
}

/// How a call is undone.
#[derive(Debug, Clone)]
pub enum Undo {
    /// Words for the person: the command that goes back.
    Say(String),
    /// A call of a tool. Each of its arguments is what a command says before the change.
    Call { tool: String, before: BTreeMap<String, Vec<String>> },
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub level: Level,
    pub params: Vec<Param>,
    pub timeout: Duration,
    pub undo: Option<Undo>,
    command: Vec<Slot>,
}

/// The arguments of a call once checked: each a text, a whole number, or true or false.
pub type Arguments = BTreeMap<String, Value>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredFile {
    tool: Vec<Declared>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declared {
    name: String,
    description: String,
    level: Level,
    command: Vec<DeclaredSlot>,
    timeout: Option<u64>,
    #[serde(default)]
    params: BTreeMap<String, DeclaredParam>,
    undo: Option<DeclaredUndo>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DeclaredSlot {
    One(String),
    Group(Vec<String>),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredParam {
    #[serde(rename = "type")]
    kind: String,
    description: String,
    #[serde(default)]
    optional: bool,
    form: Option<String>,
    min: Option<i64>,
    max: Option<i64>,
    values: Option<Vec<String>>,
    yes: Option<String>,
    no: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredUndo {
    say: Option<String>,
    tool: Option<String>,
    #[serde(default)]
    before: BTreeMap<String, Vec<String>>,
}

const DEFAULT_TIMEOUT: u64 = 30;
const LONGEST_TIMEOUT: u64 = 3600;
const DEFAULT_LENGTH: usize = 200;

/// A name as the design writes them: an area, a dot and a verb, `theme.set`.
fn is_tool_name(name: &str) -> bool {
    let plain = |part: &str| !part.is_empty() && part.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    matches!(name.split_once('.'), Some((area, verb)) if plain(area) && plain(verb))
}

fn is_param_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// What no argument may be: empty, something a command would take for an option, or
/// more than one line.
fn safe_argument(text: &str) -> Result<(), &'static str> {
    if text.is_empty() {
        Err("cannot be empty")
    } else if text.starts_with('-') {
        Err("cannot start with -")
    } else if text.chars().any(char::is_control) {
        Err("cannot hold line breaks or control characters")
    } else {
        Ok(())
    }
}

fn argument(text: &str) -> Result<Argument, String> {
    let mut parts = Vec::new();
    let mut fixed = String::new();
    let mut letters = text.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '{' if letters.peek() == Some(&'{') => {
                letters.next();
                fixed.push('{');
            }
            '}' if letters.peek() == Some(&'}') => {
                letters.next();
                fixed.push('}');
            }
            '{' => {
                let name: String = letters.by_ref().take_while(|c| *c != '}').collect();
                if !is_param_name(&name) {
                    return Err(format!("\"{text}\": {{{name}}} is not a parameter; a plain brace is written twice"));
                }
                if !fixed.is_empty() {
                    parts.push(Part::Fixed(std::mem::take(&mut fixed)));
                }
                parts.push(Part::Hole(name));
            }
            '}' => return Err(format!("\"{text}\": a plain }} is written twice")),
            other => fixed.push(other),
        }
    }
    if !fixed.is_empty() {
        parts.push(Part::Fixed(fixed));
    }
    Ok(Argument(parts))
}

impl Argument {
    fn holes(&self) -> impl Iterator<Item = &str> {
        self.0.iter().filter_map(|part| match part {
            Part::Hole(name) => Some(name.as_str()),
            Part::Fixed(_) => None,
        })
    }

    fn alone(&self) -> bool {
        self.0.len() == 1
    }
}

impl Param {
    fn declared(tool: &str, name: &str, declared: DeclaredParam) -> Result<Param, String> {
        let problem = |what: &str| format!("{tool}: the parameter {name} {what}");
        if !is_param_name(name) {
            return Err(problem("has a name that is not lowercase letters, digits and _"));
        }
        let (ranged, listed, flagged) = (declared.min.is_some() || declared.max.is_some(), declared.values.is_some(), declared.yes.is_some() || declared.no.is_some());
        let kind = match declared.kind.as_str() {
            "string" => {
                if declared.min.is_some() || listed || flagged {
                    return Err(problem("is a string: it takes form and max, the longest it can be"));
                }
                let form = match declared.form.as_deref() {
                    None | Some("text") => Form::Text,
                    Some("words") => Form::Words,
                    Some("name") => Form::Name,
                    Some("path") => Form::Path,
                    Some(other) => return Err(problem(&format!("has the form {other}; the forms are text, words, name and path"))),
                };
                match declared.max {
                    Some(max) if max < 1 => return Err(problem("has a max below 1")),
                    max => Kind::Text { form, max: max.map_or(DEFAULT_LENGTH, |max| max as usize) },
                }
            }
            "integer" => match (declared.min, declared.max) {
                _ if declared.form.is_some() || listed || flagged => return Err(problem("is an integer: it takes min and max")),
                (Some(min), Some(max)) if min <= max => Kind::Whole { min, max },
                _ => return Err(problem("is an integer: it needs a min and a max, the range it accepts")),
            },
            "enum" => match declared.values {
                _ if declared.form.is_some() || ranged || flagged => return Err(problem("is an enum: it takes values")),
                Some(values) if !values.is_empty() => {
                    if let Some(bad) = values.iter().find(|value| safe_argument(value).is_err()) {
                        return Err(problem(&format!("has the value \"{bad}\", which cannot be an argument")));
                    }
                    Kind::OneOf(values)
                }
                _ => return Err(problem("is an enum: it needs its values")),
            },
            "boolean" => match (declared.yes, declared.no) {
                _ if declared.form.is_some() || ranged || listed => return Err(problem("is a boolean: it takes yes and no")),
                (Some(yes), Some(no)) if safe_argument(&yes).is_ok() && safe_argument(&no).is_ok() => Kind::Flag { yes, no },
                _ => return Err(problem("is a boolean: it needs yes and no, what the command is given for true and for false")),
            },
            other => return Err(problem(&format!("has the type {other}; the types are string, integer, boolean and enum"))),
        };
        Ok(Param { name: name.to_string(), description: declared.description, kind, optional: declared.optional })
    }

    fn accepts(&self) -> String {
        match &self.kind {
            Kind::Text { form: Form::Text, max } => format!("a text of up to {max} characters"),
            Kind::Text { form: Form::Words, max } => format!("words, up to {max} characters"),
            Kind::Text { form: Form::Name, max } => format!("a name of up to {max} letters, digits and @ . _ + -"),
            Kind::Text { form: Form::Path, max } => format!("the whole path of a file, starting with /, up to {max} characters"),
            Kind::Whole { min, max } => format!("a whole number from {min} to {max}"),
            Kind::OneOf(values) => values.join(", "),
            Kind::Flag { .. } => "true or false".into(),
        }
    }

    /// Checks one value and gives it back as the log and the command line will have it.
    fn check(&self, tool: &str, value: &Value) -> Result<Value, String> {
        let refused = || format!("{tool}: {} takes {}", self.name, self.accepts());
        let because = |why: &str| format!("{tool}: {} {why}", self.name);
        match &self.kind {
            Kind::Whole { min, max } => {
                // a small model often writes a number between quotes
                let number = match value {
                    Value::Number(number) => number.as_i64(),
                    Value::String(text) => text.parse().ok(),
                    _ => None,
                };
                number.filter(|number| number >= min && number <= max).map(Value::from).ok_or_else(refused)
            }
            Kind::OneOf(values) => match value {
                Value::String(text) if values.contains(text) => Ok(value.clone()),
                _ => Err(refused()),
            },
            Kind::Flag { .. } => match value {
                Value::Bool(_) => Ok(value.clone()),
                Value::String(text) if text == "true" => Ok(Value::Bool(true)),
                Value::String(text) if text == "false" => Ok(Value::Bool(false)),
                _ => Err(refused()),
            },
            Kind::Text { form, max } => {
                let Value::String(text) = value else { return Err(refused()) };
                if text.chars().count() > *max {
                    return Err(refused());
                }
                safe_argument(text).map_err(because)?;
                let fits = match form {
                    Form::Text => true,
                    Form::Words => text.split_whitespace().all(|word| !word.starts_with('-')),
                    Form::Name => text.chars().all(|c| c.is_ascii_alphanumeric() || "@._+-".contains(c)) && !text.starts_with('.'),
                    Form::Path => text.starts_with('/') && !text.split('/').any(|part| part == ".."),
                };
                if fits { Ok(value.clone()) } else { Err(refused()) }
            }
        }
    }

    /// The parameter as JSON Schema, which is what a model's tool call is described with.
    fn schema(&self) -> Value {
        let mut schema = match &self.kind {
            Kind::Text { max, .. } => json!({ "type": "string", "maxLength": max }),
            Kind::Whole { min, max } => json!({ "type": "integer", "minimum": min, "maximum": max }),
            Kind::OneOf(values) => json!({ "type": "string", "enum": values }),
            Kind::Flag { .. } => json!({ "type": "boolean" }),
        };
        schema["description"] = Value::from(self.description.as_str());
        schema
    }

    /// A checked value as the command is given it.
    fn written(&self, value: &Value) -> String {
        match (&self.kind, value) {
            (Kind::Flag { yes, .. }, Value::Bool(true)) => yes.clone(),
            (Kind::Flag { no, .. }, _) => no.clone(),
            (_, Value::String(text)) => text.clone(),
            (_, other) => other.to_string(),
        }
    }
}

impl Tool {
    fn declared(declared: Declared) -> Result<Tool, String> {
        let name = declared.name;
        if !is_tool_name(&name) {
            return Err(format!("{name}: a tool is named as an area, a dot and a verb, in lowercase: theme.set"));
        }
        if declared.description.trim().is_empty() {
            return Err(format!("{name}: the description is what the model chooses by; it cannot be empty"));
        }
        let timeout = declared.timeout.unwrap_or(DEFAULT_TIMEOUT);
        if timeout == 0 || timeout > LONGEST_TIMEOUT {
            return Err(format!("{name}: the timeout is from 1 to {LONGEST_TIMEOUT} seconds"));
        }
        let mut params = Vec::new();
        for (param, declared) in declared.params {
            params.push(Param::declared(&name, &param, declared)?);
        }
        let problem = |problem: String| format!("{name}: the command: {problem}");
        let mut command = Vec::new();
        for slot in declared.command {
            command.push(match slot {
                DeclaredSlot::One(text) => Slot::One(argument(&text).map_err(problem)?),
                DeclaredSlot::Group(texts) => Slot::Group(texts.iter().map(|text| argument(text)).collect::<Result<_, _>>().map_err(problem)?),
            });
        }
        // the program is a fixed file: nothing a call says can choose what runs
        match command.first() {
            Some(Slot::One(Argument(parts))) if matches!(parts.as_slice(), [Part::Fixed(program)] if program.starts_with('/')) => {}
            _ => return Err(format!("{name}: the command starts with the whole path of its program, like /usr/bin/kav")),
        }
        let arguments = || command.iter().flat_map(|slot| match slot {
            Slot::One(argument) => std::slice::from_ref(argument),
            Slot::Group(arguments) => arguments.as_slice(),
        });
        for argument in arguments() {
            for hole in argument.holes() {
                let Some(param) = params.iter().find(|param| param.name == hole) else {
                    return Err(format!("{name}: the command uses {{{hole}}}, which is not one of its parameters"));
                };
                // a text inside a longer argument could change what that argument means
                if matches!(param.kind, Kind::Text { .. }) && !argument.alone() {
                    return Err(format!("{name}: {{{hole}}} is a string, so it is an argument by itself; only an integer, a boolean or an enum can be part of a longer one"));
                }
            }
        }
        if let Some(unused) = params.iter().find(|param| !arguments().any(|argument| argument.holes().any(|hole| hole == param.name))) {
            return Err(format!("{name}: the parameter {} is not used by the command", unused.name));
        }
        let undo = match declared.undo {
            None => None,
            Some(DeclaredUndo { say: Some(say), tool: None, before }) if before.is_empty() => Some(Undo::Say(say)),
            Some(DeclaredUndo { say: None, tool: Some(tool), before }) if !before.is_empty() => {
                if before.values().any(|command| !command.first().is_some_and(|program| program.starts_with('/'))) {
                    return Err(format!("{name}: each command under undo.before starts with the whole path of its program"));
                }
                Some(Undo::Call { tool, before })
            }
            Some(_) => return Err(format!("{name}: undo is either say, the words that tell how, or tool with before, the commands that read what there was")),
        };
        Ok(Tool { name, description: declared.description, level: declared.level, params, timeout: Duration::from_secs(timeout), undo, command })
    }

    /// Checks the arguments of a call: every one known and of its type, none missing.
    pub fn check(&self, given: &Map<String, Value>) -> Result<Arguments, String> {
        let names = || self.params.iter().map(|param| param.name.as_str()).collect::<Vec<_>>().join(", ");
        if let Some(unknown) = given.keys().find(|key| !self.params.iter().any(|param| param.name == **key)) {
            return Err(match self.params.is_empty() {
                true => format!("{} takes no parameters; it was given {unknown}", self.name),
                false => format!("{} has no parameter called {unknown}; it takes: {}", self.name, names()),
            });
        }
        let mut checked = Arguments::new();
        for param in &self.params {
            match given.get(&param.name) {
                // a model leaves a null where it has nothing to say
                None | Some(Value::Null) if param.optional => {}
                None | Some(Value::Null) => return Err(format!("{} needs {}: {}", self.name, param.name, param.accepts())),
                Some(value) => {
                    checked.insert(param.name.clone(), param.check(&self.name, value)?);
                }
            }
        }
        Ok(checked)
    }

    /// The command line of a call, from arguments that `check` gave back. Each one goes
    /// to the program as it is: there is no shell that could read it again.
    pub fn command(&self, arguments: &Arguments) -> Vec<String> {
        let mut line = Vec::new();
        let given = |argument: &Argument| argument.holes().all(|hole| arguments.contains_key(hole));
        // every place in the command is a parameter the tool has: that was checked when
        // the tool was read
        let param = |name: &str| self.params.iter().find(|param| param.name == name).expect("a place for a parameter the tool has");
        let mut fill = |argument: &Argument| {
            if let [Part::Hole(name)] = argument.0.as_slice()
                && matches!(param(name).kind, Kind::Text { form: Form::Words, .. })
            {
                line.extend(param(name).written(&arguments[name]).split_whitespace().map(str::to_string));
                return;
            }
            line.push(argument.0.iter().map(|part| match part {
                Part::Fixed(text) => text.clone(),
                Part::Hole(name) => param(name).written(&arguments[name]),
            }).collect());
        };
        for slot in &self.command {
            match slot {
                Slot::One(argument) if given(argument) => fill(argument),
                Slot::Group(group) if group.iter().all(given) => group.iter().for_each(&mut fill),
                _ => {}
            }
        }
        line
    }

    /// What a client, and later the model, is told about the tool.
    pub fn describe(&self) -> Value {
        let properties: Map<String, Value> = self.params.iter().map(|param| (param.name.clone(), param.schema())).collect();
        let required: Vec<&str> = self.params.iter().filter(|param| !param.optional).map(|param| param.name.as_str()).collect();
        json!({
            "name": self.name,
            "description": self.description,
            "level": self.level.word(),
            "parameters": { "type": "object", "properties": properties, "required": required, "additionalProperties": false },
        })
    }
}

#[derive(Debug, Default)]
pub struct Registry {
    tools: BTreeMap<String, Tool>,
}

impl Registry {
    /// Adds the tools of one file. `origin` is the file's name, for the messages.
    pub fn add(&mut self, origin: &str, text: &str) -> Result<(), String> {
        let file: DeclaredFile = toml::from_str(text).map_err(|error| format!("{origin}: {}", error.to_string().trim_end()))?;
        for declared in file.tool {
            let tool = Tool::declared(declared).map_err(|problem| format!("{origin}: {problem}"))?;
            if self.tools.contains_key(&tool.name) {
                return Err(format!("{origin}: {} is declared twice", tool.name));
            }
            self.tools.insert(tool.name.clone(), tool);
        }
        Ok(())
    }

    /// What can only be checked with every tool known: that an undo names a tool, and
    /// arguments that tool has.
    pub fn finish(self) -> Result<Registry, String> {
        for tool in self.tools.values() {
            if let Some(Undo::Call { tool: target, before }) = &tool.undo {
                let Some(target) = self.tools.get(target) else {
                    return Err(format!("{}: its undo is the tool {target}, which does not exist", tool.name));
                };
                if let Some(unknown) = before.keys().find(|key| !target.params.iter().any(|param| param.name == **key)) {
                    return Err(format!("{}: its undo gives {} the argument {unknown}, which it does not take", tool.name, target.name));
                }
            }
        }
        Ok(self)
    }

    /// The tools of one text, as a test or a single file has them.
    pub fn parse(text: &str) -> Result<Registry, String> {
        let mut registry = Registry::default();
        registry.add("tools", text)?;
        registry.finish()
    }

    /// Every `.toml` file of a folder, in the order of their names. One file that is
    /// wrong stops them all: a tool is never half known.
    pub fn load(folder: &Path) -> Result<Registry, String> {
        let problem = |error: std::io::Error| format!("{}: {error}", folder.display());
        let mut files = Vec::new();
        for entry in std::fs::read_dir(folder).map_err(problem)? {
            let path = entry.map_err(problem)?.path();
            if path.extension().is_some_and(|extension| extension == "toml") {
                files.push(path);
            }
        }
        files.sort();
        let mut registry = Registry::default();
        for path in files {
            let text = std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
            registry.add(&path.display().to_string(), &text)?;
        }
        if registry.tools.is_empty() {
            return Err(format!("{}: there are no tools there", folder.display()));
        }
        registry.finish()
    }

    pub fn get(&self, name: &str) -> Result<&Tool, String> {
        self.tools.get(name).ok_or_else(|| format!("there is no tool called {name}"))
    }

    pub fn tools(&self) -> impl Iterator<Item = &Tool> {
        self.tools.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOLS: &str = r#"
        [[tool]]
        name = "test.echo"
        description = "Say a text back."
        level = "read"
        command = ["/usr/bin/echo", "{text}", ["--times", "{times}"], "kind={kind}"]

        [tool.params.text]
        type = "string"
        description = "What to say."
        max = 20

        [tool.params.times]
        type = "integer"
        description = "How many times."
        min = 1
        max = 5
        optional = true

        [tool.params.kind]
        type = "enum"
        description = "Loud or quiet."
        values = ["loud", "quiet"]
        optional = true
    "#;

    fn given(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).unwrap()
    }

    fn check(json: &str) -> Result<Arguments, String> {
        Registry::parse(TOOLS).unwrap().get("test.echo").unwrap().check(&given(json))
    }

    fn line(json: &str) -> Vec<String> {
        let registry = Registry::parse(TOOLS).unwrap();
        let tool = registry.get("test.echo").unwrap();
        tool.command(&tool.check(&given(json)).unwrap())
    }

    #[test]
    fn a_tool_that_does_not_exist_is_said_so() {
        let registry = Registry::parse(TOOLS).unwrap();
        assert_eq!(registry.get("test.shell").unwrap_err(), "there is no tool called test.shell");
    }

    #[test]
    fn a_missing_argument_is_refused() {
        assert_eq!(check("{}").unwrap_err(), "test.echo needs text: a text of up to 20 characters");
        assert!(check(r#"{"text": null}"#).is_err());
    }

    #[test]
    fn an_argument_the_tool_does_not_have_is_refused() {
        let problem = check(r#"{"text": "hi", "shell": "sh"}"#).unwrap_err();
        assert_eq!(problem, "test.echo has no parameter called shell; it takes: kind, text, times");
    }

    #[test]
    fn an_argument_of_another_type_is_refused() {
        assert!(check(r#"{"text": 7}"#).is_err());
        assert!(check(r#"{"text": ["hi"]}"#).is_err());
        assert!(check(r#"{"text": "hi", "times": "many"}"#).is_err());
        assert!(check(r#"{"text": "hi", "times": 1.5}"#).is_err());
        assert!(check(r#"{"text": "hi", "kind": 1}"#).is_err());
    }

    #[test]
    fn a_number_is_held_to_its_range() {
        assert_eq!(check(r#"{"text": "hi", "times": 0}"#).unwrap_err(), "test.echo: times takes a whole number from 1 to 5");
        assert!(check(r#"{"text": "hi", "times": 6}"#).is_err());
        assert_eq!(check(r#"{"text": "hi", "times": 5}"#).unwrap()["times"], 5);
        // between quotes, as a small model writes it, it is the same number
        assert_eq!(check(r#"{"text": "hi", "times": "3"}"#).unwrap()["times"], 3);
    }

    #[test]
    fn an_enum_takes_only_its_values() {
        assert_eq!(check(r#"{"text": "hi", "kind": "LOUD"}"#).unwrap_err(), "test.echo: kind takes loud, quiet");
        assert!(check(r#"{"text": "hi", "kind": "quiet"}"#).is_ok());
    }

    #[test]
    fn a_text_cannot_pass_for_an_option_or_run_over_a_line() {
        assert!(check(r#"{"text": "--help"}"#).is_err());
        assert!(check(r#"{"text": "one\ntwo"}"#).is_err());
        assert!(check(r#"{"text": ""}"#).is_err());
        assert!(check(r#"{"text": "longer than twenty characters"}"#).is_err());
    }

    #[test]
    fn what_was_not_given_is_left_out_of_the_command() {
        assert_eq!(line(r#"{"text": "hi"}"#), ["/usr/bin/echo", "hi"]);
        assert_eq!(line(r#"{"text": "hi", "times": 2, "kind": "loud"}"#), ["/usr/bin/echo", "hi", "--times", "2", "kind=loud"]);
    }

    #[test]
    fn a_text_reaches_the_command_as_one_argument() {
        assert_eq!(line(r#"{"text": "a; rm -rf $HOME"}"#), ["/usr/bin/echo", "a; rm -rf $HOME"]);
    }

    fn one(command: &str, param: &str) -> Result<Registry, String> {
        Registry::parse(&format!("[[tool]]\nname = \"test.one\"\ndescription = \"One.\"\nlevel = \"read\"\ncommand = {command}\n{param}"))
    }

    const TEXT: &str = "[tool.params.text]\ntype = \"string\"\ndescription = \"A text.\"";

    #[test]
    fn a_string_cannot_be_part_of_a_longer_argument() {
        assert!(one(r#"["/usr/bin/echo", "{text}"]"#, TEXT).is_ok());
        let problem = one(r#"["/usr/bin/sh", "-c", "echo {text}"]"#, TEXT).unwrap_err();
        assert!(problem.contains("is a string, so it is an argument by itself"), "{problem}");
    }

    #[test]
    fn the_program_is_a_fixed_file() {
        assert!(one(r#"["echo", "{text}"]"#, TEXT).is_err());
        assert!(one(r#"["{text}"]"#, TEXT).is_err());
        assert!(one("[]", "").is_err());
    }

    #[test]
    fn a_declaration_that_does_not_add_up_is_refused() {
        // a place for a parameter there is not, and a parameter with no place
        assert!(one(r#"["/usr/bin/echo", "{other}"]"#, TEXT).is_err());
        assert!(one(r#"["/usr/bin/echo"]"#, TEXT).is_err());
        // an integer with no range, a level there is not, a key nobody reads
        assert!(one(r#"["/usr/bin/echo", "{n}"]"#, "[tool.params.n]\ntype = \"integer\"\ndescription = \"A number.\"").is_err());
        assert!(Registry::parse("[[tool]]\nname = \"test.one\"\ndescription = \"One.\"\nlevel = \"root\"\ncommand = [\"/usr/bin/true\"]").is_err());
        assert!(one(r#"["/usr/bin/true"]"#, "shell = true").is_err());
        // a name that is not an area and a verb
        assert!(Registry::parse("[[tool]]\nname = \"echo\"\ndescription = \"One.\"\nlevel = \"read\"\ncommand = [\"/usr/bin/true\"]").is_err());
    }

    #[test]
    fn braces_written_twice_are_plain_braces() {
        let registry = one(r#"["/usr/bin/echo", "focus({{ workspace = {n} }})"]"#, "[tool.params.n]\ntype = \"integer\"\ndescription = \"A number.\"\nmin = 1\nmax = 10").unwrap();
        let tool = registry.get("test.one").unwrap();
        assert_eq!(tool.command(&tool.check(&given(r#"{"n": 3}"#)).unwrap()), ["/usr/bin/echo", "focus({ workspace = 3 })"]);
    }

    #[test]
    fn true_and_false_are_what_the_tool_says_they_are() {
        let flag = "[tool.params.muted]\ntype = \"boolean\"\ndescription = \"Muted or not.\"\nyes = \"1\"\nno = \"0\"";
        let registry = one(r#"["/usr/bin/echo", "{muted}"]"#, flag).unwrap();
        let tool = registry.get("test.one").unwrap();
        let line = |json: &str| tool.check(&given(json)).map(|arguments| tool.command(&arguments));
        assert_eq!(line(r#"{"muted": true}"#).unwrap(), ["/usr/bin/echo", "1"]);
        assert_eq!(line(r#"{"muted": "false"}"#).unwrap(), ["/usr/bin/echo", "0"]);
        assert!(line(r#"{"muted": "yes"}"#).is_err());
        assert!(line(r#"{"muted": 1}"#).is_err());
        assert!(one(r#"["/usr/bin/echo", "{muted}"]"#, "[tool.params.muted]\ntype = \"boolean\"\ndescription = \"Muted or not.\"").is_err());
    }

    #[test]
    fn words_are_an_argument_each() {
        let registry = one(r#"["/usr/bin/echo", "{text}"]"#, &format!("{TEXT}\nform = \"words\"")).unwrap();
        let tool = registry.get("test.one").unwrap();
        assert_eq!(tool.command(&tool.check(&given(r#"{"text": "video  editor"}"#)).unwrap()), ["/usr/bin/echo", "video", "editor"]);
        assert!(tool.check(&given(r#"{"text": "video --help"}"#)).is_err());
    }

    #[test]
    fn a_name_and_a_path_are_held_to_their_form() {
        let name = one(r#"["/usr/bin/echo", "{text}"]"#, &format!("{TEXT}\nform = \"name\"")).unwrap();
        let name = name.get("test.one").unwrap();
        assert!(name.check(&given(r#"{"text": "gtk4-layer-shell"}"#)).is_ok());
        assert!(name.check(&given(r#"{"text": "vim; reboot"}"#)).is_err());
        let path = one(r#"["/usr/bin/echo", "{text}"]"#, &format!("{TEXT}\nform = \"path\"")).unwrap();
        let path = path.get("test.one").unwrap();
        assert!(path.check(&given(r#"{"text": "/home/me/Pictures/a b.png"}"#)).is_ok());
        assert!(path.check(&given(r#"{"text": "a.png"}"#)).is_err());
        assert!(path.check(&given(r#"{"text": "/home/me/../../etc/shadow"}"#)).is_err());
    }

    #[test]
    fn an_undo_names_a_tool_that_exists() {
        let undo = "[tool.undo]\ntool = \"test.other\"\nbefore = { text = [\"/usr/bin/true\"] }";
        assert!(one(r#"["/usr/bin/echo", "{text}"]"#, &format!("{undo}\n{TEXT}")).unwrap_err().contains("which does not exist"));
    }

    /// The tools the system comes with, in tools/ beside this crate.
    #[test]
    fn the_systems_own_tools_are_read_and_make_the_commands_they_should() {
        let registry = Registry::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tools")).unwrap();
        let line = |name: &str, json: &str| {
            let tool = registry.get(name).unwrap();
            tool.check(&given(json)).map(|arguments| tool.command(&arguments).join(" "))
        };
        assert_eq!(line("packages.search", r#"{"query": "video editor"}"#).unwrap(), "/usr/bin/kav search video editor");
        assert_eq!(line("packages.list", r#"{"source": "aur"}"#).unwrap(), "/usr/bin/kav list --aur");
        assert_eq!(line("packages.install", r#"{"name": "obs-studio"}"#).unwrap(), "/usr/bin/kav install obs-studio");
        assert!(line("packages.install", r#"{"name": "--noconfirm"}"#).is_err());
        assert!(line("packages.install", r#"{"name": "vim neovim"}"#).is_err());
        assert_eq!(line("theme.get", "{}").unwrap(), "/usr/bin/kavat-theme get");
        assert_eq!(line("theme.set", r#"{"mode": "light"}"#).unwrap(), "/usr/bin/kavat-theme set mode light");
        assert_eq!(line("theme.set", r#"{"palette": "nord", "accent": "frost"}"#).unwrap(), "/usr/bin/kavat-theme set palette nord accent frost");
        assert_eq!(line("workspaces.switch", r#"{"workspace": 3}"#).unwrap(), "/usr/bin/hyprctl dispatch hl.dsp.focus({ workspace = 3 })");
        assert!(line("workspaces.switch", r#"{"workspace": "3 }) os.execute('id') --"}"#).is_err());
        assert_eq!(line("mode.set", r#"{"mode": "gamer"}"#).unwrap(), "/usr/bin/kavat-mode gamer");
        assert_eq!(line("wallpaper.set", r#"{"file": "/home/me/Pictures/Wallpapers/images/sea.png"}"#).unwrap(), "/usr/bin/kavat-wallpaper set /home/me/Pictures/Wallpapers/images/sea.png");
        assert_eq!(line("volume.set", r#"{"level": 40}"#).unwrap(), "/usr/bin/wpctl set-volume @DEFAULT_AUDIO_SINK@ 40%");
        assert_eq!(line("volume.mute", r#"{"muted": true}"#).unwrap(), "/usr/bin/wpctl set-mute @DEFAULT_AUDIO_SINK@ 1");
        assert_eq!(line("notes.add", r#"{"text": "milk", "notebook": "home"}"#).unwrap(), "/usr/bin/kavat-notes add --to home milk");
        assert_eq!(line("notes.due", "{}").unwrap(), "/usr/bin/kavat-notes due");
        // the design's table: what reads and what can be put back runs at once, packages wait
        let level = |name: &str| registry.get(name).unwrap().level;
        assert_eq!((level("packages.search"), level("theme.set"), level("packages.install"), level("packages.remove")), (Level::Read, Level::Reversible, Level::WithSnapshot, Level::WithSnapshot));
        assert!(registry.tools().all(|tool| tool.level != Level::Privileged));
    }

    #[test]
    fn the_description_is_a_schema_a_model_can_read() {
        let registry = Registry::parse(TOOLS).unwrap();
        let described = registry.get("test.echo").unwrap().describe();
        assert_eq!(described["level"], "read");
        assert_eq!(described["parameters"]["required"], json!(["text"]));
        assert_eq!(described["parameters"]["properties"]["times"], json!({ "type": "integer", "minimum": 1, "maximum": 5, "description": "How many times." }));
        assert_eq!(described["parameters"]["properties"]["kind"]["enum"], json!(["loud", "quiet"]));
    }
}
