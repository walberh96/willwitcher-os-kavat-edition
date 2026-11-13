//! From the settings a person writes to the tokens the templates use.
//!
//! The settings are a combination of independent axes: palette, mode, accent, tone,
//! contrast, shape, effects and fonts. A palette is one TOML file with a dark and a light
//! variant. Resolving them gives the tokens: every colour and value an app's file needs.

use crate::color::{BLACK, Color, WHITE};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Table};

/// The settings every system starts with; the file doubles as their documentation.
pub const DEFAULTS: &str = include_str!("../theme.toml");

/// What a setting holds, and what it accepts.
#[derive(Clone, Copy)]
pub enum Kind {
    Text,
    OneOf(&'static [&'static str]),
    Whole(i64, i64),
    Number(f64, f64),
    Flag,
}

/// Every setting there is. A key that is not here is a mistake, and is reported as one.
pub const KEYS: &[(&str, Kind)] = &[
    ("palette", Kind::Text),
    ("mode", Kind::OneOf(&["dark", "light"])),
    ("accent", Kind::Text),
    ("tone", Kind::OneOf(&["neutral", "warm", "cool", "oled"])),
    ("contrast", Kind::OneOf(&["normal", "high"])),
    ("shape.radius", Kind::Whole(0, 40)),
    ("shape.border", Kind::Whole(0, 10)),
    ("shape.gap", Kind::Whole(0, 60)),
    ("shape.edge", Kind::Whole(0, 120)),
    ("effects.blur", Kind::Flag),
    ("effects.transparency", Kind::Flag),
    ("effects.shadows", Kind::Flag),
    ("effects.animations", Kind::Number(0.0, 4.0)),
    ("fonts.ui", Kind::Text),
    ("fonts.mono", Kind::Text),
    ("fonts.scale", Kind::Number(0.5, 2.0)),
];

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Color(Color),
    Number(f64),
    Flag(bool),
    Text(String),
}

pub type Tokens = BTreeMap<String, Token>;

pub fn kind_of(key: &str) -> Option<Kind> {
    KEYS.iter().find(|(name, _)| *name == key).map(|(_, kind)| *kind)
}

fn accepted(kind: Kind) -> String {
    match kind {
        Kind::Text => "a text of letters, digits, spaces and - _ . #".into(),
        Kind::OneOf(values) => values.join(", "),
        Kind::Whole(low, high) => format!("a whole number from {low} to {high}"),
        Kind::Number(low, high) => format!("a number from {low} to {high}"),
        Kind::Flag => "true or false".into(),
    }
}

/// Checks a value against what its setting accepts.
fn check(key: &str, kind: Kind, value: &Token) -> Result<(), String> {
    let ok = match (kind, value) {
        (Kind::Text, Token::Text(text)) => !text.is_empty() && plain(text),
        (Kind::OneOf(values), Token::Text(text)) => values.contains(&text.as_str()),
        (Kind::Whole(low, high), Token::Number(n)) => n.fract() == 0.0 && *n >= low as f64 && *n <= high as f64,
        (Kind::Number(low, high), Token::Number(n)) => *n >= low && *n <= high,
        (Kind::Flag, Token::Flag(_)) => true,
        _ => false,
    };
    if ok { Ok(()) } else { Err(format!("{key} takes {}", accepted(kind))) }
}

/// Whether a text is only letters, digits, spaces and `- _ . #`.
///
/// Texts end up inside other programs' configuration files, some of which are code. One
/// with a quote or a line break in it could add lines of its own there, so it is refused.
fn plain(text: &str) -> bool {
    text.chars().all(|c| c.is_alphanumeric() || " -_.#".contains(c))
}

/// A value typed on the command line, read as its setting expects it.
pub fn parse_value(key: &str, text: &str) -> Result<Token, String> {
    let kind = kind_of(key).ok_or_else(|| unknown_key(key))?;
    let value = match kind {
        Kind::Text | Kind::OneOf(_) => Token::Text(text.to_string()),
        Kind::Whole(..) | Kind::Number(..) => {
            Token::Number(text.parse::<f64>().map_err(|_| format!("{key} takes {}", accepted(kind)))?)
        }
        Kind::Flag => match text {
            "true" | "on" | "yes" => Token::Flag(true),
            "false" | "off" | "no" => Token::Flag(false),
            _ => return Err(format!("{key} takes {}", accepted(kind))),
        },
    };
    check(key, kind, &value)?;
    Ok(value)
}

