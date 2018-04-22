use std::cmp::min;

use score;
use console;
use ansi;

pub struct Renderer<'a> {
    scores: &'a Vec<score::Score<'a>>,
    console: &'a console::Console,
    query: &'a [char],
}

impl<'a> Renderer<'a> {
    pub fn new(
        scores: &'a Vec<score::Score<'a>>,
        console: &'a console::Console,
        query: &'a [char],
    ) -> Renderer<'a> {
        Renderer {
            scores: scores,
            console: console,
            query: query,
        }
    }

    fn highlight_score_line(&self, score: &score::Score) -> String {
        let truncated = score
            .line
            .split_at(min(self.console.width as usize, score.line.len()))
            .0;
        // need to split _after_ score.last so we include the last
        // character in the highlighted portion
        let (left, right) =
            truncated.split_at(min(truncated.len(), score.last + 1));
        let (left, middle) = left.split_at(min(left.len(), score.first));
        format!(
            "{}{}{}{}{}",
            left,
            ansi::color("red", "default"),
            middle,
            ansi::reset(),
            right
        )
    }

    pub fn render_search_line(&self) -> String {
        format!(
            "{} > {}",
            self.scores.len(),
            self.query.iter().collect::<String>()
        )
    }

    pub fn render_match_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = vec![];
        lines.push(self.render_search_line());

        for score in &self.scores[0..19] {
            lines.push(self.highlight_score_line(score));
        }

        lines
    }

    pub fn render(&self) {
        let mut lines = self.render_match_lines();
        lines.insert(0, "".to_string());
        self.console.write_lines(lines);
    }
}
