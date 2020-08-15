use clap::ArgMatches;
use std::io::{Error, ErrorKind, Result};
use termion::{color, style};

pub struct Colors {
    pub normal: String,
    pub selected: String,
    pub matched: String,
    pub matched_selected: String,
    pub tag: String,
}

fn default_fg() -> Colors {
    Colors {
        normal: format!("{}", color::Fg(color::Reset)),
        selected: format!("{}{}", color::Fg(color::Reset), style::Invert),
        matched: format!("{}", color::Fg(color::Red)),
        matched_selected: format!("{}", color::Fg(color::Red)),
        tag: format!("{}", color::Fg(color::LightBlue)),
    }
}

fn default_bg() -> Colors {
    Colors {
        normal: format!("{}", color::Bg(color::Reset)),
        selected: String::new(),
        matched: String::new(),
        matched_selected: String::new(),
        tag: String::new(),
    }
}

fn parse_hex_color(color: &str) -> Result<u8> {
    match u8::from_str_radix(color, 16) {
        Ok(n) => Ok(n),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("invalid hex input for RGB color: \"{}\"", color),
        )),
    }
}

fn parse_rgb_color(color: &str) -> Result<color::Rgb> {
    if color.len() != 7 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "RGB color specification must be in the form \"#RRGGBB\"",
        ));
    }
    let red = parse_hex_color(color.get(1..3).unwrap())?;
    let green = parse_hex_color(color.get(3..5).unwrap())?;
    let blue = parse_hex_color(color.get(5..).unwrap())?;
    Ok(color::Rgb(red, green, blue))
}

macro_rules! parse_color {
    ($color:ident, $attr:tt) => {
        if $color.starts_with('#') {
            Ok(format!("{}", color::$attr(parse_rgb_color($color)?)))
        } else if let Ok(n) = u8::from_str_radix($color, 10) {
            Ok(format!("{}", color::$attr(color::AnsiValue(n))))
        } else {
            Ok(match $color.to_lowercase().as_str() {
                "black" => format!("{}", color::$attr(color::Black)),
                "blue" => format!("{}", color::$attr(color::Blue)),
                "cyan" => format!("{}", color::$attr(color::Cyan)),
                "green" => format!("{}", color::$attr(color::Green)),
                "lightblack" => format!("{}", color::$attr(color::LightBlack)),
                "lightblue" => format!("{}", color::$attr(color::LightBlue)),
                "lightcyan" => format!("{}", color::$attr(color::LightCyan)),
                "lightgreen" => format!("{}", color::$attr(color::LightGreen)),
                "lightmagenta" => format!("{}", color::$attr(color::LightMagenta)),
                "lightred" => format!("{}", color::$attr(color::LightRed)),
                "lightwhite" => format!("{}", color::$attr(color::LightWhite)),
                "lightyellow" => format!("{}", color::$attr(color::LightYellow)),
                "magenta" => format!("{}", color::$attr(color::Magenta)),
                "red" => format!("{}", color::$attr(color::Red)),
                "white" => format!("{}", color::$attr(color::White)),
                "yellow" => format!("{}", color::$attr(color::Yellow)),
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("invalid color specification: \"{}\"", $color),
                    ));
                }
            })
        } as Result<String>
    };
}

pub fn get_colors(matches: &ArgMatches) -> Result<(Colors, Colors)> {
    let mut fg = default_fg();
    let mut bg = default_bg();
    if let Some(c) = matches.value_of("color-normal-fg") {
        fg.normal = parse_color!(c, Fg)?;
    }
    if let Some(c) = matches.value_of("color-matched-fg") {
        fg.matched = parse_color!(c, Fg)?;
    }
    if let Some(c) = matches.value_of("color-selected-fg") {
        fg.selected = parse_color!(c, Fg)?;
    }
    if let Some(c) = matches.value_of("color-normal-bg") {
        bg.normal = parse_color!(c, Bg)?;
    }
    if let Some(c) = matches.value_of("color-matched-bg") {
        bg.matched = parse_color!(c, Bg)?;
    }
    if let Some(c) = matches.value_of("color-selected-bg") {
        bg.selected = parse_color!(c, Bg)?;
        if matches.value_of("color-selected-fg").is_none() {
            // if we set a background color on the selected line but don't set a foreground color,
            // then we need to clear the default foreground style::Invert. by setting it to the
            // same style as the normal fg, we ensure highlighting works correctly on selected
            // lines - if just set to an empty string, the section after the matched portion of the
            // selected line will not be cleared correctly.
            fg.selected = fg.normal.clone();
        }
    }
    if let Some(c) = matches.value_of("color-matched-selected-fg") {
        fg.matched_selected = parse_color!(c, Fg)?;
    } else {
        fg.matched_selected = fg.matched.clone();
    }
    if let Some(c) = matches.value_of("color-matched-selected-bg") {
        bg.matched_selected = parse_color!(c, Bg)?;
    } else {
        bg.matched_selected = bg.selected.clone();
    }
    if let Some(c) = matches.value_of("color-tag-fg") {
        fg.tag = parse_color!(c, Fg)?;
    }
    if let Some(c) = matches.value_of("color-tag-bg") {
        bg.tag = parse_color!(c, Bg)?;
    };
    Ok((fg, bg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(0x99u8, parse_hex_color("99").unwrap());
        assert!(parse_hex_color("z7").is_err());
    }

    #[test]
    fn test_parse_rgb_color() {
        assert_eq!(
            color::Rgb(0x33u8, 0x99u8, 0xCCu8),
            parse_rgb_color("#3399cc").unwrap()
        );
        assert!(parse_rgb_color("#123").is_err());
        assert!(parse_rgb_color("#z7z7z7").is_err());
    }

    fn test_parse_color_macro_helper(color: &str) -> Result<String> {
        parse_color!(color, Fg)
    }

    #[test]
    fn test_parse_color_macro() {
        assert_eq!(
            format!("{}", color::Fg(color::Red)),
            test_parse_color_macro_helper("red").unwrap()
        );
        assert_eq!(
            format!("{}", color::Fg(color::LightBlack)),
            test_parse_color_macro_helper("LIGHTBLACK").unwrap()
        );
        assert!(test_parse_color_macro_helper("awoijf").is_err());
    }
}