pub enum WriteProblem {
    Value(String),
    NotASection(String),
}

/// Puts a value typed on the command line in a settings document, leaving the rest of the
/// document, comments included, as it was.
pub fn write_setting(document: &mut DocumentMut, key: &str, text: &str) -> Result<(), WriteProblem> {
    let value = match parse_value(key, text).map_err(WriteProblem::Value)? {
        Token::Text(text) => toml_edit::Value::from(text),
        Token::Flag(flag) => toml_edit::Value::from(flag),
        Token::Number(number) if matches!(kind_of(key), Some(Kind::Whole(..))) => toml_edit::Value::from(number as i64),
        Token::Number(number) => toml_edit::Value::from(number),
        Token::Color(_) => unreachable!(),
    };
    let mut table = document.as_table_mut();
    let mut parts = key.split('.').peekable();
    while let Some(part) = parts.next() {
        if parts.peek().is_some() {
            table = table
                .entry(part)
                .or_insert(toml_edit::table())
                .as_table_mut()
                .ok_or_else(|| WriteProblem::NotASection(part.to_string()))?;
        } else if let Some(old) = table.get_mut(part).and_then(Item::as_value_mut) {
            // the value changes; the comment next to it stays
            let decor = old.decor().clone();
            *old = value.clone();
            *old.decor_mut() = decor;
        } else {
            table[part] = toml_edit::value(value.clone());
        }
    }
    Ok(())
}

fn unknown_key(key: &str) -> String {
    let names: Vec<&str> = KEYS.iter().map(|(name, _)| *name).collect();
    format!("there is no setting called {key}; the settings are: {}", names.join(", "))
}

/// The settings in force: the system's defaults with the files above them laid over.
pub struct Settings(BTreeMap<&'static str, Token>);

impl Settings {
    /// `layers` go from the weakest to the strongest: defaults, the machine's, the user's.
    pub fn from_layers(layers: &[(&str, &DocumentMut)]) -> Result<Settings, String> {
        let mut values = BTreeMap::new();
        for (origin, document) in layers {
            for (key, item) in flatten(document.as_table(), "") {
                let (name, kind) = KEYS
                    .iter()
                    .find(|(name, _)| *name == key)
                    .ok_or_else(|| format!("{origin}: {}", unknown_key(&key)))?;
                let value = item_token(item).ok_or_else(|| format!("{origin}: {key} takes {}", accepted(*kind)))?;
                check(name, *kind, &value).map_err(|problem| format!("{origin}: {problem}"))?;
                values.insert(*name, value);
            }
        }
        for (name, _) in KEYS {
            if !values.contains_key(name) {
                return Err(format!("the setting {name} has no value"));
            }
        }
        Ok(Settings(values))
    }

    pub fn get(&self, key: &str) -> &Token {
        &self.0[key]
    }

    fn text(&self, key: &str) -> &str {
        match &self.0[key] {
            Token::Text(text) => text,
            _ => unreachable!(),
        }
    }

    fn number(&self, key: &str) -> f64 {
        match self.0[key] {
            Token::Number(n) => n,
            _ => unreachable!(),
        }
    }

    fn flag(&self, key: &str) -> bool {
        matches!(self.0[key], Token::Flag(true))
    }
}

/// Every value of a document with its dotted key: `shape.radius`.
fn flatten<'a>(table: &'a Table, prefix: &str) -> Vec<(String, &'a Item)> {
    let mut found = Vec::new();
    for (key, item) in table.iter() {
        let name = format!("{prefix}{key}");
        match item.as_table() {
            Some(inner) => found.extend(flatten(inner, &format!("{name}."))),
            None => found.push((name, item)),
        }
    }
    found
}

fn item_token(item: &Item) -> Option<Token> {
    let value = item.as_value()?;
    if let Some(text) = value.as_str() {
        Some(Token::Text(text.to_string()))
    } else if let Some(flag) = value.as_bool() {
        Some(Token::Flag(flag))
    } else if let Some(whole) = value.as_integer() {
        Some(Token::Number(whole as f64))
    } else {
        value.as_float().map(Token::Number)
    }
}

