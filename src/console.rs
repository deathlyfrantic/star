use libc::{c_ushort, ioctl, TIOCGWINSZ};
use termion::{self, cursor};
use termios::{cfmakeraw, tcsetattr, Termios, ECHO, ICANON, TCSANOW};

use std::fs::File;
use std::io::{self, Write};
use std::mem;
use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Debug)]
pub struct Console {
    pub width: u16,
    pub height: u16,
    original_state: Termios,
    pub tty: File,
}

impl Console {
    pub fn new() -> Result<Console, io::Error> {
        let tty = termion::get_tty()?;
        let (width, height) = terminal_size(tty.as_raw_fd())?;
        let mut termios = Termios::from_fd(tty.as_raw_fd())?;
        let original_state = termios.clone();

        cfmakeraw(&mut termios);
        termios.c_lflag &= !(ECHO | ICANON);
        tcsetattr(tty.as_raw_fd(), TCSANOW, &mut termios)?;

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

#[repr(C)]
struct TermSize {
    row: c_ushort,
    col: c_ushort,
    _x: c_ushort,
    _y: c_ushort,
}

fn terminal_size(fd: RawFd) -> io::Result<(u16, u16)> {
    unsafe {
        let mut size: TermSize = mem::zeroed();
        if ioctl(fd, TIOCGWINSZ, &mut size as *mut _) == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok((size.col as u16, size.row as u16))
        }
    }
}
