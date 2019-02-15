#[derive(Debug, PartialEq)]
pub struct Line {
    pub buf: String,
    pub low_buf: String,
    pub low_char_vec: Vec<(usize, char)>,
}

impl Line {
    pub fn new(buf: String) -> Line {
        let low_buf = buf.to_lowercase();
        Line {
            low_char_vec: low_buf.char_indices().collect(),
            low_buf,
            buf,
        }
    }

    #[cfg(test)]
    pub fn from(s: &str) -> Line {
        Line::new(String::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_new() {
        let l = Line::from("FOOBAR");
        assert_eq!(l.low_buf, "foobar");
    }

    #[test]
    fn test_low_char_vec() {
        let l = Line::from("FOOBAR");
        for (i, c) in l.low_char_vec {
            assert_eq!(
                match i {
                    0 => 'f',
                    3 => 'b',
                    4 => 'a',
                    5 => 'r',
                    _ => 'o',
                },
                c
            );
        }
    }
}
