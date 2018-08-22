extern crate libc;
extern crate regex;
extern crate termion;
extern crate termios;

use termion::event::Key;
use termion::input::TermRead;

mod console;
mod line;
mod render;
mod score;

use line::Line;
use render::Renderer;

use std::collections::HashMap;
use std::io::{self, BufRead};
use std::process::exit;
use std::rc::Rc;

fn query_str(query: &[char]) -> String {
    query.iter().collect::<String>()
}

fn run<'a>(stdin_lines: Box<Vec<Line>>) -> Result<String, io::Error> {
    let console = console::Console::new()?;
    let mut query: Vec<char> = vec![];
    let tty = termion::get_tty()?;
    let mut new_scores = false;
    let mut score_map: HashMap<String, Rc<Vec<score::Score>>> = HashMap::new();
    let mut scores = Rc::new(
        stdin_lines
            .iter()
            .filter_map(|l| score::calculate_score(l, &query))
            .collect(),
    );
    score_map.insert("".to_string(), Rc::clone(&scores));

    let mut renderer = Renderer::new(Rc::clone(&scores), &console, "".to_string(), 0);
    renderer.render();

    for c in tty.keys() {
        match c.unwrap() {
            Key::Ctrl('c') | Key::Esc => {
                renderer.clear();
                return Err(io::Error::new(io::ErrorKind::Other, "ctrl-c"));
            }
            Key::Char('\n') => {
                renderer.clear();
                return Ok(scores[renderer.selected].line.buf.clone());
            }
            Key::Ctrl('n') | Key::Down => {
                // move selection down
                if renderer.selected < renderer.num_rendered - 1 {
                    renderer.selected += 1;
                    renderer.render();
                } else if renderer.selected > 0 {
                    renderer.selected = 0;
                    renderer.render();
                }
            }
            Key::Ctrl('p') | Key::Up => {
                // move selection up
                if renderer.selected > 0 {
                    renderer.selected -= 1;
                    renderer.render();
                } else if renderer.num_rendered > 0 {
                    renderer.selected = renderer.num_rendered - 1;
                    renderer.render()
                }
            }
            Key::Ctrl('w') => {
                // delete word
                new_scores = true;
                let mut saw_nonspace = false;
                while let Some(c) = query.pop() {
                    if c.is_whitespace() {
                        if saw_nonspace {
                            query.push(c);
                            break;
                        }
                    } else if !saw_nonspace {
                        saw_nonspace = true;
                    }
                }
                renderer.scores = Rc::clone(&score_map.get(&query_str(&query)).unwrap());
            }
            Key::Ctrl('u') => {
                // delete to beginning of line
                query.clear();
                scores = Rc::clone(&score_map.get("").unwrap());
                renderer.scores = Rc::clone(&scores);
                new_scores = true;
            }
            Key::Backspace => if let Some(_) = query.pop() {
                renderer.scores = Rc::clone(&score_map.get(&query_str(&query)).unwrap());
                new_scores = true;
            },
            Key::Char(c) => {
                query.push(c);
                if score_map.contains_key(&query_str(&query)) {
                    renderer.scores = Rc::clone(&score_map.get(&query_str(&query)).unwrap());
                } else {
                    scores = Rc::new(
                        scores
                            .iter()
                            .filter_map(|s| score::calculate_score(s.line, &query))
                            .collect(),
                    );
                    score_map.insert(query_str(&query), Rc::clone(&scores));
                    renderer.scores = Rc::clone(&scores);
                }
                new_scores = true;
            }
            _ => {}
        }

        if new_scores {
            new_scores = false;
            renderer.query = query_str(&query);
            renderer.render();
        }
    }

    // should never get here but the compiler doesn't know that
    Ok("".to_string())
}

fn main() {
    let stdin = io::stdin();
    let stdin_lines: Vec<Line> = stdin
        .lock()
        .lines()
        .filter_map(|l| (l.ok()))
        .map(|l| Line::new(l))
        .collect();
    let stdin_lines = Box::new(stdin_lines);

    match run(stdin_lines) {
        Ok(l) => println!("{}", l),
        Err(_) => unsafe {
            println!("{}", termion::clear::CurrentLine);
            libc::killpg(libc::getpgrp(), libc::SIGINT);
            exit(1);
        },
    };
}
