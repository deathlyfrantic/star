use clap::ArgMatches;
use std::io::{Error, ErrorKind, Result};
use termion::{color, style};

pub struct Colors {
    pub normal: String,
    pub selected: String,
    pub matched: String,
    pub matched_selected: String,
}

fn default_fg() -> Colors {
    Colors {
        normal: format!("{}", color::Fg(color::Reset)),
        selected: format!("{}{}", color::Fg(color::Reset), style::Invert),
        matched: format!("{}", color::Fg(color::Red)),
        matched_selected: format!("{}", color::Fg(color::Red)),
    }
}

fn default_bg() -> Colors {
    Colors {
        normal: format!("{}", color::Bg(color::Reset)),
        selected: String::new(),
        matched: String::new(),
        matched_selected: String::new(),
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

fn parse_fg_color(color: &str) -> Result<String> {
    if color.starts_with('#') {
        return Ok(format!("{}", color::Fg(parse_rgb_color(color)?)));
    }
    if let Ok(n) = u8::from_str_radix(color, 10) {
        return Ok(format!("{}", color::Fg(color::AnsiValue(n))));
    }
    Ok(match color.to_lowercase().as_str() {
        "black" => format!("{}", color::Fg(color::Black)),
        "blue" => format!("{}", color::Fg(color::Blue)),
        "cyan" => format!("{}", color::Fg(color::Cyan)),
        "green" => format!("{}", color::Fg(color::Green)),
        "lightblack" => format!("{}", color::Fg(color::LightBlack)),
        "lightblue" => format!("{}", color::Fg(color::LightBlue)),
        "lightcyan" => format!("{}", color::Fg(color::LightCyan)),
        "lightgreen" => format!("{}", color::Fg(color::LightGreen)),
        "lightmagenta" => format!("{}", color::Fg(color::LightMagenta)),
        "lightred" => format!("{}", color::Fg(color::LightRed)),
        "lightwhite" => format!("{}", color::Fg(color::LightWhite)),
        "lightyellow" => format!("{}", color::Fg(color::LightYellow)),
        "magenta" => format!("{}", color::Fg(color::Magenta)),
        "red" => format!("{}", color::Fg(color::Red)),
        "white" => format!("{}", color::Fg(color::White)),
        "yellow" => format!("{}", color::Fg(color::Yellow)),
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid color specification: \"{}\"", color),
            ));
        }
    })
}

fn parse_bg_color(color: &str) -> Result<String> {
    if color.starts_with('#') {
        return Ok(format!("{}", color::Bg(parse_rgb_color(color)?)));
    }
    if let Ok(n) = u8::from_str_radix(color, 10) {
        return Ok(format!("{}", color::Bg(color::AnsiValue(n))));
    }
    Ok(match color.to_lowercase().as_str() {
        "black" => format!("{}", color::Bg(color::Black)),
        "blue" => format!("{}", color::Bg(color::Blue)),
        "cyan" => format!("{}", color::Bg(color::Cyan)),
        "green" => format!("{}", color::Bg(color::Green)),
        "lightblack" => format!("{}", color::Bg(color::LightBlack)),
        "lightblue" => format!("{}", color::Bg(color::LightBlue)),
        "lightcyan" => format!("{}", color::Bg(color::LightCyan)),
        "lightgreen" => format!("{}", color::Bg(color::LightGreen)),
        "lightmagenta" => format!("{}", color::Bg(color::LightMagenta)),
        "lightred" => format!("{}", color::Bg(color::LightRed)),
        "lightwhite" => format!("{}", color::Bg(color::LightWhite)),
        "lightyellow" => format!("{}", color::Bg(color::LightYellow)),
        "magenta" => format!("{}", color::Bg(color::Magenta)),
        "red" => format!("{}", color::Bg(color::Red)),
        "white" => format!("{}", color::Bg(color::White)),
        "yellow" => format!("{}", color::Bg(color::Yellow)),
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid color specification: \"{}\"", color),
            ));
        }
    })
}

pub fn get_colors(matches: &ArgMatches) -> Result<(Colors, Colors)> {
    let mut fg = default_fg();
    let mut bg = default_bg();
    if let Some(c) = matches.value_of("color-normal-fg") {
        fg.normal = parse_fg_color(c)?;
    }
    if let Some(c) = matches.value_of("color-matched-fg") {
        fg.matched = parse_fg_color(c)?;
    }
    if let Some(c) = matches.value_of("color-selected-fg") {
        fg.selected = parse_fg_color(c)?;
    }
    if let Some(c) = matches.value_of("color-normal-bg") {
        bg.normal = parse_bg_color(c)?;
    }
    if let Some(c) = matches.value_of("color-matched-bg") {
        bg.matched = parse_bg_color(c)?;
    }
    if let Some(c) = matches.value_of("color-selected-bg") {
        bg.selected = parse_bg_color(c)?;
        if let None = matches.value_of("color-selected-fg") {
            // if we set a background color on the selected line but don't set a foreground color,
            // then we need to clear the default foreground style::Invert. by setting it to the
            // same style as the normal fg, we ensure highlighting works correctly on selected
            // lines - if just set to an empty string, the section after the matched portion of the
            // selected line will not be cleared correctly.
            fg.selected = fg.normal.clone();
        }
    }
    if let Some(c) = matches.value_of("color-matched-selected-fg") {
        fg.matched_selected = parse_fg_color(c)?;
    } else {
        fg.matched_selected = fg.matched.clone();
    }
    if let Some(c) = matches.value_of("color-matched-selected-bg") {
        bg.matched_selected = parse_bg_color(c)?;
    } else {
        bg.matched_selected = bg.selected.clone();
    }
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

    #[test]
    fn test_parse_fg_color() {
        assert_eq!(
            format!("{}", color::Fg(color::Red)),
            parse_fg_color("red").unwrap()
        );
        assert_eq!(
            format!("{}", color::Fg(color::LightBlack)),
            parse_fg_color("LIGHTBLACK").unwrap()
        );
        assert!(parse_fg_color("awoijf").is_err());
    }
}
