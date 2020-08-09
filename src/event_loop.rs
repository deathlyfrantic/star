use crate::{
    color::Colors,
    console::Console,
    line::Line,
    render::{Renderer, RendererConfig},
    score::{calculate_score, Score},
};
use rayon::prelude::*;
use std::{cmp::min, collections::HashMap, io, rc::Rc};
use termion::{event::Key, input::TermRead};

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
                .par_iter()
                .filter_map(|s| calculate_score(s.line, &query))
                .collect();
            new_scores.sort_unstable_by(Score::cmp);
            let new_scores = Rc::new(new_scores);
            map.insert(query_str(&query), Rc::clone(&new_scores));
            return new_scores;
        }
    }
}

pub fn run(
    stdin_lines: Vec<Line>,
    initial_search: &str,
    height: usize,
    colors: (Colors, Colors),
    multiple: bool,
) -> io::Result<String> {
    let console = Console::new()?;
    let tty = &console.tty;
    let mut query: Vec<char> = initial_search.chars().collect();
    let mut need_new_scores = false;
    let mut score_map: HashMap<String, Rc<Vec<Score>>> = HashMap::new();
    let mut tagged: Vec<usize> = vec![];

    if !query.is_empty() {
        // "prime" the cache with the scores for "", since an initial query was specified
        let mut scores: Vec<Score> = stdin_lines
            .par_iter()
            .filter_map(|l| calculate_score(l, &[]))
            .collect();
        scores.sort_unstable_by(Score::cmp);
        score_map.insert("".to_string(), Rc::new(scores));
    }

    let mut scores: Vec<Score> = stdin_lines
        .par_iter()
        .filter_map(|l| calculate_score(l, &query))
        .collect();
    scores.sort_unstable_by(Score::cmp);
    let mut scores = Rc::new(scores);
    score_map.insert(query_str(&query), Rc::clone(&scores));

    let renderer_config = RendererConfig {
        width: console.width as usize,
        height: min(height, console.height as usize),
        fg: &colors.0,
        bg: &colors.1,
        match_count_length: format!("{}", stdin_lines.len()).len(),
    };

    let render = |scores: Rc<Vec<Score>>, query: &[char], selected: usize, tagged: &[usize]| {
        console.write(
            &Renderer::new(
                &renderer_config,
                scores,
                query_str(&query),
                selected,
                tagged,
            )
            .render(),
        )
    };

    let num_visible = |scores: &[Score]| min(renderer_config.height - 1, scores.len());
    let mut selected = 0;
    render(Rc::clone(&scores), &query, selected, &tagged);

    for c in tty.keys() {
        match c.unwrap() {
            Key::Ctrl('c') | Key::Esc => {
                console.write(&Renderer::clear());
                return Err(io::Error::new(io::ErrorKind::Other, ""));
            }
            Key::Char('\n') => {
                console.write(&Renderer::clear());
                if scores.is_empty() {
                    return Ok(String::new());
                }
                return Ok(scores[selected].line.buf.clone());
            }
            Key::Alt('\r') => {
                console.write(&Renderer::clear());
                if scores.is_empty() {
                    return Ok(String::new());
                }
                if tagged.is_empty() || !multiple {
                    return Ok(scores[selected].line.buf.clone());
                }
                return Ok(tagged
                    .iter()
                    .map(|i| stdin_lines[*i].buf.clone())
                    .collect::<Vec<String>>()
                    .join("\n"));
            }
            Key::Char('\t') => {
                if multiple {
                    let line = scores[selected].line;
                    if tagged.contains(&line.index) {
                        if let Ok(index) = tagged.binary_search(&line.index) {
                            tagged.remove(index);
                        }
                    } else {
                        tagged.push(line.index);
                    }
                    render(Rc::clone(&scores), &query, selected, &tagged);
                } else {
                    query.push('\t');
                    need_new_scores = true;
                }
            }
            Key::Ctrl('n') | Key::Down => {
                // move selection down
                if selected < num_visible(&scores) - 1 {
                    selected += 1;
                    render(Rc::clone(&scores), &query, selected, &tagged);
                } else if selected > 0 {
                    selected = 0;
                    render(Rc::clone(&scores), &query, selected, &tagged);
                }
            }
            Key::Ctrl('p') | Key::Up => {
                // move selection up
                if selected > 0 {
                    selected -= 1;
                    render(Rc::clone(&scores), &query, selected, &tagged);
                } else if num_visible(&scores) > 0 {
                    selected = num_visible(&scores) - 1;
                    render(Rc::clone(&scores), &query, selected, &tagged);
                }
            }
            Key::Ctrl('w') => {
                // delete word
                need_new_scores = !query.is_empty();
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
                need_new_scores = !query.is_empty();
                query.clear();
            }
            Key::Backspace => {
                if query.pop().is_some() {
                    need_new_scores = true;
                }
            }
            Key::Char(c) => {
                query.push(c);
                need_new_scores = true;
            }
            _ => {}
        }

        if need_new_scores {
            need_new_scores = false;
            scores = get_scores(&mut score_map, &query);
            if scores.len() > 0 {
                selected = min(selected, scores.len() - 1);
            }
            render(Rc::clone(&scores), &query, selected, &tagged);
        }
    }

    unreachable!();
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