/// The six neutrals and six hues of one mode of a palette.
pub struct Variant {
    bg: Color,
    surface: Color,
    surface2: Color,
    text: Color,
    muted: Color,
    border: Color,
    /// red, green, yellow, blue, magenta, cyan; a palette may leave them out
    hues: [Option<Color>; 6],
}

pub struct Palette {
    pub id: String,
    pub name: String,
    /// by name, in the file's order; the first one is the palette's own
    pub accents: Vec<(String, Color)>,
    pub dark: Option<Variant>,
    pub light: Option<Variant>,
}

const HUES: [&str; 6] = ["red", "green", "yellow", "blue", "magenta", "cyan"];
// the hues of a palette that brings none
const DARK_HUES: [&str; 6] = ["#ef6b73", "#7ec77a", "#e6b450", "#6cb6ff", "#b79cff", "#5fd0c3"];
const LIGHT_HUES: [&str; 6] = ["#c7384a", "#3d8b40", "#a86f00", "#2468c4", "#7a52cc", "#177f76"];

impl Palette {
    pub fn parse(id: &str, text: &str) -> Result<Palette, String> {
        let document: DocumentMut = text.parse().map_err(|problem| format!("{problem}"))?;
        let colour = |table: &Table, key: &str| -> Result<Option<Color>, String> {
            match table.get(key) {
                None => Ok(None),
                Some(item) => item
                    .as_str()
                    .and_then(Color::parse)
                    .map(Some)
                    .ok_or_else(|| format!("{key} is not a colour like \"#f2a93b\"")),
            }
        };
        let variant = |name: &str| -> Result<Option<Variant>, String> {
            let Some(table) = document.get(name).and_then(Item::as_table) else { return Ok(None) };
            let need = |key: &str| colour(table, key)?.ok_or_else(|| format!("[{name}] has no {key}"));
            let mut hues = [None; 6];
            for (slot, hue) in hues.iter_mut().zip(HUES) {
                *slot = colour(table, hue).map_err(|problem| format!("[{name}] {problem}"))?;
            }
            Ok(Some(Variant {
                bg: need("bg")?,
                surface: need("surface")?,
                surface2: need("surface2")?,
                text: need("text")?,
                muted: need("muted")?,
                border: need("border")?,
                hues,
            }))
        };
        let mut accents = Vec::new();
        if let Some(table) = document.get("accents").and_then(Item::as_table) {
            for (name, _) in table.iter() {
                let value = colour(table, name)?.unwrap();
                accents.push((name.to_string(), value));
            }
        }
        if accents.is_empty() {
            return Err("[accents] names no colour".into());
        }
        let name = document.get("name").and_then(Item::as_str).unwrap_or(id);
        if !plain(id) || !plain(name) || accents.iter().any(|(accent, _)| !plain(accent)) {
            return Err("its names can only have letters, digits, spaces and - _ . #".into());
        }
        let palette = Palette {
            id: id.to_string(),
            name: name.to_string(),
            accents,
            dark: variant("dark")?,
            light: variant("light")?,
        };
        if palette.dark.is_none() && palette.light.is_none() {
            return Err("it has neither [dark] nor [light]".into());
        }
        Ok(palette)
    }

    pub fn load(id: &str, folders: &[PathBuf]) -> Result<Palette, String> {
        for folder in folders {
            let path = folder.join(format!("{id}.toml"));
            if let Ok(text) = std::fs::read_to_string(&path) {
                return Palette::parse(id, &text).map_err(|problem| format!("{}: {problem}", path.display()));
            }
        }
        Err(format!("there is no palette called {id}; the palettes are: {}", available(folders).join(", ")))
    }
}

