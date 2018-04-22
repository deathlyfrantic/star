extern crate isatty;
extern crate termsize;

use std;
use std::fs;
use std::io::prelude::*;
use std::process::Command;

use ansi;

#[derive(Debug)]
pub struct Console {
    pub width: u16,
    pub height: u16,
    original_state: String,
}

impl Console {
    pub fn new<'a>() -> Result<Console, &'a str> {
        if !isatty::stdout_isatty() {
            return Err("not a TTY");
        }

        let (width, height) = match termsize::get() {
            Some(size) => (size.cols, size.rows),
            None => (80, 25),
        };

        Ok(Console {
            width: width,
            height: height,
            original_state: match stty(&["-g"]) {
                Some(output) => output.trim_right_matches("\n").to_string(),
                None => "".to_string(),
            },
        })
    }

    pub fn restore(&self) -> Option<String> {
        stty(&[self.original_state.as_str()])
    }

    pub fn configure(&self) -> Option<String> {
        stty(&["-echo", "-icanon"])
    }

    pub fn write(&self, buf: &[u8]) {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open("/dev/tty")
            .expect("unable to open /dev/tty");
        let _ = f.write_all(ansi::hide_cursor().as_bytes());
        let _ = f.write_all(buf);
        let _ = f.write_all(ansi::show_cursor().as_bytes());
    }

    pub fn write_lines(&self, lines: Vec<String>) {
        let mut buf = lines.join("\n");
        buf.push_str("\n");
        self.write(buf.as_bytes());
    }
}

fn stty(args: &[&str]) -> Option<String> {
    let output = Command::new("stty")
        .stdin(
            fs::OpenOptions::new()
                .write(true)
                .read(true)
                .open("/dev/tty")
                .expect("unable to open /dev/tty"),
        )
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
