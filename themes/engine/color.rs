//! Colours, and the arithmetic the theme does with them.
//!
//! Mixing and lightening happen in OKLab, where equal steps look equal; contrast is the
//! WCAG ratio, the one accessibility rules are written in.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };

/// A colour as lightness (0 to 1) and two colour axes.
#[derive(Clone, Copy)]
struct Oklab {
    l: f64,
    a: f64,
    b: f64,
}

fn to_linear(channel: u8) -> f64 {
    let c = channel as f64 / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn from_linear(c: f64) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let c = if c <= 0.0031308 { 12.92 * c } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
    (c * 255.0).round() as u8
}

impl Color {
    /// Reads `#rgb` or `#rrggbb`.
    pub fn parse(text: &str) -> Option<Color> {
        let hex = text.strip_prefix('#')?;
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let digits: Vec<u8> = hex.chars().map(|c| c.to_digit(16).unwrap() as u8).collect();
        match digits.len() {
            3 => Some(Color { r: digits[0] * 17, g: digits[1] * 17, b: digits[2] * 17 }),
            6 => Some(Color {
                r: digits[0] * 16 + digits[1],
                g: digits[2] * 16 + digits[3],
                b: digits[4] * 16 + digits[5],
            }),
            _ => None,
        }
    }

    /// `rrggbb`, without the `#`.
    pub fn hex(&self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    fn oklab(&self) -> Oklab {
        let (r, g, b) = (to_linear(self.r), to_linear(self.g), to_linear(self.b));
        let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
        let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
        let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
        Oklab {
            l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        }
    }

    fn from_oklab(c: Oklab) -> Color {
        let l = (c.l + 0.3963377774 * c.a + 0.2158037573 * c.b).powi(3);
        let m = (c.l - 0.1055613458 * c.a - 0.0638541728 * c.b).powi(3);
        let s = (c.l - 0.0894841775 * c.a - 1.2914855480 * c.b).powi(3);
        Color {
            r: from_linear(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
            g: from_linear(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
            b: from_linear(-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s),
        }
    }

    /// This colour with `amount` (0 to 1) of another mixed in.
    pub fn mix(&self, other: Color, amount: f64) -> Color {
        let (x, y) = (self.oklab(), other.oklab());
        Color::from_oklab(Oklab {
            l: x.l + (y.l - x.l) * amount,
            a: x.a + (y.a - x.a) * amount,
            b: x.b + (y.b - x.b) * amount,
        })
    }

    /// Lighter (positive) or darker (negative), by a share of the whole range.
    pub fn lighten(&self, by: f64) -> Color {
        let mut c = self.oklab();
        c.l = (c.l + by).clamp(0.0, 1.0);
        Color::from_oklab(c)
    }

    /// Pushed towards a hue, in degrees, by a small `amount`: warms or cools a neutral.
    pub fn tinted(&self, hue: f64, amount: f64) -> Color {
        let mut c = self.oklab();
        c.a += amount * hue.to_radians().cos();
        c.b += amount * hue.to_radians().sin();
        Color::from_oklab(c)
    }

    /// How light the eye takes it to be, 0 for black and 1 for white.
    pub fn luminance(&self) -> f64 {
        0.2126 * to_linear(self.r) + 0.7152 * to_linear(self.g) + 0.0722 * to_linear(self.b)
    }

    /// The WCAG contrast ratio with another colour, from 1 to 21.
    pub fn contrast(&self, other: Color) -> f64 {
        let (x, y) = (self.luminance(), other.luminance());
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    /// The closest colour of the same hue that reaches `ratio` against a background.
    ///
    /// Text goes lighter on a dark background and darker on a light one. A colour that
    /// already reaches the ratio comes back as it is.
    pub fn readable_on(&self, background: Color, ratio: f64) -> Color {
        if self.contrast(background) >= ratio {
            return *self;
        }
        let start = self.oklab();
        let end = if background.luminance() < 0.18 { 1.0 } else { 0.0 };
        let at = |t: f64| Color::from_oklab(Oklab { l: start.l + (end - start.l) * t, ..start });
        // the least change that is enough
        let (mut low, mut high) = (0.0, 1.0);
        for _ in 0..20 {
            let middle = (low + high) / 2.0;
            if at(middle).contrast(background) >= ratio { high = middle } else { low = middle }
        }
        let found = at(high);
        if found.contrast(background) >= ratio {
            found
        } else if end == 1.0 {
            WHITE
        } else {
            BLACK
        }
    }

    /// Of two colours, the one that reads better on this one.
    pub fn best_ink(&self, first: Color, second: Color) -> Color {
        let ink = if self.contrast(first) >= self.contrast(second) { first } else { second };
        if self.contrast(ink) >= 4.5 {
            ink
        } else if self.contrast(WHITE) >= self.contrast(BLACK) {
            WHITE
        } else {
            BLACK
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#{}", self.hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_prints() {
        assert_eq!(Color::parse("#f2a93b"), Some(Color { r: 0xf2, g: 0xa9, b: 0x3b }));
        assert_eq!(Color::parse("#fa0").unwrap().to_string(), "#ffaa00");
        assert_eq!(Color::parse("f2a93b"), None);
        assert_eq!(Color::parse("#f2a93"), None);
        assert_eq!(Color::parse("#gggggg"), None);
    }

    #[test]
    fn contrast_is_the_wcag_ratio() {
        assert!((BLACK.contrast(WHITE) - 21.0).abs() < 0.01);
        assert!((WHITE.contrast(WHITE) - 1.0).abs() < 0.01);
        // a pair with a published ratio: #767676 on white is 4.54
        let grey = Color::parse("#767676").unwrap();
        assert!((grey.contrast(WHITE) - 4.54).abs() < 0.01);
    }

    #[test]
    fn oklab_goes_there_and_back() {
        for text in ["#000000", "#ffffff", "#f2a93b", "#6cb6ff", "#1d1f26"] {
            let c = Color::parse(text).unwrap();
            assert_eq!(Color::from_oklab(c.oklab()), c);
        }
    }

    #[test]
    fn mixing_ends_at_the_two_colours() {
        let (a, b) = (Color::parse("#f2a93b").unwrap(), Color::parse("#151821").unwrap());
        assert_eq!(a.mix(b, 0.0), a);
        assert_eq!(a.mix(b, 1.0), b);
    }

    #[test]
    fn readable_reaches_the_ratio_and_changes_no_more_than_needed() {
        let paper = Color::parse("#fbf9f4").unwrap();
        let amber = Color::parse("#f2a93b").unwrap();
        let fixed = amber.readable_on(paper, 4.5);
        assert!(fixed.contrast(paper) >= 4.5);
        assert!(fixed.contrast(paper) < 4.8);
        // already readable: untouched
        let night = Color::parse("#151821").unwrap();
        assert_eq!(amber.readable_on(night, 4.5), amber);
    }
}
