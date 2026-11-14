//! Applying the theme: every file is made first, in a new folder, and then one link changes.
//!
//! If anything fails before that, the theme that was in place stays in place, whole.

use crate::template::{Template, render};
use crate::theme::{DEFAULTS, Palette, Settings, Tokens, resolve};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use toml_edit::DocumentMut;

/// Where things are. Each folder follows the environment, as every program's does.
pub struct Paths {
    /// the system's palettes and templates
    pub data: PathBuf,
    /// the settings of this machine, above the defaults and below the user's
    pub machine: PathBuf,
    /// the user's configuration folder, ~/.config
    pub config: PathBuf,
    /// where the generated files live, ~/.local/state/kavat/theme
    pub state: PathBuf,
    /// true for the screens before the desktop (the boot menu, the loading screen and the
    /// login): they belong to the machine, so nothing of a user's is read
    pub system: bool,
}

impl Paths {
    pub fn from_environment() -> Result<Paths, String> {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let folder = |variable: &str, fallback: &str| -> Result<PathBuf, String> {
            match std::env::var_os(variable).filter(|value| !value.is_empty()) {
                Some(value) => Ok(PathBuf::from(value)),
                None => home.as_ref().map(|home| home.join(fallback)).ok_or_else(|| "HOME is not set".to_string()),
            }
        };
        Ok(Paths {
            data: std::env::var_os("KAVAT_THEME_DATA").map_or("/usr/share/kavat/themes".into(), PathBuf::from),
            machine: "/etc/kavat/theme.toml".into(),
            config: folder("XDG_CONFIG_HOME", ".config")?,
            state: folder("XDG_STATE_HOME", ".local/state")?.join("kavat/theme"),
            system: false,
        })
    }

    /// The places for the screens before the desktop.
    pub fn for_the_system() -> Paths {
        Paths {
            data: std::env::var_os("KAVAT_THEME_DATA").map_or("/usr/share/kavat/themes".into(), PathBuf::from),
            machine: "/etc/kavat/theme.toml".into(),
            config: "/etc".into(),
            state: "/var/lib/kavat/theme".into(),
            system: true,
        }
    }

    /// The one file a person edits.
    pub fn user_file(&self) -> PathBuf {
        self.config.join("kavat/theme.toml")
    }

    pub fn palette_folders(&self) -> Vec<PathBuf> {
        if self.system {
            return vec![self.data.join("palettes")];
        }
        vec![self.config.join("kavat/palettes"), self.data.join("palettes")]
    }

    /// The templates by name; one of the user's takes the place of the system's.
    fn templates(&self) -> Result<Vec<Template>, String> {
        let mut folders = BTreeMap::new();
        let parents = match self.system {
            true => vec![self.data.join("boot")],
            false => vec![self.data.join("templates"), self.config.join("kavat/templates")],
        };
        for parent in parents {
            for entry in fs::read_dir(parent).into_iter().flatten().filter_map(Result::ok) {
                if entry.path().join("template.toml").exists() {
                    folders.insert(entry.file_name(), entry.path());
                }
            }
        }
        folders.values().map(|folder| Template::load(folder)).collect()
    }
}

