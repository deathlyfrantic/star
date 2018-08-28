use termion::{clear, color, cursor, style};
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

use score::Score;

use std::cmp::min;
use std::rc::Rc;

pub struct Renderer<'a> {
    pub scores: Rc<Vec<Score<'a>>>,
    pub query: String,
    pub selected: usize,
    pub num_rendered: usize,
    match_count_length: usize,
    height: usize,
    width: usize,
    pub clear: String,
}

impl<'a> Renderer<'a> {
    pub fn new(
        scores: Rc<Vec<Score<'a>>>,
        query: String,
        selected: usize,
        match_count_length: usize,
        height: usize,
        width: usize,
    ) -> Renderer<'a> {
        Renderer {
            scores: scores,
            query: query,
            selected: selected,
            num_rendered: 0,
            match_count_length: match_count_length,
            height: height,
            width: width,
            clear: format!("\r{}", clear::AfterCursor),
        }
    }

    fn render_search_line(&self, num_scores: usize) -> String {
        let line = format!(
            "{:>width$} > {}",
            num_scores,
            self.query,
            width = self.match_count_length
        );
        let mut rv = String::with_capacity(line.len());
        for char in line.chars() {
            if rv.width() < self.width {
                rv.push(char);
            }
        }
        rv.push_str(&format!("{}", clear::UntilNewline));
        rv
    }

    fn highlight_line(&self, score: &Score, selected: bool) -> String {
        // this function highlights matches, expands tabs, and truncates lines to width
        let mut visible_chars: usize = 0;
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
                    if visible_chars % 8 == 0 || visible_chars >= self.width {
                        break;
                    }
                }
            } else if self.width > visible_chars {
                rv.push(c);
                visible_chars += match c.width() {
                    Some(w) => w,
                    None => 0,
                };
            }
            if self.width <= visible_chars {
                break;
            }
        }
        rv.push_str(&format!("{}{}", style::Reset, clear::UntilNewline));
        rv
    }

    fn render_lines(&mut self) -> Vec<String> {
        let mut lines: Vec<String> = vec!["".to_string()]; // to account for search line
        let num_matches = min((self.height - 1) as usize, self.scores.len());
        self.num_rendered = num_matches;

        for (i, score) in self.scores.iter().enumerate().take(num_matches) {
            lines.push(self.highlight_line(score, self.selected == i));
        }

        lines
    }

    pub fn render(&mut self) -> String {
        let mut output = self.render_lines().join("\r\n");
        output.push_str(&format!("{}", clear::AfterCursor));
        if self.num_rendered > 0 {
            output.push_str(&format!("{}", cursor::Up(self.num_rendered as u16)));
        }
        output.push_str("\r");
        output.push_str(&self.render_search_line(self.scores.len()));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
