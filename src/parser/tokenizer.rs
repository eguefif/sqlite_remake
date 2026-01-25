use std::iter::{Iterator, Peekable};
use std::str::Chars;

#[derive(PartialEq, Debug)]
pub enum Token {
    Select,
    Coma,
    From,
    Value(String),
    SemiComa,
}

impl Token {
    pub fn from_str(str: &str) -> Self {
        let lower_str = str.to_lowercase();
        match lower_str.as_str() {
            "select" => Token::Select,
            "from" => Token::From,
            "," => Token::Coma,
            ";" => Token::SemiComa,
            _ => Token::Value(lower_str),
        }
    }
}

pub struct Tokenizer<'a> {
    buffer: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(query_str: &'a str) -> Self {
        Self {
            buffer: query_str.chars().peekable(),
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let mut token_str = String::new();
        loop {
            let Some(next) = self.buffer.next() else {
                return None;
            };
            if next == ' ' {
                break;
            }
            token_str.push(next);
        }
        Some(Token::from_str(&token_str))
    }
}