pub fn read_document(path: &Path) -> Result<Option<DocumentMut>, String> {
    match fs::read_to_string(path) {
        Ok(text) => text.parse().map(Some).map_err(|problem| format!("{}: {problem}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

/// The settings in force. `user` stands in for the user's file when it is about to change;
/// for the system, it is the settings the command was given.
pub fn settings(paths: &Paths, user: Option<&DocumentMut>) -> Result<Settings, String> {
    let defaults: DocumentMut = DEFAULTS.parse().expect("the defaults are valid");
    let machine = read_document(&paths.machine)?;
    let on_disk = if user.is_none() && !paths.system { read_document(&paths.user_file())? } else { None };
    let machine_name = paths.machine.display().to_string();
    let user_name = if paths.system { "the settings given".to_string() } else { paths.user_file().display().to_string() };
    let mut layers = vec![("the defaults", &defaults)];
    if let Some(machine) = &machine {
        layers.push((&machine_name, machine));
    }
    if let Some(user) = user.or(on_disk.as_ref()) {
        layers.push((&user_name, user));
    }
    Settings::from_layers(&layers)
}

/// Everything applying the theme will do, worked out before anything is touched.
pub struct Plan {
    /// each generated file, by its place in the generated folder
    files: BTreeMap<String, String>,
    /// links to leave where apps look, and the generated file each one leads to
    links: Vec<(PathBuf, String)>,
    reload: Vec<Step>,
}

/// A command of a template, run in the template's folder once the files are in place.
struct Step {
    template: String,
    folder: PathBuf,
    words: Vec<String>,
}

impl Step {
    /// The command, ready to run: KAVAT_THEME_FILES tells it where the generated files are.
    fn command(&self, paths: &Paths) -> Command {
        let mut command = Command::new(&self.words[0]);
        command.args(&self.words[1..]).current_dir(&self.folder).env("KAVAT_THEME_FILES", paths.state.join("current"));
        command.stdin(Stdio::null());
        command
    }
}

pub fn tokens(paths: &Paths, settings: &Settings) -> Result<Tokens, String> {
    let palette = match settings.get("palette") {
        crate::theme::Token::Text(id) => Palette::load(id, &paths.palette_folders())?,
        _ => unreachable!(),
    };
    resolve(settings, &palette)
}

pub fn plan(paths: &Paths, tokens: &Tokens) -> Result<Plan, String> {
    let mut plan = Plan { files: BTreeMap::new(), links: Vec::new(), reload: Vec::new() };
    for template in paths.templates()? {
        for rule in &template.files {
            let problem = |what: String| format!("{}: {what}", rule.from.display());
            let text = fs::read_to_string(&rule.from).map_err(|error| problem(error.to_string()))?;
            let filled = render(&text, tokens).map_err(problem)?;
            if plan.files.insert(rule.to.clone(), filled).is_some() {
                return Err(format!("two templates write {}", rule.to));
            }
            match &rule.link {
                Some(_) if paths.system => return Err(problem("a template for the boot cannot leave links".into())),
                Some(link) => plan.links.push((paths.config.join(link.trim_start_matches("config:")), rule.to.clone())),
                None => {}
            }
        }
        for command in &template.reload {
            let words: Result<Vec<String>, String> = command.iter().map(|word| render(word, tokens)).collect();
            let words = words.map_err(|problem| format!("the reload command of {}: {problem}", template.name))?;
            plan.reload.push(Step { template: template.name.clone(), folder: template.folder.clone(), words });
        }
    }
    Ok(plan)
}

/// Whether the folder holds exactly the plan's files, with the same content.
fn holds(folder: &Path, files: &BTreeMap<String, String>) -> bool {
    fn count(folder: &Path) -> usize {
        fs::read_dir(folder).into_iter().flatten().filter_map(Result::ok).fold(0, |total, entry| {
            total + if entry.path().is_dir() { count(&entry.path()) } else { 1 }
        })
    }
    count(folder) == files.len()
        && files.iter().all(|(place, content)| fs::read_to_string(folder.join(place)).is_ok_and(|found| found == *content))
}

/// Puts the plan's files in place. True when something changed.
pub fn commit(paths: &Paths, plan: &Plan) -> Result<bool, String> {
    let problem = |error: std::io::Error| format!("{}: {error}", paths.state.display());
    fs::create_dir_all(&paths.state).map_err(problem)?;
    // one at a time: the session and a person may apply the theme at the same moment
    let lock = fs::File::create(paths.state.join("lock")).map_err(problem)?;
    lock.lock().map_err(problem)?;

    let current = paths.state.join("current");
    let changed = !holds(&current, &plan.files);
    if changed {
        let generations = |paths: &Paths| -> Vec<u64> {
            let names = fs::read_dir(&paths.state).into_iter().flatten().filter_map(Result::ok);
            names.filter_map(|entry| entry.file_name().to_string_lossy().parse().ok()).collect()
        };
        let before = fs::read_link(&current).ok().and_then(|target| target.to_string_lossy().parse::<u64>().ok());
        let number = generations(paths).into_iter().max().unwrap_or(0) + 1;
        let folder = paths.state.join(number.to_string());
        for (place, content) in &plan.files {
            let file = folder.join(place);
            fs::create_dir_all(file.parent().unwrap()).map_err(problem)?;
            fs::write(&file, content).map_err(problem)?;
        }
        // the one step that changes the theme: a new link takes the place of the old one
        let fresh = paths.state.join("current.new");
        let _ = fs::remove_file(&fresh);
        std::os::unix::fs::symlink(number.to_string(), &fresh).map_err(problem)?;
        fs::rename(&fresh, &current).map_err(problem)?;
        // the one before stays, for a program that is still reading it
        for old in generations(paths) {
            if old != number && Some(old) != before {
                let _ = fs::remove_dir_all(paths.state.join(old.to_string()));
            }
        }
    }

    if paths.system {
        // The login screen runs as its own user: it has to be able to read these, whatever
        // permissions the one who ran this gives to new files.
        fn open(path: &Path) {
            use std::os::unix::fs::PermissionsExt;
            let Ok(found) = fs::symlink_metadata(path) else { return };
            if found.is_dir() {
                let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
                for entry in fs::read_dir(path).into_iter().flatten().filter_map(Result::ok) {
                    open(&entry.path());
                }
            } else if found.is_file() {
                let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o644));
            }
        }
        if let Some(parent) = paths.state.parent() {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o755));
        }
        open(&paths.state);
    }

    for (link, place) in &plan.links {
        let target = current.join(place);
        match fs::read_link(link) {
            Ok(found) if found == target => {}
            // a link of ours from before, or one to a theme that is gone
            Ok(found) if found.starts_with(&paths.state) || !link.exists() => {
                let _ = fs::remove_file(link);
                std::os::unix::fs::symlink(&target, link).map_err(|error| format!("{}: {error}", link.display()))?;
            }
            Ok(_) => {}
            Err(_) if link.exists() => {} // the user's own file: it wins
            Err(_) => {
                fs::create_dir_all(link.parent().unwrap()).map_err(|error| format!("{}: {error}", link.display()))?;
                std::os::unix::fs::symlink(&target, link).map_err(|error| format!("{}: {error}", link.display()))?;
            }
        }
    }
    Ok(changed)
}

