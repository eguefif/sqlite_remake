use std::fmt;
use std::iter::{Iterator, Peekable};
use std::str::Chars;

pub struct Tokenizer<'a> {
    buffer: Peekable<Chars<'a>>,
    peeked: Option<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(query_str: &'a str) -> Self {
        Self {
            buffer: query_str.chars().peekable(),
            peeked: None,
        }
    }

    // Returns the next token without consuming it
    pub fn peek(&mut self) -> Option<&Token> {
        if let Some(ref peeked) = self.peeked {
            return Some(peeked);
        }
        if let Some(next) = self.next() {
            self.peeked = Some(next);
            return self.peeked.as_ref();
        }

        None
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    // Returns the next token, consuming it
    // TODO: add token COUNT
    // add TOKEN *
    // add TOKEN open parenthesis and close one
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(peeked) = self.peeked.take() {
            return Some(peeked);
        }
        let mut token_str = String::new();
        loop {
            let Some(peek) = self.buffer.peek() else {
                if token_str.len() == 0 {
                    return None;
                } else {
                    return Some(Token::from_str(&token_str));
                }
            };
            // TODO: refactor, this can be better
            match *peek {
                ';' => {
                    // TODO: this pattern is repeated, we can do better, refactor
                    if token_str.len() == 0 {
                        self.buffer.next();
                        return Some(Token::from_str(";"));
                    }
                    break;
                }
                ',' => {
                    if token_str.len() == 0 {
                        self.buffer.next();
                        return Some(Token::from_str(","));
                    }
                    break;
                }
                ' ' => {
                    self.buffer.next();
                    if token_str.len() > 0 {
                        break;
                    }
                }
                _ => {
                    let next = self.buffer.next().unwrap();
                    token_str.push(next);
                }
            }
        }
        Some(Token::from_str(&token_str))
    }
}

#[derive(PartialEq, Debug)]
pub enum Token {
    Select,
    Coma,
    From,
    Value(String),
    SemiColon,
}

impl Token {
    pub fn from_str(str: &str) -> Self {
        let lower_str = str.to_lowercase();
        match lower_str.as_str() {
            "select" => Token::Select,
            "from" => Token::From,
            "," => Token::Coma,
            ";" => Token::SemiColon,
            _ => Token::Value(lower_str),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Select => write!(f, "Token::Select"),
            Token::Coma => write!(f, "Token::Coma"),
            Token::From => write!(f, "Token::From"),
            Token::SemiColon => write!(f, "Token::SemiColon"),
            Token::Value(value) => write!(f, "Token::Value({})", value),
        }
    }
}

