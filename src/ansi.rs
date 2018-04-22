fn escape(sequence: &str) -> String {
    format!("\x1B[{}", sequence)
}

pub fn clear() -> String {
    escape("2J")
}

pub fn hide_cursor() -> String {
    escape("?25l")
}

pub fn show_cursor() -> String {
    escape("?25h")
}

pub fn cursor_up(lines: usize) -> String {
    escape(format!("{}A", lines).as_str())
}

pub fn clear_line() -> String {
    escape("2K")
}

pub fn color(fg: &str, bg: &str) -> String {
    let fg = match fg.to_lowercase().as_str() {
        "black" => 30,
        "red" => 31,
        "green" => 32,
        "yellow" => 33,
        "blue" => 34,
        "magenta" => 35,
        "cyan" => 36,
        "white" => 37,
        _ => 39,
    };

    let bg = match bg.to_lowercase().as_str() {
        "black" => 40,
        "red" => 41,
        "green" => 42,
        "yellow" => 43,
        "blue" => 44,
        "magenta" => 45,
        "cyan" => 46,
        "white" => 47,
        _ => 49,
    };

    escape(format!("{};{}m", fg, bg).as_str())
}

pub fn inverse() -> String {
    escape("7m")
}

pub fn reset() -> String {
    escape("0m")
}
