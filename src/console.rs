use termion::{self, cursor};

use std::io::Write;
use std::process::Command;

#[derive(Debug)]
pub struct Console {
    pub width: u16,
    pub height: u16,
    original_state: String,
}

impl Console {
    pub fn new<'a>() -> Result<Console, &'a str> {
        if !termion::is_tty(&termion::get_tty().unwrap()) {
            return Err("not a TTY");
        }

        let (width, height) = match termion::terminal_size() {
            Ok(size) => size,
            Err(_) => return Err("not a TTY"),
        };

        let original_state = match stty(&["-g"]) {
            Some(s) => s,
            None => "".to_string(),
        };

        stty(&["raw", "-echo", "-icanon"]);

        Ok(Console {
            width: width,
            height: height,
            original_state: original_state,
        })
    }

    pub fn write(&self, buf: &str) {
        let mut tty = termion::get_tty().unwrap();
        let _ = write!(tty, "{}", cursor::Hide);
        let _ = write!(tty, "{}", buf);
        let _ = write!(tty, "{}", cursor::Show);
        let _ = tty.flush();
    }

    pub fn write_lines(&self, lines: Vec<String>) {
        self.write(lines.join("\r\n").as_str());
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        stty(&[self.original_state.as_str()]);
    }
}

fn stty(args: &[&str]) -> Option<String> {
    let output = Command::new("stty")
        .stdin(termion::get_tty().unwrap())
        .args(args)
        .output()
        .expect(
            format!("failed to execute process (stty {}) ", args.join(" "))
                .as_str(),
        );

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}
