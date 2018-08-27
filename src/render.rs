use termion::{clear, color, cursor, style};

use console;
use score::Score;

use std::cmp::min;
use std::rc::Rc;

pub struct Renderer<'a> {
    pub scores: Rc<Vec<Score<'a>>>,
    console: &'a console::Console,
    pub query: String,
    pub selected: usize,
    pub num_rendered: usize,
    match_count_length: usize,
    height: usize,
}

impl<'a> Renderer<'a> {
    pub fn new(
        scores: Rc<Vec<Score<'a>>>,
        console: &'a console::Console,
        query: String,
        selected: usize,
        match_count_length: usize,
        height: usize,
    ) -> Renderer<'a> {
        Renderer {
            scores: scores,
            console: console,
            query: query,
            selected: selected,
            num_rendered: 0,
            match_count_length: match_count_length,
            height: min(height, console.height as usize),
        }
    }

    fn render_search_line(&self, num_scores: usize) -> String {
        format!(
            "{:>width$} > {}{}",
            num_scores,
            self.query,
            clear::UntilNewline,
            width = self.match_count_length
        )
    }

    fn highlight_line(&self, score: &Score, selected: bool) -> String {
        // this function highlights matches, expands tabs, and truncates lines to width
        let mut visible_chars = 0;
        let mut rv = format!("{}", color::Fg(color::Reset));
        if selected {
            rv.push_str(&format!("{}", style::Invert));
        }
        for (i, c) in score.line.buf.chars().enumerate() {
            if score.first != score.last {
                if score.first == i {
                    rv.push_str(&format!("{}", color::Fg(color::Red)));
                } else if score.last == i {
                    rv.push_str(&format!("{}", color::Fg(color::Reset)));
                }
            }
            if c == '\t' {
                loop {
                    rv.push(' ');
                    visible_chars += 1;
                    if visible_chars % 8 == 0 || visible_chars >= self.console.width {
                        break;
                    }
                }
            } else if self.console.width > visible_chars {
                rv.push(c);
                visible_chars += 1;
            }
            if self.console.width <= visible_chars {
                break;
            }
        }
        rv.push_str(&format!("{}{}", style::Reset, clear::UntilNewline));
        rv
    }

    pub fn render_lines(&mut self) -> Vec<String> {
        let mut lines: Vec<String> = vec!["".to_string()]; // to account for search line
        let num_matches = min((self.height - 1) as usize, self.scores.len());
        self.num_rendered = num_matches;

        for (i, score) in self.scores.iter().enumerate().take(num_matches) {
            lines.push(self.highlight_line(score, self.selected == i));
        }

        lines
    }

    pub fn render(&mut self) {
        let lines = self.render_lines();
        self.console.write_lines(lines);
        self.console.write(&format!("{}", clear::AfterCursor));
        if self.num_rendered > 0 {
            self.console
                .write(&format!("{}", cursor::Up(self.num_rendered as u16)));
        }
        self.console.write("\r");
        self.console
            .write(self.render_search_line(self.scores.len()).as_str());
    }

    pub fn clear(&mut self) {
        self.console.write(
            format!("{}\r\n", clear::CurrentLine)
                .repeat(self.num_rendered)
                .as_str(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
