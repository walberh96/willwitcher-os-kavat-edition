//! kav's own settings, in /etc/kav/kav.toml: the few that no other tool has a file for.
//!
//!   cache = 3              # versions of each package kept in pacman's cache
//!   jobs  = 0              # processors a build from the AUR uses; 0 is all of them
//!   hold  = ["some-tool"]  # packages of the AUR that kav update leaves as they are

use crate::allowlist::is_name;
use std::path::Path;
use toml_edit::{DocumentMut, Item};

#[derive(Debug, PartialEq)]
pub struct Config {
    pub cache: u32,
    pub jobs: u32,
    pub hold: Vec<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config { cache: 3, jobs: 0, hold: Vec::new() }
    }
}

impl Config {
    pub fn parse(origin: &str, text: &str) -> Result<Config, String> {
        let problem = |what: String| format!("{origin}: {what}");
        let document: DocumentMut = text.parse().map_err(|error| problem(format!("{error}")))?;
        let mut config = Config::default();
        for (key, item) in document.iter() {
            let number = || item.as_integer().and_then(|value| u32::try_from(value).ok()).ok_or_else(|| problem(format!("{key} is a number")));
            match key {
                "cache" => config.cache = number()?,
                "jobs" => config.jobs = number()?,
                "hold" => {
                    for name in item.as_array().ok_or_else(|| problem("hold is a list of packages".into()))? {
                        match name.as_str().filter(|name| is_name(name)) {
                            Some(name) => config.hold.push(name.to_string()),
                            None => return Err(problem("hold is a list of the names of packages".into())),
                        }
                    }
                }
                _ => return Err(problem(format!("{key} is not something this file holds; it holds cache, jobs and hold"))),
            }
        }
        Ok(config)
    }

    /// What the file says; what it does not say, and all of it when there is no file, is
    /// as kav comes.
    pub fn load(file: &Path) -> Result<Config, String> {
        match std::fs::read_to_string(file) {
            Ok(text) => Config::parse(&file.display().to_string(), &text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(error) => Err(format!("{}: {error}", file.display())),
        }
    }
}

/// The file's text with one of its settings changed, and the rest as it was.
fn changed(origin: &str, text: &str, change: &dyn Fn(&mut DocumentMut)) -> Result<String, String> {
    // a file that cannot be read as settings is not written over
    Config::parse(origin, text)?;
    let mut document: DocumentMut = text.parse().map_err(|error| format!("{origin}: {error}"))?;
    change(&mut document);
    let text = document.to_string();
    Config::parse(origin, &text)?;
    Ok(text)
}

/// Changes one setting in the file, as root.
fn write(file: &Path, change: &dyn Fn(&mut DocumentMut)) -> Result<(), String> {
    let (opening, text) = match std::fs::read_to_string(file) {
        Ok(text) => ("", text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ("# kav's own settings on this machine. `kav set` writes them.\n\n", String::new()),
        Err(error) => return Err(format!("{}: {error}", file.display())),
    };
    let text = changed(&file.display().to_string(), &text, change)?;
    crate::system::write_as_root(&file.to_string_lossy(), &format!("{opening}{text}"))
}

pub fn set_number(file: &Path, key: &str, value: u32) -> Result<(), String> {
    write(file, &|document| document[key] = toml_edit::value(i64::from(value)))
}

/// Holds a package back, or lets it go.
pub fn set_held(file: &Path, name: &str, held: bool) -> Result<(), String> {
    write(file, &|document| {
        let mut names: Vec<String> = document.get("hold").and_then(Item::as_array).into_iter().flatten().filter_map(|name| name.as_str().map(str::to_string)).collect();
        names.retain(|other| other != name);
        if held {
            names.push(name.to_string());
        }
        names.sort();
        document["hold"] = toml_edit::value(names.iter().map(String::as_str).collect::<toml_edit::Array>());
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_the_file_does_not_say_is_as_kav_comes() {
        assert_eq!(Config::parse("kav.toml", "").unwrap(), Config { cache: 3, jobs: 0, hold: vec![] });
        let config = Config::parse("kav.toml", "cache = 1\nhold = [\"some-tool\"]\n").unwrap();
        assert_eq!((config.cache, config.jobs, config.hold), (1, 0, vec!["some-tool".to_string()]));
    }

    #[test]
    fn what_is_not_a_setting_is_an_error() {
        assert!(Config::parse("kav.toml", "cache = \"many\"\n").is_err());
        assert!(Config::parse("kav.toml", "cache = -1\n").is_err());
        assert!(Config::parse("kav.toml", "hold = [\"--overwrite\"]\n").is_err());
        assert!(Config::parse("kav.toml", "mirror = \"example.org\"\n").is_err());
    }

    #[test]
    fn a_change_keeps_the_rest_of_the_file() {
        let text = "# mine\ncache = 3   # three\nhold = [\"b-tool\"]\n";
        let more = changed("kav.toml", text, &|document| document["jobs"] = toml_edit::value(4)).unwrap();
        assert!(more.starts_with("# mine\ncache = 3   # three\n") && more.contains("jobs = 4"), "{more}");
        assert!(changed("kav.toml", "nonsense", &|_| {}).is_err());
    }
}
