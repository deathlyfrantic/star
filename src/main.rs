mod color;
mod console;
mod event_loop;
mod line;
mod render;
mod score;

use crate::line::Line;
use clap::{App, Arg};
use color::{get_colors, Colors};
use libc;
use std::{
    io::{self, BufRead, Error, ErrorKind},
    process::exit,
};
use termion;

fn error_exit(err: Error) {
    unsafe {
        println!("{}", termion::clear::CurrentLine);
        eprintln!("{}", err);
        libc::killpg(libc::getpgrp(), libc::SIGINT);
        exit(1);
    }
}

fn run(initial_search: &str, height: usize, colors: (Colors, Colors)) {
    let stdin_lines: Box<Vec<Line>> = Box::new(
        io::stdin()
            .lock()
            .lines()
            .filter_map(|l| l.ok())
            .map(|l| Line::new(l))
            .collect(),
    );
    match event_loop::run(stdin_lines, initial_search, height, colors) {
        Ok(l) => println!("{}", l),
        Err(e) => error_exit(e),
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
        )
        .arg(
            Arg::with_name("search")
                .short("s")
                .long("search")
                .help("Specify an initial search string")
                .takes_value(true)
                .default_value("")
                .hide_default_value(true),
        )
        .arg(
            Arg::with_name("color-normal-fg")
                .long("color-normal-fg")
                .help("Foreground color of normal text")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("color-normal-bg")
                .long("color-normal-bg")
                .help("Background color of normal text")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("color-matched-fg")
                .long("color-matched-fg")
                .help("Foreground color of matched text")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("color-matched-bg")
                .long("color-matched-bg")
                .help("Background color of matched text")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("color-selected-fg")
                .long("color-selected-fg")
                .help("Foreground color of selected line")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("color-selected-bg")
                .long("color-selected-bg")
                .help("Background color of selected line")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("color-matched-selected-fg")
                .long("color-matched-selected-fg")
                .help("Foreground color of matched text on selected line")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("color-matched-selected-bg")
                .long("color-matched-selected-bg")
                .help("Background color of matched text on selected line")
                .takes_value(true),
        )
        .get_matches();
    let height = match matches.value_of("height") {
        Some(h) => match h.parse::<usize>() {
            Ok(h) => h,
            Err(_) => {
                return error_exit(Error::new(
                    ErrorKind::InvalidInput,
                    format!("invalid height specification: \"{}\"", h),
                ));
            }
        },
        None => 21,
    };
    let search = match matches.value_of("search") {
        Some(s) => s,
        None => "",
    };
    let colors = match get_colors(&matches) {
        Ok(c) => c,
        Err(e) => return error_exit(e),
    };
    run(search, height, colors);
}
