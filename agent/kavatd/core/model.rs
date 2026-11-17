//! Where the models go. Nothing here is built yet: each function is the place of a piece
//! of phase 4, with what stands in for it today and what comes next.

use crate::registry::{Registry, Tool};
use serde_json::{Map, Value};

/// The most tools the model is shown at a time: the design says five to eight.
pub const SHOWN: usize = 8;

/// A call somebody chose: the tool and its arguments, still to be checked.
#[derive(Debug)]
pub struct Chosen {
    pub tool: String,
    pub arguments: Map<String, Value>,
}

/// STUB, the search model. The tools that fit some words, the best first, at most
/// `SHOWN`; every tool when there are no words.
///
/// Today a tool fits by the words of the request that are in its name, its description
/// or its parameters. Next: a small embedding model on the CPU, which finds a tool by
/// what a request means; the descriptions are embedded once when the tools are loaded,
/// and when nothing is near the list is widened before saying there is no tool for it.
pub fn retrieve<'a>(registry: &'a Registry, words: &str) -> Vec<&'a Tool> {
    let words: Vec<String> = words.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return registry.tools().collect();
    }
    let mut found: Vec<(usize, &Tool)> = registry
        .tools()
        .filter_map(|tool| {
            let mut text = format!("{} {}", tool.name, tool.description);
            tool.params.iter().for_each(|param| text.push_str(&format!(" {} {}", param.name, param.description)));
            let text = text.to_lowercase();
            let score = words.iter().filter(|word| text.contains(word.as_str())).count();
            (score > 0).then_some((score, tool))
        })
        .collect();
    // the sort keeps the order of the names among tools that fit the same
    found.sort_by_key(|one| std::cmp::Reverse(one.0));
    found.into_iter().take(SHOWN).map(|(_, tool)| tool).collect()
}

/// STUB, the skills' exact phrases. A request that is, word for word, the phrase of a
/// skill launches it without the model.
///
/// Today there are no skills, so nothing matches. Next: the skills are read from
/// /usr/share/kavat/agent/skills and ~/.config/kavat/skills, a skill being a list of
/// tool calls that the daemon runs one after another at the highest level among them,
/// and this compares the request with their phrases.
pub fn skill_for(_request: &str) -> Option<Chosen> {
    None
}

/// STUB, the language model. From what the person said in their own words to one tool
/// and its arguments.
///
/// Today it only says so. Next: the request and the descriptions of the tools that
/// `retrieve` gives go to the model Ollama serves on this machine, and what it answers
/// comes back as a `Chosen`, which is then checked and gated like any other call. What a
/// tool wrote may go back to the model to be put into words, and never comes back from
/// it as a call that runs by itself: a call the model makes after reading an output is
/// marked `from_reading`, and waits for the person.
pub fn choose(_request: &str, _tools: &[&Tool]) -> Result<Chosen, String> {
    Err("the model is not built yet: kavat-ask tools lists the tools, and kavat-ask call runs one".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOLS: &str = r#"
        [[tool]]
        name = "theme.get"
        description = "The look of the system: its palette and whether it is dark or light."
        level = "read"
        command = ["/usr/bin/true"]

        [[tool]]
        name = "volume.set"
        description = "Make the sound louder or quieter."
        level = "reversible"
        command = ["/usr/bin/true"]
    "#;

    #[test]
    fn tools_are_found_by_the_words_of_a_request() {
        let registry = Registry::parse(TOOLS).unwrap();
        let names = |words: &str| retrieve(&registry, words).iter().map(|tool| tool.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names(""), ["theme.get", "volume.set"]);
        assert_eq!(names("is it DARK"), ["theme.get"]);
        assert_eq!(names("louder sound"), ["volume.set"]);
        assert!(names("pizza").is_empty());
    }

    #[test]
    fn the_model_is_not_built_yet() {
        assert!(skill_for("open the browser").is_none());
        assert!(choose("make it dark", &[]).unwrap_err().starts_with("the model is not built yet"));
    }
}