/// Tells the running apps to read their files again, all at once. An app that is not
/// running, or does not answer, is not an error: it reads its file when it starts.
pub fn reload(paths: &Paths, plan: &Plan) {
    let running: Vec<_> = plan
        .reload
        .iter()
        .filter_map(|step| step.command(paths).stdout(Stdio::null()).stderr(Stdio::null()).spawn().ok())
        .collect();
    for mut child in running {
        let _ = child.wait();
    }
}

/// For the screens before the desktop: each template's command puts its files where the
/// system reads them. One at a time and in the open, because these take a while and can
/// fail; one that fails leaves its screen as it was, and the others still get theirs.
pub fn install(paths: &Paths, plan: &Plan) -> Result<(), String> {
    let mut failed = Vec::new();
    for step in &plan.reload {
        match step.command(paths).status() {
            Ok(status) if status.success() => {}
            Ok(_) => failed.push(step.template.clone()),
            Err(error) => failed.push(format!("{} ({}: {error})", step.template, step.words[0])),
        }
    }
    if failed.is_empty() { Ok(()) } else { Err(format!("these kept the look they had: {}", failed.join(", "))) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A place of its own for a test: the system's files, and an empty user.
    fn paths(name: &str) -> Paths {
        let root = std::env::temp_dir().join(format!("kavat-theme-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        Paths {
            data: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            machine: root.join("etc/theme.toml"),
            config: root.join("config"),
            state: root.join("state/kavat/theme"),
            system: false,
        }
    }

    fn apply(paths: &Paths) -> Result<bool, String> {
        let settings = settings(paths, None)?;
        let plan = plan(paths, &tokens(paths, &settings)?)?;
        commit(paths, &plan)
    }

    fn write(path: PathBuf, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn the_files_are_made_and_reached_through_one_link() {
        let paths = paths("made");
        assert_eq!(apply(&paths), Ok(true));
        let look = fs::read_to_string(paths.state.join("current/hyprland/look.lua")).unwrap();
        assert!(look.contains("rgb(f2a93b)"), "{look}");
        assert_eq!(fs::read_link(paths.state.join("current")).unwrap(), PathBuf::from("1"));
        // where an app looks, a link leads to the current files
        let palette = fs::read_to_string(paths.config.join("noctalia/palettes/Kavat.json")).unwrap();
        assert!(palette.contains("\"mPrimary\": \"#f2a93b\""), "{palette}");
    }

    #[test]
    fn the_screens_before_the_desktop_take_the_machine_s_look_and_nothing_of_a_user() {
        let mut paths = paths("boot");
        paths.system = true;
        write(paths.machine.clone(), "mode = \"light\"\n");
        write(paths.user_file(), "palette = \"nord\"\n");
        write(paths.config.join("kavat/templates/mine/template.toml"), "[[file]]\nfrom = \"a.conf\"\n");
        let settings = settings(&paths, None).unwrap();
        let plan = plan(&paths, &tokens(&paths, &settings).unwrap()).unwrap();
        assert_eq!(commit(&paths, &plan), Ok(true));
        let menu = fs::read_to_string(paths.state.join("current/grub/theme.txt")).unwrap();
        assert!(menu.contains("desktop-color: \"#ece8de\""), "{menu}");
        let loading = fs::read_to_string(paths.state.join("current/plymouth/kavat.script")).unwrap();
        assert!(loading.contains("Window.SetBackgroundTopColor(0.925, 0.910, 0.871);"), "{loading}");
        assert!(!paths.state.join("current/kitty").exists(), "the desktop's templates are not for the boot");
        // the settings a person hands over go on top of the machine's
        let given: DocumentMut = "mode = \"dark\"\n".parse().unwrap();
        let settings = super::settings(&paths, Some(&given)).unwrap();
        let plan = super::plan(&paths, &tokens(&paths, &settings).unwrap()).unwrap();
        assert_eq!(commit(&paths, &plan), Ok(true));
        let menu = fs::read_to_string(paths.state.join("current/grub/theme.txt")).unwrap();
        assert!(menu.contains("desktop-color: \"#0d0f14\""), "{menu}");
    }

    #[test]
    fn applying_again_changes_nothing_and_a_change_makes_a_new_generation() {
        let paths = paths("again");
        assert_eq!(apply(&paths), Ok(true));
        assert_eq!(apply(&paths), Ok(false));
        assert_eq!(fs::read_link(paths.state.join("current")).unwrap(), PathBuf::from("1"));
        write(paths.user_file(), "mode = \"light\"\n");
        assert_eq!(apply(&paths), Ok(true));
        assert_eq!(fs::read_link(paths.state.join("current")).unwrap(), PathBuf::from("2"));
        let conf = fs::read_to_string(paths.state.join("current/kitty/theme.conf")).unwrap();
        assert!(conf.contains("background           #fbf9f4"), "{conf}");
        // the link an app follows now leads to the new files, without having changed
        let palette = fs::read_to_string(paths.config.join("noctalia/palettes/Kavat.json")).unwrap();
        assert!(palette.contains("\"mSurface\": \"#fbf9f4\""), "{palette}");
    }

    #[test]
    fn a_template_that_fails_leaves_the_theme_as_it_was() {
        let paths = paths("fails");
        assert_eq!(apply(&paths), Ok(true));
        write(paths.config.join("kavat/templates/broken/template.toml"), "[[file]]\nfrom = \"x.conf\"\n");
        write(paths.config.join("kavat/templates/broken/x.conf"), "colour {{ acent }}\n");
        write(paths.user_file(), "mode = \"light\"\n");
        let problem = apply(&paths).unwrap_err();
        assert!(problem.ends_with("x.conf: line 1: there is no token called acent"), "{problem}");
        assert_eq!(fs::read_link(paths.state.join("current")).unwrap(), PathBuf::from("1"));
        let look = fs::read_to_string(paths.state.join("current/hyprland/look.lua")).unwrap();
        assert!(look.contains("rgb(2a2f3f)"), "the dark theme is still there: {look}");
    }

    #[test]
    fn a_template_of_the_user_adds_an_app_and_can_replace_one_of_the_system() {
        let paths = paths("added");
        write(paths.config.join("kavat/templates/mine/template.toml"), "[[file]]\nfrom = \"a.conf\"\n");
        write(paths.config.join("kavat/templates/mine/a.conf"), "accent={{ accent | hex }}\n");
        write(paths.config.join("kavat/templates/kitty/template.toml"), "[[file]]\nfrom = \"theme.conf\"\n");
        write(paths.config.join("kavat/templates/kitty/theme.conf"), "background {{ bg }}\n");
        assert_eq!(apply(&paths), Ok(true));
        assert_eq!(fs::read_to_string(paths.state.join("current/mine/a.conf")).unwrap(), "accent=f2a93b\n");
        assert_eq!(fs::read_to_string(paths.state.join("current/kitty/theme.conf")).unwrap(), "background #0d0f14\n");
    }

    #[test]
    fn a_file_of_the_user_where_a_link_would_go_is_left_alone() {
        let paths = paths("own");
        write(paths.config.join("gtk-4.0/gtk.css"), "/* mine */\n");
        assert_eq!(apply(&paths), Ok(true));
        assert_eq!(fs::read_to_string(paths.config.join("gtk-4.0/gtk.css")).unwrap(), "/* mine */\n");
        assert!(fs::read_link(paths.config.join("gtk-3.0/gtk.css")).is_ok());
    }

    #[test]
    fn a_broken_settings_file_is_an_error_that_names_it() {
        let paths = paths("broken");
        write(paths.user_file(), "mode = light\n");
        let problem = apply(&paths).unwrap_err();
        assert!(problem.contains("config/kavat/theme.toml"), "{problem}");
    }
}
