use core::iter::Iterator;
use std::fmt::{Display, Formatter};
use std::str::{Chars, FromStr};
use itertools::{Itertools, MultiPeek};

#[derive(Clone)]
pub struct Parser<'a> {
    orig: &'a str,
    stream: MultiPeek<Chars<'a>>,
    lines: usize,
}

impl Display for Parser<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.orig.lines().skip(self.lines).next() {
            writeln!(f)?;
            writeln!(f, "{}", line)?;

            let len = line.len();
            if let Some(next_newline) = self.stream.clone().position(|i| i == '\n') {
                let from_prev = len - next_newline;
                write!(f, "{:level$}^", "", level = from_prev)?;
            }

            Ok(())
        } else {
            write!(f, "<unknown file location>")
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(i: &'a str) -> Self {
        Self {
            orig: i,
            stream: i.chars().multipeek(),
            lines: 0,
        }
    }

    fn next(&mut self) -> Option<char> {
        let c = self.stream.next();
        if let Some(i) = c {
            if i == '\n' {
                self.lines += 1;
            }
        }
        c
    }

    pub fn is_empty(&mut self) -> bool {
        self.stream.peek().is_none()
    }

    pub fn accept(&mut self, c: char) -> Option<()> {
        self.stream.reset_peek();
        let next = self.stream.peek();
        if next == Some(&c) {
            let _ = self.next();
            // println!("accept: {c}");
            Some(())
        } else {
            self.stream.reset_peek();
            None
        }
    }

    pub fn accept_with(&mut self, c: impl Fn(char) -> bool) -> Option<char> {
        self.stream.reset_peek();
        if let Some(&i) = self.stream.peek() {
            if c(i) {
                let _ = self.next();
                Some(i)
            } else {
                self.stream.reset_peek();
                None
            }
        } else {
            self.stream.reset_peek();
            None
        }
    }

    pub fn accept_str(&mut self, s: &str) -> Option<()> {
        for i in s.chars() {
            if self.stream.peek() != Some(&i) {
                self.stream.reset_peek();
                return None;
            }
        }

        for _ in s.chars() {
            self.next();
        }

        Some(())
    }

    pub fn whitespace(&mut self) -> bool {
        let mut res = false;

        while self.accept_with(|i| i.is_whitespace()).is_some() {
            res = true;
        }

        res
    }

    pub fn parse_num<T: FromStr>(&mut self) -> Option<T> {
        let mut res = String::new();

        while let Some(i) = self.accept_with(|i| i.is_ascii_digit()) {
            res.push(i)
        }

        if res.is_empty() {
            None
        } else {
            // println!("num: {res}");
            Some(res.parse().ok()?)
        }
    }


    pub fn parse_ident(&mut self) -> Option<String> {
        let mut res = String::new();

        while let Some(i) = self.accept_with(|i| i.is_alphanumeric()) {
            res.push(i)
        }

        if res.len() == 0 {
            return None
        }
        if res.chars().next().unwrap().is_ascii_digit() {
            return None
        }

        Some(res)
    }
}