/// The palettes found in the folders, by the name their file gives them.
pub fn available(folders: &[PathBuf]) -> Vec<String> {
    let mut ids: Vec<String> = folders
        .iter()
        .filter_map(|folder| std::fs::read_dir(folder).ok())
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
        .filter_map(|path: PathBuf| Path::file_stem(&path).map(|stem| stem.to_string_lossy().into_owned()))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// Works out every token from the settings and their palette.
pub fn resolve(settings: &Settings, palette: &Palette) -> Result<Tokens, String> {
    let mode = settings.text("mode");
    let dark = mode == "dark";
    let variant = if dark { &palette.dark } else { &palette.light }
        .as_ref()
        .ok_or_else(|| format!("the palette {} has no {mode} variant", palette.name))?;
    let high = settings.text("contrast") == "high";
    let readable = if high { 7.0 } else { 4.5 };

    // Neutrals: the tone warms or cools them, or takes the dark ones down to pure black.
    let tone = settings.text("tone");
    let tinted = |colour: Color| match tone {
        "warm" => colour.tinted(70.0, 0.012),
        "cool" => colour.tinted(250.0, 0.012),
        _ => colour,
    };
    let (mut bg, mut surface, mut surface2, mut border) =
        (tinted(variant.bg), tinted(variant.surface), tinted(variant.surface2), tinted(variant.border));
    if tone == "oled" && dark {
        bg = BLACK;
        surface = surface.mix(BLACK, 0.55);
        surface2 = surface2.mix(BLACK, 0.35);
        border = border.mix(BLACK, 0.2);
    }
    let mut text = variant.text;
    let mut muted = variant.muted;
    if high {
        text = text.mix(if dark { WHITE } else { BLACK }, 0.6);
        muted = muted.mix(text, 0.5);
        border = border.mix(text, 0.3);
    }
    let text = text.readable_on(surface, 7.0);
    let muted = muted.readable_on(surface, 4.5);

    // The accent: one of the palette's by name, or any colour.
    let wanted = settings.text("accent");
    let position = match wanted {
        "default" => Some(0),
        name => palette.accents.iter().position(|(accent, _)| accent == name),
    };
    let accent = match position {
        Some(at) => palette.accents[at].1,
        None => Color::parse(wanted).ok_or_else(|| {
            let names: Vec<&str> = palette.accents.iter().map(|(name, _)| name.as_str()).collect();
            format!(
                "accent takes default, one of {}'s accents ({}) or a colour like \"#f2a93b\"",
                palette.name,
                names.join(", ")
            )
        })?,
    };
    // the two after it in the palette go with it; a free colour takes the first two
    let others = |step: usize| {
        let at = position.map_or(step - 1, |at| at + step) % palette.accents.len();
        palette.accents[at].1
    };
    let (accent2, accent3) = (others(1), others(2));
    let ink = |on: Color| on.best_ink(variant.text, variant.bg);

    let mut hues = [BLACK; 6];
    for (index, hue) in hues.iter_mut().enumerate() {
        let fallback = Color::parse(if dark { DARK_HUES[index] } else { LIGHT_HUES[index] }).unwrap();
        *hue = variant.hues[index].unwrap_or(fallback).readable_on(surface, 4.5);
    }
    let [red, green, yellow, blue, magenta, cyan] = hues;

    let mut tokens = Tokens::new();
    let mut colour = |name: &str, value: Color| {
        tokens.insert(name.to_string(), Token::Color(value));
    };
    colour("bg", bg);
    colour("surface", surface);
    colour("surface2", surface2);
    colour("text", text);
    colour("muted", muted);
    colour("border", border);
    colour("accent", accent);
    colour("accent_text", accent.readable_on(surface, readable));
    colour("on_accent", ink(accent));
    colour("accent2", accent2);
    colour("on_accent2", ink(accent2));
    colour("accent3", accent3);
    colour("on_accent3", ink(accent3));
    for (name, value) in HUES.iter().zip(hues) {
        colour(name, value);
    }
    colour("ok", green);
    colour("warn", yellow);
    colour("danger", red);
    colour("on_danger", ink(red));
    colour("info", blue);
    colour("selection", surface.mix(accent, 0.3));
    colour("hover", surface2.mix(text, 0.08));
    colour("shadow", BLACK);

    // The terminal. Its sixteen colours keep their names in both modes: black is the dark
    // one and white the light one, as in the light themes most programs are tried with.
    colour("term_bg", surface);
    colour("term_fg", text);
    colour("cursor", accent.readable_on(surface, 3.0));
    colour("cursor_text", surface);
    let (black, grey, white, bright_white) = if dark {
        (surface2, muted.mix(surface, 0.35).readable_on(surface, 3.0), text.mix(muted, 0.35), text)
    } else {
        (text.mix(muted, 0.5), muted, border.mix(muted, 0.35), border)
    };
    let brighter = if dark { 0.06 } else { -0.06 };
    let terminal = [black, red, green, yellow, blue, magenta, cyan, white];
    for (index, value) in terminal.iter().enumerate() {
        colour(&format!("term{index}"), *value);
        let bright = match index {
            0 => grey,
            7 => bright_white,
            _ => value.lighten(brighter),
        };
        colour(&format!("term{}", index + 8), bright);
    }

    let mut value = |name: &str, token: Token| {
        tokens.insert(name.to_string(), token);
    };
    value("mode", Token::Text(mode.to_string()));
    value("palette", Token::Text(palette.id.clone()));
    value("palette_name", Token::Text(palette.name.clone()));
    let radius = settings.number("shape.radius");
    value("radius", Token::Number(radius));
    // for programs that scale their own corners; 1 is the system's usual radius
    value("radius_scale", Token::Number(radius / 10.0));
    value("border_size", Token::Number(settings.number("shape.border")));
    value("gap", Token::Number(settings.number("shape.gap")));
    value("edge", Token::Number(settings.number("shape.edge")));
    value("blur", Token::Flag(settings.flag("effects.blur")));
    value("shadows", Token::Flag(settings.flag("effects.shadows")));
    let transparency = settings.flag("effects.transparency");
    value("transparency", Token::Flag(transparency));
    value("opacity", Token::Number(if transparency { 0.9 } else { 1.0 }));
    let speed = settings.number("effects.animations");
    value("animations", Token::Flag(speed > 0.0));
    value("animation_speed", Token::Number(if speed > 0.0 { speed } else { 1.0 }));
    value("font_ui", Token::Text(settings.text("fonts.ui").to_string()));
    value("font_mono", Token::Text(settings.text("fonts.mono").to_string()));
    let scale = settings.number("fonts.scale");
    value("font_scale", Token::Number(scale));
    value("font_size", Token::Number((11.0 * scale * 2.0).round() / 2.0));
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KAVAT: &str = include_str!("../palettes/kavat.toml");

    fn settings(over: &str) -> Result<Settings, String> {
        let defaults: DocumentMut = DEFAULTS.parse().unwrap();
        let user: DocumentMut = over.parse().unwrap();
        Settings::from_layers(&[("defaults", &defaults), ("user", &user)])
    }

    fn tokens(over: &str) -> Tokens {
        resolve(&settings(over).unwrap(), &Palette::parse("kavat", KAVAT).unwrap()).unwrap()
    }

    fn colour(tokens: &Tokens, name: &str) -> Color {
        match tokens[name] {
            Token::Color(colour) => colour,
            _ => panic!("{name} is not a colour"),
        }
    }

    #[test]
    fn the_defaults_are_the_dark_kavat_of_the_mockup() {
        let tokens = tokens("");
        assert_eq!(colour(&tokens, "bg").to_string(), "#0d0f14");
        assert_eq!(colour(&tokens, "surface").to_string(), "#151821");
        assert_eq!(colour(&tokens, "text").to_string(), "#e8e6df");
        assert_eq!(colour(&tokens, "accent").to_string(), "#f2a93b");
        assert_eq!(tokens["mode"], Token::Text("dark".into()));
        assert_eq!(tokens["radius"], Token::Number(10.0));
    }

    #[test]
    fn the_user_layer_wins_and_the_rest_stays() {
        let tokens = tokens("mode = \"light\"\n[shape]\nradius = 4\n");
        assert_eq!(colour(&tokens, "surface").to_string(), "#fbf9f4");
        assert_eq!(tokens["radius"], Token::Number(4.0));
        assert_eq!(tokens["border_size"], Token::Number(2.0));
    }

    #[test]
    fn every_text_colour_is_readable_in_both_modes() {
        for mode in ["dark", "light"] {
            for contrast in ["normal", "high"] {
                let tokens = tokens(&format!("mode = \"{mode}\"\ncontrast = \"{contrast}\"\n"));
                let surface = colour(&tokens, "surface");
                for name in ["text", "muted", "accent_text", "red", "green", "yellow", "blue", "magenta", "cyan"] {
                    let ratio = colour(&tokens, name).contrast(surface);
                    assert!(ratio >= 4.5, "{name} in {mode}, {contrast} contrast: {ratio:.2}");
                }
                for (fill, ink) in [("accent", "on_accent"), ("danger", "on_danger")] {
                    let ratio = colour(&tokens, fill).contrast(colour(&tokens, ink));
                    assert!(ratio >= 4.5, "{ink} on {fill} in {mode}: {ratio:.2}");
                }
            }
        }
    }

    #[test]
    fn the_accent_is_a_name_of_the_palette_or_a_colour() {
        assert_eq!(colour(&tokens("accent = \"teal\""), "accent").to_string(), "#3ecfb2");
        assert_eq!(colour(&tokens("accent = \"#ff00aa\""), "accent").to_string(), "#ff00aa");
        let palette = Palette::parse("kavat", KAVAT).unwrap();
        let problem = resolve(&settings("accent = \"mauve\"").unwrap(), &palette).unwrap_err();
        assert!(problem.contains("amber"), "{problem}");
    }

    #[test]
    fn oled_is_pure_black_and_only_in_the_dark() {
        assert_eq!(colour(&tokens("tone = \"oled\""), "bg"), BLACK);
        assert_eq!(colour(&tokens("tone = \"oled\"\nmode = \"light\""), "bg").to_string(), "#ece8de");
    }

    #[test]
    fn a_wrong_setting_is_said_with_its_file_and_what_it_takes() {
        let problem = settings("mod = \"light\"").err().unwrap();
        assert!(problem.starts_with("user: there is no setting called mod"), "{problem}");
        let problem = settings("mode = \"sepia\"").err().unwrap();
        assert_eq!(problem, "user: mode takes dark, light");
        let problem = settings("[shape]\nradius = 2.5").err().unwrap();
        assert_eq!(problem, "user: shape.radius takes a whole number from 0 to 40");
    }

    #[test]
    fn a_text_that_could_write_code_into_a_file_is_refused() {
        assert!(settings("[fonts]\nui = 'Inter\" os.execute(\"x\")'").is_err());
        assert!(settings("[fonts]\nui = \"Inter\\nfoo\"").is_err());
        assert!(settings("[fonts]\nui = \"JetBrains Mono\"").is_ok());
        let evil = KAVAT.replace("name = \"Kavat\"", "name = 'Kavat\"; x'");
        assert!(Palette::parse("kavat", &evil).is_err());
    }

    #[test]
    fn every_palette_of_the_system_resolves_in_both_modes() {
        let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("palettes");
        let ids = available(std::slice::from_ref(&folder));
        assert!(ids.len() >= 7, "{ids:?}");
        for id in ids {
            let palette = Palette::load(&id, std::slice::from_ref(&folder)).unwrap();
            for mode in ["dark", "light"] {
                let tokens = resolve(&settings(&format!("mode = \"{mode}\"")).unwrap(), &palette).unwrap();
                let surface = colour(&tokens, "surface");
                for name in ["text", "muted", "accent_text", "red", "green", "yellow", "blue", "magenta", "cyan"] {
                    let ratio = colour(&tokens, name).contrast(surface);
                    assert!(ratio >= 4.5, "{id}, {mode}: {name} has {ratio:.2}");
                }
                for index in 1..7 {
                    let ratio = colour(&tokens, &format!("term{}", index + 8)).contrast(surface);
                    assert!(ratio >= 4.5, "{id}, {mode}: bright colour {index} has {ratio:.2}");
                }
            }
        }
    }

    #[test]
    fn changing_a_setting_keeps_the_comments_of_the_file() {
        let mut document: DocumentMut = "# mine\nmode = \"dark\"  # for now\n\n[shape]\nradius = 10 # round\n".parse().unwrap();
        for (key, value) in [("mode", "light"), ("shape.radius", "4"), ("shape.gap", "8"), ("fonts.scale", "1.25")] {
            assert!(write_setting(&mut document, key, value).is_ok());
        }
        assert_eq!(
            document.to_string(),
            "# mine\nmode = \"light\"  # for now\n\n[shape]\nradius = 4 # round\ngap = 8\n\n[fonts]\nscale = 1.25\n"
        );
        assert!(write_setting(&mut document, "mode", "sepia").is_err());
    }

    #[test]
    fn values_typed_on_the_command_line_are_read_by_their_setting() {
        assert_eq!(parse_value("shape.radius", "12"), Ok(Token::Number(12.0)));
        assert_eq!(parse_value("effects.blur", "off"), Ok(Token::Flag(false)));
        assert!(parse_value("shape.radius", "twelve").is_err());
        assert!(parse_value("shape.raduis", "12").unwrap_err().contains("no setting called"));
    }
}
