use termion::{clear, color, cursor, style};
use regex::{Captures, Regex};

use score::Score;
use console;

use std::rc::Rc;
use std::cmp::min;

pub struct Renderer<'a> {
    pub scores: Rc<Vec<Score<'a>>>,
    console: &'a console::Console,
    pub query: String,
    pub selected: usize,
    pub num_rendered: usize,
    match_count_length: usize,
}

impl<'a> Renderer<'a> {
    pub fn new(
        scores: Rc<Vec<Score<'a>>>,
        console: &'a console::Console,
        query: String,
        selected: usize,
    ) -> Renderer<'a> {
        let scores_len = scores.len();
        Renderer {
            scores: scores,
            console: console,
            query: query,
            selected: selected,
            num_rendered: 0,
            match_count_length: format!("{}", scores_len).len(),
        }
    }

    fn render_search_line(&self, num_scores: usize) -> String {
        format!(
            "{:>width$} > {}",
            num_scores,
            self.query,
            width = self.match_count_length
        )
    }

    pub fn render_lines(&mut self) -> Vec<String> {
        let mut lines: Vec<String> = vec![];
        lines.push(format!(
            "\r{}{}",
            self.render_search_line(self.scores.len()),
            clear::AfterCursor
        ));

        let height = min(self.console.height, 20);
        let num_matches = min((height - 1) as usize, self.scores.len());
        self.num_rendered = num_matches;

        for (i, score) in self.scores.iter().enumerate().take(num_matches) {
            lines.push(highlight_score_line(
                score,
                self.console.width as usize,
                self.selected == i,
                self.query.len() != 0,
            ));
        }

        lines
    }

    pub fn render(&mut self) {
        let lines = self.render_lines();
        self.console.write_lines(lines);
        if self.num_rendered > 0 {
            self.console.write(
                format!("{}", cursor::Up(self.num_rendered as u16)).as_str(),
            );
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

fn highlight_score_line(
    score: &Score,
    width: usize,
    selected: bool,
    has_query: bool,
) -> String {
    let truncated = score.line.split_at(min(width, score.line.len())).0;
    // need to split _after_ score.last so we include the last
    // character in the highlighted portion
    let (left, right) =
        truncated.split_at(min(truncated.len(), score.last + 1));
    let (left, middle) = left.split_at(min(left.len(), score.first));
    format!(
        "{}{}{}{}{}{}{}{}",
        if selected {
            format!("{}", style::Invert)
        } else {
            "".to_string()
        },
        expand_tabs(left),
        if has_query {
            format!("{}", color::Fg(color::Red))
        } else {
            "".to_string()
        },
        expand_tabs(middle),
        color::Fg(color::Reset),
        expand_tabs(right),
        style::Reset,
        clear::AfterCursor
    )
}

pub fn expand_tabs(line: &str) -> String {
    let tab_width = 8;
    let re = Regex::new(r"([^\t\n]*)\t").unwrap();
    re.replace_all(line, |caps: &Captures| {
        format!(
            "{}{}",
            &caps[1],
            " ".repeat(tab_width - (&caps[1].len() % tab_width))
        )
    }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use score::calculate_score;

    #[test]
    fn test_highlight_score_line() {
        let score = calculate_score(
            "xxxfoobarxxx",
            "foobar".chars().collect::<Vec<char>>().as_slice(),
        ).unwrap();
        let expected = format!(
            "xxx{}foobar{}xxx{}{}",
            color::Fg(color::Red),
            color::Fg(color::Reset),
            style::Reset,
            clear::AfterCursor
        );
        assert_eq!(highlight_score_line(&score, 80, false, true), expected);

        let expected = format!(
            "{}xxx{}foobar{}xxx{}{}",
            style::Invert,
            color::Fg(color::Red),
            color::Fg(color::Reset),
            style::Reset,
            clear::AfterCursor
        );
        assert_eq!(highlight_score_line(&score, 80, true, true), expected);

        let expected = format!(
            "xxxfoobar{}xxx{}{}",
            color::Fg(color::Reset),
            style::Reset,
            clear::AfterCursor
        );
        assert_eq!(highlight_score_line(&score, 80, false, false), expected);
    }

    #[test]
    fn test_expand_tabs() {
        assert_eq!(expand_tabs("foo\tbar"), "foo     bar");
        assert_eq!(expand_tabs("fo\tbar"), "fo      bar");
        assert_eq!(expand_tabs("f\tbar"), "f       bar");
        assert_eq!(
            expand_tabs("foo\tbar\tbaz\tquux"),
            "foo     bar     baz     quux"
        );
        assert_eq!(
            expand_tabs("foo\tbar\t\tquux"),
            "foo     bar             quux"
        );
    }
}
