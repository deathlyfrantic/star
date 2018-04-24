use std::cmp::min;
use termion::{clear, color, cursor, style};
use regex::{Captures, Regex};

use score::Score;
use console;

pub struct Renderer<'a> {
    scores: &'a Vec<Score<'a>>,
    console: &'a console::Console,
    query: &'a str,
    pub selected: usize,
}

impl<'a> Renderer<'a> {
    pub fn new(
        scores: &'a Vec<Score<'a>>,
        console: &'a console::Console,
        query: &'a str,
        selected: usize,
    ) -> Renderer<'a> {
        Renderer {
            scores: scores,
            console: console,
            query: query,
            selected: selected,
        }
    }

    pub fn render_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = vec![];
        lines.push(format!(
            "\r{}",
            render_search_line(self.scores.len(), self.query)
        ));

        let height = min(self.console.height, 20);
        let num_matches = min((height - 1) as usize, self.scores.len());

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

    pub fn render(&self) {
        let lines = self.render_lines();
        self.console.write_lines(lines);
        self.console.write(format!("{}", cursor::Up(19)).as_str());
        self.console.write("\r");
        self.console
            .write(render_search_line(self.scores.len(), self.query).as_str());
    }

    pub fn clear(&self) {
        let height = min(self.console.height, 19);
        self.console.write(
            format!("{}\r\n", clear::CurrentLine)
                .repeat(height as usize)
                .as_str(),
        );
    }
}

fn render_search_line(num_scores: usize, query: &str) -> String {
    format!("{} > {}", num_scores, query)
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
    fn test_render_search_line() {
        assert_eq!(render_search_line(10, "foo"), "10 > foo");
    }

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
