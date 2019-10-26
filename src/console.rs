use std::{
    fs::File,
    io::{self, Write},
    os::unix::io::AsRawFd,
};
use termion::{self, cursor, terminal_size};
use termios::{cfmakeraw, tcsetattr, Termios, ECHO, ICANON, TCSANOW};

#[derive(Debug)]
pub struct Console {
    pub width: u16,
    pub height: u16,
    original_state: Termios,
    pub tty: File,
}

impl Console {
    pub fn new() -> io::Result<Self> {
        let tty = termion::get_tty()?;
        let (width, height) = terminal_size()?;
        let mut termios = Termios::from_fd(tty.as_raw_fd())?;
        let original_state = termios;

        cfmakeraw(&mut termios);
        termios.c_lflag &= !(ECHO | ICANON);
        tcsetattr(tty.as_raw_fd(), TCSANOW, &termios)?;

        Ok(Self {
            width,
            height,
            original_state,
            tty,
        })
    }

    pub fn write(&self, buf: &str) {
        let mut tty = termion::get_tty().unwrap();
        write!(tty, "{}{}{}", cursor::Hide, buf, cursor::Show).unwrap();
        tty.flush().unwrap();
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        tcsetattr(self.tty.as_raw_fd(), TCSANOW, &self.original_state).unwrap();
    }
}
