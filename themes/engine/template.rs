//! Templates: an app's file with `{{ token }}` where the theme's values go.
//!
//! A template is a folder with a `template.toml` that says which files to fill in, where
//! each one goes and how the app is told to read it again. Supporting a new app is adding
//! a folder; no theme changes.

use crate::theme::{Token, Tokens};
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item};

/// Fills in a template. A token the theme does not have is an error, not an empty space.
///
/// `{{ accent }}` gives `#f2a93b`; `{{ accent | hex }}` gives `f2a93b`,
/// `{{ accent | rgb }}` gives `242, 169, 59` and `{{ accent | parts }}` gives each of the
/// three as a part of one, `0.949, 0.663, 0.231`. Numbers, flags and texts go in as they are.
pub fn render(text: &str, tokens: &Tokens) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let line = text[..text.len() - rest.len() + start].matches('\n').count() + 1;
        let after = &rest[start + 2..];
        let end = after.find("}}").ok_or_else(|| format!("line {line}: {{{{ is never closed"))?;
        let mut parts = after[..end].split('|').map(str::trim);
        let name = parts.next().unwrap_or("");
        let filter = parts.next();
        let token = tokens.get(name).ok_or_else(|| format!("line {line}: there is no token called {name}"))?;
        out.push_str(&match (token, filter) {
            (Token::Color(colour), None) => colour.to_string(),
            (Token::Color(colour), Some("hex")) => colour.hex(),
            (Token::Color(colour), Some("rgb")) => format!("{}, {}, {}", colour.r, colour.g, colour.b),
            (Token::Color(colour), Some("parts")) => {
                let part = |value: u8| f64::from(value) / 255.0;
                format!("{:.3}, {:.3}, {:.3}", part(colour.r), part(colour.g), part(colour.b))
            }
            (Token::Number(number), None) => format!("{number}"),
            (Token::Flag(flag), None) => format!("{flag}"),
            (Token::Text(text), None) => text.clone(),
            (_, Some(filter)) => return Err(format!("line {line}: {name} cannot be written as {filter}")),
        });
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

/// One file of a template.
pub struct FileRule {
    /// the file to fill in, inside the template's folder
    pub from: PathBuf,
    /// where the result goes, inside the generated folder
    pub to: String,
    /// where the app looks for it, when the app cannot be pointed at the generated folder:
    /// a link is left there. Starts with `config:`, the user's configuration folder.
    pub link: Option<String>,
}

pub struct Template {
    pub name: String,
    pub folder: PathBuf,
    pub files: Vec<FileRule>,
    /// commands that tell the running app to read its file again; they may use tokens
    pub reload: Vec<Vec<String>>,
}

impl Template {
    pub fn load(folder: &Path) -> Result<Template, String> {
        let name = folder.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let manifest = folder.join("template.toml");
        let problem = |what: String| format!("{}: {what}", manifest.display());
        let text = std::fs::read_to_string(&manifest).map_err(|error| problem(error.to_string()))?;
        let document: DocumentMut = text.parse().map_err(|error| problem(format!("{error}")))?;

        let mut files = Vec::new();
        let rules = document.get("file").and_then(Item::as_array_of_tables);
        for rule in rules.into_iter().flatten() {
            let from = rule.get("from").and_then(Item::as_str).ok_or_else(|| problem("a [[file]] has no from".into()))?;
            let to = rule.get("to").and_then(Item::as_str).map_or(format!("{name}/{from}"), str::to_string);
            let link = rule.get("link").and_then(Item::as_str).map(str::to_string);
            for path in [from, &to] {
                if path.starts_with('/') || path.split('/').any(|part| part == ".." || part.is_empty()) {
                    return Err(problem(format!("{path} has to stay inside its folder")));
                }
            }
            if link.as_ref().is_some_and(|link| !link.starts_with("config:") || link.contains("..")) {
                return Err(problem("link has to start with config: and stay inside it".into()));
            }
            files.push(FileRule { from: folder.join(from), to, link });
        }
        if files.is_empty() {
            return Err(problem("it names no [[file]]".into()));
        }

        let mut reload = Vec::new();
        for command in document.get("reload").and_then(Item::as_array).into_iter().flatten() {
            let words: Option<Vec<String>> =
                command.as_array().map(|words| words.iter().filter_map(|word| word.as_str().map(str::to_string)).collect());
            match words {
                Some(words) if !words.is_empty() => reload.push(words),
                _ => return Err(problem("reload is a list of commands, each a list of words".into())),
            }
        }
        Ok(Template { name, folder: folder.to_path_buf(), files, reload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    fn tokens() -> Tokens {
        let mut tokens = Tokens::new();
        tokens.insert("accent".into(), Token::Color(Color::parse("#f2a93b").unwrap()));
        tokens.insert("radius".into(), Token::Number(10.0));
        tokens.insert("opacity".into(), Token::Number(0.9));
        tokens.insert("blur".into(), Token::Flag(true));
        tokens.insert("font".into(), Token::Text("Inter".into()));
        tokens
    }

    #[test]
    fn fills_in_each_kind_of_token() {
        let text = "a {{accent}} b {{ accent | hex }} c {{accent|rgb}} d {{radius}} e {{opacity}} f {{blur}} g {{ font }}";
        assert_eq!(render(text, &tokens()).unwrap(), "a #f2a93b b f2a93b c 242, 169, 59 d 10 e 0.9 f true g Inter");
        assert_eq!(render("{{ accent | parts }}", &tokens()).unwrap(), "0.949, 0.663, 0.231");
    }

    #[test]
    fn text_without_tokens_comes_back_as_it_is() {
        assert_eq!(render("plain { text }\n", &tokens()).unwrap(), "plain { text }\n");
    }

    #[test]
    fn a_missing_token_is_an_error_with_its_line() {
        assert_eq!(render("one\ntwo {{ acent }}\n", &tokens()).unwrap_err(), "line 2: there is no token called acent");
        assert_eq!(render("{{ radius | hex }}", &tokens()).unwrap_err(), "line 1: radius cannot be written as hex");
        assert_eq!(render("{{ radius", &tokens()).unwrap_err(), "line 1: {{ is never closed");
    }
}
