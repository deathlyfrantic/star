extern crate libc;
extern crate regex;
extern crate termion;

use termion::event::Key;
use termion::input::TermRead;

mod score;
mod console;
mod render;

use render::Renderer;

use std::io::{self, BufRead};
use std::process::exit;

fn get_scores<'a>(
    lines: &'a Vec<String>,
    query: &[char],
) -> Vec<score::Score<'a>> {
    lines
        .iter()
        .filter_map(|line| score::calculate_score(line, query))
        .collect()
}

fn run<'a>(stdin_lines: Vec<String>) -> Result<String, &'a str> {
    let console = console::Console::new()?;
    let mut query: Vec<char> = vec![];
    let tty = termion::get_tty().unwrap();
    let mut dirty = false;
    let mut selected = 0;

    let scores = get_scores(&stdin_lines, &query);

    let renderer = Renderer::new(&scores, &console, "", selected);
    renderer.render();

    for c in tty.keys() {
        match c.unwrap() {
            Key::Ctrl('c') | Key::Esc => return Err("ctrl-c"),
            Key::Ctrl('n') | Key::Down => {
                // move selection down
                if selected < 20 {
                    selected += 1;
                    dirty = true;
                }
            }
            Key::Ctrl('p') | Key::Up => {
                // move selection up
                if selected > 0 {
                    selected -= 1;
                    dirty = true;
                }
            }
            Key::Ctrl('w') => {
                // delete word
            }
            Key::Ctrl('u') => {
                query.clear();
                dirty = true;
            }
            Key::Backspace => {
                if query.len() > 0 {
                    query.pop();
                    dirty = true;
                }
            }
            Key::Char('\n') => {
                let rv = String::from(scores[selected].line);
                renderer.clear();
                return Ok(rv);
            }
            Key::Char(c) => {
                query.push(c);
                dirty = true;
            }
            _ => {}
        }

        if dirty {
            let scores = get_scores(&stdin_lines, &query);
            let query_str = query.iter().collect::<String>();
            let renderer =
                Renderer::new(&scores, &console, query_str.as_str(), selected);
            renderer.render();
            dirty = false;
        }
    }

    Ok("foo".to_string())
}

fn main() {
    let stdin = io::stdin();
    let stdin_lines: Vec<String> =
        stdin.lock().lines().filter_map(|line| line.ok()).collect();

    match run(stdin_lines) {
        Ok(l) => println!("{}", l),
        Err(_) => unsafe {
            println!("!ctrl-c!");
            libc::killpg(libc::getpgrp(), libc::SIGINT);
            exit(1);
        },
    };
}
