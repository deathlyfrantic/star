use line::Line;

#[derive(Debug, PartialEq)]
enum MatchKind {
    Sequential,
    Boundary,
    Normal,
}

#[derive(Debug, PartialEq)]
pub struct Score<'a> {
    pub first: usize,
    pub last: usize,
    pub points: usize,
    pub line: &'a Line,
}

impl<'a> Score<'a> {
    fn new(line: &'a Line) -> Score {
        Score {
            first: 0,
            last: 0,
            points: usize::max_value(),
            line: line,
        }
    }
}

pub fn calculate_score<'a>(line: &'a Line, query: &[char]) -> Option<Score<'a>> {
    let mut score = Score::new(line);
    match query.len() {
        0 => Some(score),
        1 => match line
            .low_buf
            .find(query[0].to_lowercase().to_string().as_str())
        {
            Some(index) => {
                score.points = 1;
                score.last = index;
                score.first = index;
                Some(score)
            }
            None => None,
        },
        _ => {
            let mut found_score = false;
            for (start, _) in line
                .low_buf
                .match_indices(query[0].to_lowercase().to_string().as_str())
            {
                if let Some((points, last_index)) = find_end_of_match(&line, &query[1..], start) {
                    found_score = true;
                    if last_index != 0 && points < score.points {
                        score.first = start;
                        score.last = last_index;
                        score.points = points;
                    }
                }
            }

            if found_score {
                Some(score)
            } else {
                None
            }
        }
    }
}

fn find_end_of_match(line: &Line, chars: &[char], start: usize) -> Option<(usize, usize)> {
    let mut last_index = start;
    let mut score = 1;
    let mut last_match_kind = MatchKind::Normal;

    for c in chars {
        let index = match line
            .low_char_vec
            .iter()
            .find(|t| t.0 > last_index && c == &t.1)
        {
            Some(t) => t.0,
            None => return None,
        };

        if index == last_index + 1 {
            if last_match_kind != MatchKind::Sequential {
                last_match_kind = MatchKind::Sequential;
                score += 1;
            }
        } else if !line.low_char_vec[index - 1].1.is_alphanumeric() {
            if last_match_kind != MatchKind::Boundary {
                last_match_kind = MatchKind::Boundary;
                score += 1;
            }
        } else {
            last_match_kind = MatchKind::Normal;
            score += index - last_index;
        }

        last_index = index;
    }

    Some((score, last_index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_score() {
        // test score is None if query isn't in string
        assert_eq!(
            calculate_score(&Line::new(String::from("foo")), &['q', 'x', 'z']),
            None
        );

        // test score is usize::max_value() if query is empty
        let line = Line::new(String::from("foo"));
        let expected = Some(Score {
            first: 0,
            last: 0,
            points: usize::max_value(),
            line: &line,
        });
        assert_eq!(
            calculate_score(&Line::new(String::from("foo")), &[]),
            expected
        );

        // test single character query
        let line = Line::new(String::from("oof"));
        let expected = Some(Score {
            first: 2,
            last: 2,
            points: 1,
            line: &line,
        });
        assert_eq!(
            calculate_score(&Line::new(String::from("oof")), &['f']),
            expected
        );
        assert_eq!(
            calculate_score(&Line::new(String::from("oof")), &['b']),
            None
        );

        // some tests to match scores from selecta.rb
        let line = Line::new(String::from("foofbbar"));
        let expected = Some(Score {
            first: 0,
            last: 4,
            points: 5,
            line: &line,
        });
        assert_eq!(
            calculate_score(&Line::new(String::from("foofbbar")), &['f', 'o', 'b']),
            expected
        );

        let line = Line::new(String::from("foo / ba r"));
        let expected = Some(Score {
            first: 1,
            last: 9,
            points: 2,
            line: &line,
        });
        assert_eq!(
            calculate_score(&Line::new(String::from("foo / ba r")), &['o', 'r']),
            expected
        );

        let line = Line::new(String::from("f||||||||b||||||||||||||a||||f||||||||r"));
        let expected = Some(Score {
            first: 9,
            last: 38,
            points: 2,
            line: &line,
        });
        assert_eq!(
            calculate_score(
                &Line::new(String::from("f||||||||b||||||||||||||a||||f||||||||r")),
                &['b', 'a', 'r']
            ),
            expected
        );

        let line = Line::new(String::from("foo / ba /**  r"));
        let expected = Some(Score {
            first: 6,
            last: 14,
            points: 3,
            line: &line,
        });
        assert_eq!(
            calculate_score(
                &Line::new(String::from("foo / ba /**  r")),
                &['b', 'a', 'r']
            ),
            expected
        );

        // make sure best score is calculated when duplicates exist.
        // this case is identical to the prior, except with "bar" at the
        // beginning, so score should be much better (lower)
        let line = Line::new(String::from("barfoo / ba /**  r"));
        let expected = Some(Score {
            first: 0,
            last: 2,
            points: 2,
            line: &line,
        });
        assert_eq!(
            calculate_score(
                &Line::new(String::from("barfoo / ba /**  r")),
                &['b', 'a', 'r']
            ),
            expected
        );

        // make sure best score is calculated when duplicates exist.
        // this case is identical to the prior, except with "bar" at the
        // end, so score should be the same (though with different span)
        let line = Line::new(String::from("foo / ba /**  rbar"));
        let expected = Some(Score {
            first: 15,
            last: 17,
            points: 2,
            line: &line,
        });
        assert_eq!(
            calculate_score(
                &Line::new(String::from("foo / ba /**  rbar")),
                &['b', 'a', 'r']
            ),
            expected
        );
    }

    #[test]
    fn test_find_end_of_match() {
        // test score is None if query isn't in string
        assert_eq!(
            find_end_of_match(&Line::new(String::from("foo")), &['q', 'x', 'z'], 0),
            None
        );

        // test score is None if query isn't in string _after_ specified start
        assert_eq!(
            find_end_of_match(&Line::new(String::from("foofbar")), &['f', 'o'], 3),
            None
        );

        // find first score when multiples exist. keep in mind that
        // find_end_of_match doesn't find the _entire_ match, just the end of
        // the match given the correct starting point, so whatever `start` value
        // is provided is assumed to be the start of the match.
        let s = Line::new(String::from("foofoofoobar"));
        assert_eq!(find_end_of_match(&s, &['b'], 0), Some((10, 9)));
        assert_eq!(find_end_of_match(&s, &['b'], 4), Some((6, 9)));
        assert_eq!(find_end_of_match(&s, &['b'], 8), Some((2, 9)));
    }
}
