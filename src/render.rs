use crate::{color::Colors, score::Score};
use std::{cmp::min, rc::Rc};
use termion::{clear, color, cursor, style};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct Renderer<'a> {
    scores: Rc<Vec<Score<'a>>>,
    query: String,
    selected: usize,
    height: usize,
    width: usize,
    fg: &'a Colors,
    bg: &'a Colors,
    match_count_length: usize,
}

pub struct RendererConfig<'a> {
    pub width: usize,
    pub height: usize,
    pub fg: &'a Colors,
    pub bg: &'a Colors,
    pub match_count_length: usize,
}

impl<'a> Renderer<'a> {
    pub fn new(
        config: &'a RendererConfig,
        scores: Rc<Vec<Score<'a>>>,
        query: String,
        selected: usize,
    ) -> Self {
        Self {
            scores,
            query,
            selected,
            match_count_length: config.match_count_length,
            fg: config.fg,
            bg: config.bg,
            width: config.width,
            height: config.height,
        }
    }

    fn num_visible(&self) -> usize {
        min((self.height - 1) as usize, self.scores.len())
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
        let mut rv = format!("{}{}", self.fg.normal, self.bg.normal);
        if selected {
            rv.push_str(&format!("{}{}", self.fg.selected, self.bg.selected));
        }
        for (i, c) in score.line.buf.chars().enumerate() {
            if score.first != score.last {
                if score.first == i {
                    if selected {
                        rv.push_str(&format!(
                            "{}{}",
                            self.fg.matched_selected, self.bg.matched_selected
                        ));
                    } else {
                        rv.push_str(&format!("{}{}", self.fg.matched, self.bg.matched));
                    }
                } else if score.last == i {
                    if selected {
                        rv.push_str(&format!("{}{}", self.fg.selected, self.bg.selected));
                    } else {
                        rv.push_str(&format!("{}{}", self.fg.normal, self.bg.normal));
                    }
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
        rv.push_str(&format!(
            "{}{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            style::Reset,
            clear::UntilNewline
        ));
        rv
    }

    fn render_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = vec!["".to_string()]; // to account for search line
        for (i, score) in self.scores.iter().enumerate().take(self.num_visible()) {
            lines.push(self.highlight_line(score, self.selected == i));
        }
        lines
    }

    pub fn render(&self) -> String {
        let mut output = self.render_lines().join("\r\n");
        output.push_str(&format!("{}", clear::AfterCursor));
        if self.num_visible() > 0 {
            output.push_str(&format!("{}", cursor::Up(self.num_visible() as u16)));
        }
        output.push_str("\r");
        output.push_str(&self.render_search_line(self.scores.len()));
        output
    }

    pub fn clear() -> String {
        format!("\r{}", clear::AfterCursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::get_colors;
    use crate::line::Line;
    use crate::score::calculate_score;
    use clap;

    fn colors() -> (Colors, Colors) {
        let matches = clap::ArgMatches::new();
        get_colors(&matches).unwrap()
    }

    #[test]
    fn test_render_search_line() {
        let mut r = Renderer::new(
            Rc::new(vec![]),
            String::from("foobar"),
            5,
            colors(),
            (20, 20),
        );
        let expected = format!("12345 > foobar{}", clear::UntilNewline);
        assert_eq!(r.render_search_line(12345), expected);

        // test score number justification
        let expected = format!("  123 > foobar{}", clear::UntilNewline);
        assert_eq!(r.render_search_line(123), expected);

        // test line is truncated if necessary
        r.width = 11;
        let expected = format!("  123 > foo{}", clear::UntilNewline);
        assert_eq!(r.render_search_line(123), expected);
    }

    #[test]
    fn test_highlight_line() {
        let mut r = Renderer::new(
            Rc::new(vec![]),
            String::from("foobar"),
            5,
            colors(),
            (20, 20),
        );
        let line = Line::from("foobarbaz");
        let score = calculate_score(&line, &['b', 'a', 'r']).unwrap();
        let expected = format!(
            "{}{}foo{}bar{}{}baz{}{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            color::Fg(color::Red),
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            style::Reset,
            clear::UntilNewline
        );
        assert_eq!(r.highlight_line(&score, false), expected);

        // test highlighting the selected line
        let expected = format!(
            "{}{}{}{}foo{}bar{}{}baz{}{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            color::Fg(color::Reset),
            style::Invert,
            color::Fg(color::Red),
            color::Fg(color::Reset),
            style::Invert,
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            style::Reset,
            clear::UntilNewline
        );
        assert_eq!(r.highlight_line(&score, true), expected);

        // test truncation
        r.width = 7;
        let expected = format!(
            "{}{}foo{}bar{}{}b{}{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            color::Fg(color::Red),
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            style::Reset,
            clear::UntilNewline
        );
        assert_eq!(r.highlight_line(&score, false), expected);

        // test tab expansion
        r.width = 100;
        let line = Line::from("f\too\tbar");
        let score = calculate_score(&line, &['b', 'a', 'r']).unwrap();
        let expected = format!(
            "{}{}f       oo      {}bar{}{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            color::Fg(color::Red),
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            style::Reset,
            clear::UntilNewline
        );
        r.width = 20;
        assert_eq!(r.highlight_line(&score, false), expected);

        // test tab expansion and truncation
        r.width = 4;
        let line = Line::from("foo\tbar");
        let score = calculate_score(&line, &['b', 'a', 'r']).unwrap();
        let expected = format!(
            "{}{}foo {}{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            style::Reset,
            clear::UntilNewline
        );
        assert_eq!(r.highlight_line(&score, false), expected);
    }
}
