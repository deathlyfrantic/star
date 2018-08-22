extern crate libc;
extern crate regex;
extern crate termion;
extern crate termios;

mod console;
mod event_loop;
mod line;
mod render;
mod score;

use line::Line;

use std::io::{self, BufRead};
use std::process::exit;

fn main() {
    let stdin = io::stdin();
    let stdin_lines: Vec<Line> = stdin
        .lock()
        .lines()
        .filter_map(|l| (l.ok()))
        .map(|l| Line::new(l))
        .collect();
    let stdin_lines = Box::new(stdin_lines);

    match event_loop::run(stdin_lines) {
        Ok(l) => println!("{}", l),
        Err(_) => unsafe {
            println!("{}", termion::clear::CurrentLine);
            libc::killpg(libc::getpgrp(), libc::SIGINT);
            exit(1);
        },
    };
}
