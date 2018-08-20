use termion::{self, cursor};
use termios::{cfmakeraw, tcsetattr, Termios, ECHO, ICANON, TCSANOW};

use std::fs::File;
use std::io::Write;
use std::os::unix::io::AsRawFd;

#[derive(Debug)]
pub struct Console {
    pub width: u16,
    pub height: u16,
    original_state: Termios,
    tty: File,
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

        let tty = termion::get_tty().unwrap();
        let mut termios = Termios::from_fd(tty.as_raw_fd()).unwrap();
        let original_state = termios.clone();

        cfmakeraw(&mut termios);
        termios.c_lflag &= !(ECHO | ICANON);
        tcsetattr(tty.as_raw_fd(), TCSANOW, &mut termios).unwrap();

        Ok(Console {
            width: width,
            height: height,
            original_state: original_state,
            tty: tty,
        })
    }

    pub fn write(&self, buf: &str) {
        let mut tty = termion::get_tty().unwrap();
        write!(tty, "{}{}{}", cursor::Hide, buf, cursor::Show).unwrap();
        tty.flush().unwrap();
    }

    pub fn write_lines(&self, lines: Vec<String>) {
        self.write(lines.join("\r\n").as_str());
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        tcsetattr(self.tty.as_raw_fd(), TCSANOW, &mut self.original_state).unwrap();
    }
}
