use crate::line::Line;
use std::cmp::Ordering;

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
    fn new(line: &'a Line) -> Self {
        Self {
            first: 0,
            last: 0,
            points: usize::MAX,
            line,
        }
    }

    fn range_length(&self) -> usize {
        self.last - self.first
    }

    pub fn cmp(a: &Score, b: &Score) -> Ordering {
        if a.points == b.points {
            if a.range_length() == b.range_length() {
                a.line.len().cmp(&b.line.len())
            } else {
                a.range_length().cmp(&b.range_length())
            }
        } else {
            a.points.cmp(&b.points)
        }
    }
}

pub fn calculate_score<'a>(line: &'a Line, query: &[char]) -> Option<Score<'a>> {
    match query.len() {
        0 => Some(Score::new(line)),
        1 => line
            .low_buf
            .find(query[0].to_lowercase().to_string().as_str())
            .map(|index| Score {
                line,
                points: 1,
                last: index + 1,
                first: index,
            }),
        _ => {
            let mut score = Score::new(line);
            let mut found_score = false;
            for (start, _) in line
                .low_buf
                .match_indices(query[0].to_lowercase().to_string().as_str())
            {
                if let Some((points, last_index)) = find_end_of_match(line, &query[1..], start) {
                    found_score = true;
                    if last_index != 0 && points < score.points {
                        score.first = start;
                        score.last = last_index + 1;
                        score.points = points;
                    }
                } else {
                    // take this string: "foobarflubfuzz" - if we're searching for "fbar", we'll
                    // find it in the initial six chars ("foobar"), but won't find it past that.
                    // if find_end_of_match() returns None, that means it didn't find one of the
                    // chars it was looking for, so don't bother searching after that. i.e. we'll
                    // search from the 'f' in "flub", won't find 'a', and thus won't bother
                    // searching from the 'f' in "fuzz"
                    break;
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

    for c in chars.iter().collect::<String>().to_lowercase().chars() {
        let index = match line
            .low_char_vec
            .iter()
            .find(|t| t.0 > last_index && c == t.1)
        {
            Some(t) => t.0,
            None => return None,
        };

        if index == last_index + 1 {
            if last_match_kind != MatchKind::Sequential {
                last_match_kind = MatchKind::Sequential;
                score += 1;
            }
        } else if index < line.low_char_vec.len()
            && !line.low_char_vec[index - 1].1.is_alphanumeric()
        {
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
        // test to make sure calculate_score() breaks in else clause
        let line = Line::from("foobarflubfuzz");
        let expected = Some(Score {
            line: &line,
            first: 0,
            last: 6,
            points: 5,
        });
        assert_eq!(calculate_score(&line, &['f', 'b', 'a', 'r']), expected);

        // test score is None if query isn't in string
        assert_eq!(calculate_score(&Line::from("foo"), &['q', 'x', 'z']), None);

        // test score is usize::max_value() if query is empty
        let line = Line::from("foo");
        let expected = Some(Score {
            first: 0,
            last: 0,
            points: usize::max_value(),
            line: &line,
        });
        assert_eq!(calculate_score(&Line::from("foo"), &[]), expected);

        // test single character query
        let line = Line::from("oof");
        let expected = Some(Score {
            first: 2,
            last: 3,
            points: 1,
            line: &line,
        });
        assert_eq!(calculate_score(&Line::from("oof"), &['f']), expected);
        assert_eq!(calculate_score(&Line::from("oof"), &['b']), None);

        // some tests to match scores from selecta.rb
        let line = Line::from("foofbbar");
        let expected = Some(Score {
            first: 0,
            last: 5,
            points: 5,
            line: &line,
        });
        assert_eq!(
            calculate_score(&Line::from("foofbbar"), &['f', 'o', 'b']),
            expected
        );

        let line = Line::from("foo / ba r");
        let expected = Some(Score {
            first: 1,
            last: 10,
            points: 2,
            line: &line,
        });
        assert_eq!(
            calculate_score(&Line::from("foo / ba r"), &['o', 'r']),
            expected
        );

        let line = Line::from("f||||||||b||||||||||||||a||||f||||||||r");
        let expected = Some(Score {
            first: 9,
            last: 39,
            points: 2,
            line: &line,
        });
        assert_eq!(
            calculate_score(
                &Line::from("f||||||||b||||||||||||||a||||f||||||||r"),
                &['b', 'a', 'r']
            ),
            expected
        );

        let line = Line::from("foo / ba /**  r");
        let expected = Some(Score {
            first: 6,
            last: 15,
            points: 3,
            line: &line,
        });
        assert_eq!(
            calculate_score(&Line::from("foo / ba /**  r"), &['b', 'a', 'r']),
            expected
        );

        // make sure best score is calculated when duplicates exist.
        // this case is identical to the prior, except with "bar" at the
        // beginning, so score should be much better (lower)
        let line = Line::from("barfoo / ba /**  r");
        let expected = Some(Score {
            first: 0,
            last: 3,
            points: 2,
            line: &line,
        });
        assert_eq!(
            calculate_score(&Line::from("barfoo / ba /**  r"), &['b', 'a', 'r']),
            expected
        );

        // make sure best score is calculated when duplicates exist.
        // this case is identical to the prior, except with "bar" at the
        // end, so score should be the same (though with different span)
        let line = Line::from("foo / ba /**  rbar");
        let expected = Some(Score {
            first: 15,
            last: 18,
            points: 2,
            line: &line,
        });
        assert_eq!(
            calculate_score(&Line::from("foo / ba /**  rbar"), &['b', 'a', 'r']),
            expected
        );
    }

    #[test]
    fn test_find_end_of_match() {
        // test score is None if query isn't in string
        assert_eq!(
            find_end_of_match(&Line::from("foo"), &['q', 'x', 'z'], 0),
            None
        );

        // test score is None if query isn't in string _after_ specified start
        assert_eq!(
            find_end_of_match(&Line::from("foofbar"), &['f', 'o'], 3),
            None
        );

        // find first score when multiples exist. keep in mind that
        // find_end_of_match doesn't find the _entire_ match, just the end of
        // the match given the correct starting point, so whatever `start` value
        // is provided is assumed to be the start of the match.
        let s = Line::from("foofoofoobar");
        assert_eq!(find_end_of_match(&s, &['b'], 0), Some((10, 9)));
        assert_eq!(find_end_of_match(&s, &['b'], 4), Some((6, 9)));
        assert_eq!(find_end_of_match(&s, &['b'], 8), Some((2, 9)));
    }

    #[test]
    fn test_range_length() {
        let s = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("foobar"),
        };
        assert_eq!(s.range_length(), 3);
    }

    #[test]
    fn test_ordering() {
        // the score structs in this test might not be valid. we're just testing
        // specific attributes so that's okay.
        //
        // test points
        let a = Score {
            first: 0,
            last: 3,
            points: 1,
            line: &Line::from("foobar"),
        };
        let b = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("foobar"),
        };
        assert_eq!(Score::cmp(&a, &b), Ordering::Less);
        let a = Score {
            first: 0,
            last: 3,
            points: 3,
            line: &Line::from("foobar"),
        };
        let b = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("foobar"),
        };
        assert_eq!(Score::cmp(&a, &b), Ordering::Greater);
        // test range length
        let a = Score {
            first: 0,
            last: 1,
            points: 2,
            line: &Line::from("foobar"),
        };
        let b = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("foobar"),
        };
        assert_eq!(Score::cmp(&a, &b), Ordering::Less);
        let a = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("foobar"),
        };
        let b = Score {
            first: 0,
            last: 1,
            points: 2,
            line: &Line::from("foobar"),
        };
        assert_eq!(Score::cmp(&a, &b), Ordering::Greater);
        // test line length
        let a = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("fooba"),
        };
        let b = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("foobar"),
        };
        assert_eq!(Score::cmp(&a, &b), Ordering::Less);
        let a = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("foobar"),
        };
        let b = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("fooba"),
        };
        assert_eq!(Score::cmp(&a, &b), Ordering::Greater);
        // if points, range length and line length all match, ordering should be equal
        let a = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("foobar"),
        };
        let b = Score {
            first: 0,
            last: 3,
            points: 2,
            line: &Line::from("foobar"),
        };
        assert_eq!(Score::cmp(&a, &b), Ordering::Equal);
    }
}
