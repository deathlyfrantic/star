#[derive(Debug, PartialEq)]
pub struct Line {
    pub buf: String,
    pub low_buf: String,
}

impl Line {
    pub fn new(s: String) -> Line {
        Line {
            low_buf: s.to_lowercase(),
            buf: s,
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_new() {
        let l = Line::new(String::from("FOOBAR"));
        assert_eq!(l.low_buf, "foobar");
    }

    #[test]
    fn test_line_len() {
        let l = Line::new(String::from("FOOBAR"));
        assert_eq!(l.len(), 6);
    }
}
