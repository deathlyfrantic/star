extern crate clap;
extern crate libc;
extern crate termion;
extern crate termios;
extern crate unicode_width;

mod console;
mod event_loop;
mod line;
mod render;
mod score;

use clap::{App, Arg};

use line::Line;

use std::io::{self, BufRead};
use std::process::exit;

fn actual_program<'a>(initial_search: &'a str, height: usize) {
    let stdin = io::stdin();
    let stdin_lines: Vec<Line> = stdin
        .lock()
        .lines()
        .filter_map(|l| (l.ok()))
        .map(|l| Line::new(l))
        .collect();
    let stdin_lines = Box::new(stdin_lines);

    match event_loop::run(stdin_lines, initial_search, height) {
        Ok(l) => println!("{}", l),
        Err(_) => unsafe {
            println!("{}", termion::clear::CurrentLine);
            libc::killpg(libc::getpgrp(), libc::SIGINT);
            exit(1);
        },
    };
}

fn main() {
    let matches = App::new("star")
        .version("0.1.0")
        .author("Zandr Martin")
        .about("A recreation of Gary Bernhardt's Selecta, written in Rust")
        .arg(
            Arg::with_name("height")
                .short("H")
                .long("height")
                .help("Specify UI height in lines (including prompt)")
                .takes_value(true)
                .default_value("21"),
        ).arg(
            Arg::with_name("search")
                .short("s")
                .long("search")
                .help("Specify an initial search string")
                .takes_value(true)
                .default_value(""),
        ).arg(
            Arg::with_name("version")
                .short("v")
                .long("version")
                .help("Output version information then exit"),
        ).get_matches();

    if matches.is_present("version") {
        println!("0.1.0");
        exit(0);
    }

    let height = match matches.value_of("height") {
        Some(h) => match h.parse::<usize>() {
            Ok(h) => h,
            Err(_) => 21,
        },
        None => 21,
    };

    let search = match matches.value_of("search") {
        Some(s) => s,
        None => "",
    };

    actual_program(search, height);
}
