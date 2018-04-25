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
use std::collections::HashMap;
use std::rc::Rc;
// use std::env;

fn get_scores<'a>(
    lines: Rc<Vec<score::Score<'a>>>,
    query: &[char],
) -> Rc<Vec<score::Score<'a>>> {
    Rc::new(
        lines
            .iter()
            .filter_map(|s| score::calculate_score(s.line, query))
            .collect(),
    )
}

fn run<'a>(stdin_lines: Box<Vec<String>>) -> Result<String, &'a str> {
    let console = console::Console::new()?;
    let mut query: Vec<char> = vec![];
    let tty = termion::get_tty().unwrap();
    let mut new_scores = false;
    let mut score_map: HashMap<Vec<char>, Rc<Vec<score::Score>>> =
        HashMap::new();
    let mut scores = Rc::new(
        stdin_lines
            .iter()
            .filter_map(|line| score::calculate_score(line, &query))
            .collect(),
    );
    score_map.insert(vec![], Rc::clone(&scores));

    let mut renderer =
        Renderer::new(Rc::clone(&scores), &console, "".to_string(), 0);
    renderer.render();

    for c in tty.keys() {
        match c.unwrap() {
            Key::Ctrl('c') | Key::Esc => {
                renderer.clear();
                return Err("ctrl-c");
            }
            Key::Char('\n') => {
                renderer.clear();
                return Ok(String::from(scores[renderer.selected].line));
            }
            Key::Ctrl('n') | Key::Down => {
                // move selection down
                if renderer.selected < 18 {
                    renderer.selected += 1;
                    renderer.render();
                }
            }
            Key::Ctrl('p') | Key::Up => {
                // move selection up
                if renderer.selected > 0 {
                    renderer.selected -= 1;
                    renderer.render();
                }
            }
            Key::Ctrl('w') => {
                // delete word
                new_scores = true;
                let mut saw_nonspace = false;
                while query.len() > 0 {
                    let c = query.pop().unwrap();
                    if c.is_whitespace() {
                        if saw_nonspace {
                            query.push(c);
                            break;
                        }
                    } else if !saw_nonspace {
                        saw_nonspace = true;
                    }
                }
                renderer.scores = Rc::clone(&score_map.get(&query).unwrap());
            }
            Key::Ctrl('u') => {
                // delete to beginning of line
                query.clear();
                renderer.scores = Rc::clone(&score_map.get(&query).unwrap());
                new_scores = true;
            }
            Key::Backspace => {
                if query.len() > 0 {
                    query.pop();
                    renderer.scores =
                        Rc::clone(&score_map.get(&query).unwrap());
                    new_scores = true;
                }
            }
            Key::Char(c) => {
                query.push(c);
                if score_map.contains_key(&query) {
                    renderer.scores =
                        Rc::clone(&score_map.get(&query).unwrap());
                } else {
                    scores = get_scores(scores, &query);
                    score_map.insert(query.clone(), Rc::clone(&scores));
                    renderer.scores = Rc::clone(&scores);
                }
                new_scores = true;
            }
            _ => {}
        }

        if new_scores {
            new_scores = false;
            renderer.query = query.iter().collect::<String>();
            renderer.render();
        }
    }

    // should never get here but the compiler doesn't know that
    Ok("".to_string())
}

fn main() {
    let stdin = io::stdin();
    let stdin_lines: Vec<String> =
        stdin.lock().lines().filter_map(|line| line.ok()).collect();
    let stdin_lines = Box::new(stdin_lines);

    // let args: Vec<String> = env::args().collect();
    // let search = if args.len() > 1 {
    //     args[1].clone()
    // } else {
    //     "foobar".to_string()
    // };
    // let query = search.chars().collect::<Vec<char>>();
    // let scores: Vec<score::Score> = stdin_lines
    //     .iter()
    //     .filter_map(|line| score::calculate_score(line, &query))
    //     .collect();
    // println!("scores length: {}", scores.len());

    match run(stdin_lines) {
        Ok(l) => println!("{}", l),
        Err(_) => unsafe {
            println!("{}", termion::clear::CurrentLine);
            libc::killpg(libc::getpgrp(), libc::SIGINT);
            exit(1);
        },
    };
}
