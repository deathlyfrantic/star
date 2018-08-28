use termion::event::Key;
use termion::input::TermRead;

use console::Console;
use line::Line;
use render::Renderer;
use score::{calculate_score, Score};

use std::cmp::min;
use std::collections::HashMap;
use std::io;
use std::rc::Rc;

fn query_str(query: &[char]) -> String {
    query.iter().collect::<String>()
}

fn get_scores<'a>(
    map: &mut HashMap<String, Rc<Vec<Score<'a>>>>,
    query: &[char],
) -> Rc<Vec<Score<'a>>> {
    if let Some(scores) = &map.get(&query_str(&query)) {
        return Rc::clone(scores);
    }
    let mut tmp = query.to_vec();
    loop {
        tmp.pop();
        if map.contains_key(&query_str(&tmp)) {
            let mut new_scores: Vec<Score> = map
                .get(&query_str(&tmp))
                .unwrap()
                .iter()
                .filter_map(|s| calculate_score(s.line, &query))
                .collect();
            new_scores.sort_unstable_by_key(|score| score.points);
            let new_scores = Rc::new(new_scores);
            map.insert(query_str(&query), Rc::clone(&new_scores));
            return new_scores;
        }
    }
}

pub fn run(
    stdin_lines: Box<Vec<Line>>,
    initial_search: &str,
    height: usize,
) -> Result<String, io::Error> {
    let console = Console::new()?;
    let ref tty = console.tty;
    let mut query: Vec<char> = initial_search.chars().collect();
    let mut need_new_scores = false;
    let mut score_map: HashMap<String, Rc<Vec<Score>>> = HashMap::new();

    if query.len() > 0 {
        // "prime" the cache with the scores for "", since an initial query was specified
        let mut scores: Vec<Score> = stdin_lines
            .iter()
            .filter_map(|l| calculate_score(l, &[]))
            .collect();
        scores.sort_unstable_by_key(|score| score.points);
        score_map.insert("".to_string(), Rc::new(scores));
    }

    let mut scores: Vec<Score> = stdin_lines
        .iter()
        .filter_map(|l| calculate_score(l, &query))
        .collect();
    scores.sort_unstable_by_key(|score| score.points);
    let mut scores = Rc::new(scores);
    score_map.insert(query_str(&query), Rc::clone(&scores));

    let mut renderer = Renderer::new(
        Rc::clone(&scores),
        query_str(&query),
        0,
        format!("{}", stdin_lines.len()).len(),
        min(height, console.height as usize),
        console.width as usize,
    );
    console.write(&renderer.render());

    for c in tty.keys() {
        match c.unwrap() {
            Key::Ctrl('c') | Key::Esc => {
                console.write(&renderer.clear);
                return Err(io::Error::new(io::ErrorKind::Other, "ctrl-c"));
            }
            Key::Char('\n') => {
                console.write(&renderer.clear);
                if scores.len() == 0 {
                    return Ok(String::new());
                }
                return Ok(scores[renderer.selected].line.buf.clone());
            }
            Key::Ctrl('n') | Key::Down => {
                // move selection down
                if renderer.selected < renderer.num_rendered - 1 {
                    renderer.selected += 1;
                    console.write(&renderer.render());
                } else if renderer.selected > 0 {
                    renderer.selected = 0;
                    console.write(&renderer.render());
                }
            }
            Key::Ctrl('p') | Key::Up => {
                // move selection up
                if renderer.selected > 0 {
                    renderer.selected -= 1;
                    console.write(&renderer.render());
                } else if renderer.num_rendered > 0 {
                    renderer.selected = renderer.num_rendered - 1;
                    console.write(&renderer.render());
                }
            }
            Key::Ctrl('w') => {
                // delete word
                need_new_scores = query.len() > 0;
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
            }
            Key::Ctrl('u') => {
                // delete to beginning of line
                need_new_scores = query.len() > 0;
                query.clear();
            }
            Key::Backspace => if let Some(_) = query.pop() {
                need_new_scores = true;
            },
            Key::Char(c) => {
                query.push(c);
                need_new_scores = true;
            }
            _ => {}
        }

        if need_new_scores {
            need_new_scores = false;
            renderer.query = query_str(&query);
            scores = get_scores(&mut score_map, &query);
            renderer.scores = Rc::clone(&scores);
            renderer.selected = min(renderer.selected, scores.len() - 1);
            console.write(&renderer.render());
        }
    }

    // should never get here but the compiler doesn't know that
    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_str() {
        assert_eq!(query_str(&['f', 'o', 'o']), String::from("foo"));
    }

    #[test]
    fn test_get_scores() {
        let lines = vec![Line::from("foo"), Line::from("bar"), Line::from("baz")];
        let mut query: Vec<char> = vec![];
        let mut map: HashMap<String, Rc<Vec<Score>>> = HashMap::new();
        let scores = Rc::new(
            lines
                .iter()
                .filter_map(|l| calculate_score(l, &query))
                .collect(),
        );
        map.insert("".to_string(), Rc::clone(&scores));
        // we should get new scores that were calculated from the "" scores
        query.push('b');
        let result = get_scores(&mut map, &query);
        assert_eq!(result.len(), 2);
        assert_ne!(result, scores);
        // create new scores to associate with the "b" query
        let scores: Rc<Vec<Score>> = Rc::new(
            lines
                .iter()
                .filter_map(|l| calculate_score(l, &query))
                .collect(),
        );
        // there should only be two scores
        assert_eq!(scores.len(), 2);
        map.insert(query_str(&query), Rc::clone(&scores));
        // just make sure we got those scores back with no change in query
        let result = get_scores(&mut map, &query);
        assert_eq!(result, scores);
        // now add a char to the query
        query.push('a');
        // we should get new scores that were calculated from the "b" scores
        let result = get_scores(&mut map, &query);
        assert_eq!(result.len(), 2);
        assert_ne!(result, scores);
        // we should get scores we already calculated, if they exist
        let old_result = result.clone();
        let result = get_scores(&mut map, &query);
        assert_eq!(result, old_result);
    }
}
