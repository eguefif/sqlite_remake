use std::iter::{Iterator, Peekable};
use std::str::Chars;

use crate::parser::token::Token;

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

    fn trim_space(&mut self) {
        loop {
            let Some(peek) = self.buffer.peek() else {
                break;
            };
            if *peek != ' ' {
                break;
            }
            self.buffer.next();
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    // Returns the next token, consuming it
    fn next(&mut self) -> Option<Self::Item> {
        // Return peeked value saved when used the method Tokenizer.peek()
        if let Some(peeked) = self.peeked.take() {
            return Some(peeked);
        }
        if let None = self.buffer.peek() {
            return None;
        }
        self.trim_space();
        let mut next = self.buffer.next().unwrap();
        match next {
            ';' => return Some(Token::SemiColon),
            ',' => return Some(Token::Coma),
            '(' => return Some(Token::LParen),
            ')' => return Some(Token::RParen),
            '+' => return Some(Token::Plus),
            '-' => return Some(Token::Minus),
            '/' => return Some(Token::Div),
            '*' => return Some(Token::Star),
            '=' => return Some(Token::Equal),
            '!' => {
                if let Some(peek) = self.buffer.peek() {
                    if *peek == '=' {
                        self.buffer.next();
                        return Some(Token::NotEq);
                    }
                }
                return Some(Token::Illegal("!".to_string()));
            }
            '>' => {
                if let Some(peek) = self.buffer.peek() {
                    if *peek == '=' {
                        self.buffer.next();
                        return Some(Token::GTEQ);
                    }
                }
                return Some(Token::GT);
            }
            '<' => {
                if let Some(peek) = self.buffer.peek() {
                    if *peek == '=' {
                        self.buffer.next();
                        return Some(Token::LTEQ);
                    }
                }
                return Some(Token::LT);
            }
            '\'' => {
                let mut token_qident = String::new();
                token_qident.push(next);
                loop {
                    let Some(peek) = self.buffer.peek() else {
                        break;
                    };
                    if *peek == '\'' {
                        token_qident.push(self.buffer.next().unwrap());
                        break;
                    }
                    let next = self.buffer.next().unwrap();
                    token_qident.push(next);
                }
                return Some(Token::from_str(&token_qident));
            }
            _ => {
                let mut token_str = String::new();
                loop {
                    token_str.push(next);
                    let Some(peek) = self.buffer.peek() else {
                        break;
                    };

                    if is_stop_identifier(*peek) {
                        if *peek == ' ' {
                            self.buffer.next();
                            self.trim_space();
                        }
                        break;
                    }
                    next = self.buffer.next().unwrap();
                }
                return Some(Token::from_str(&token_str));
            }
        }
    }
}

const STOP_CHARS: [char; 12] = [';', '(', ')', ',', ' ', '*', '=', '<', '!', '>', '+', '-'];

fn is_stop_identifier(c: char) -> bool {
    for stop_char in STOP_CHARS.iter() {
        if c == *stop_char {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_tokenize_simple_query() {
        let tokenizer = Tokenizer::new("SELECT COUNT(*) FROM apples;");
        let expected_tokens = [
            Token::Select,
            Token::Ident("count".to_string()),
            Token::LParen,
            Token::Star,
            Token::RParen,
            Token::From,
            Token::Ident("apples".to_string()),
        ];

        for (expected, token) in expected_tokens.into_iter().zip(tokenizer) {
            println!("|{}|", token);
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn it_should_tokenize_multiple_value_select() {
        let tokenizer = Tokenizer::new("SELECT name, color, number FROM apples;");
        let expected_tokens = [
            Token::Select,
            Token::Ident("name".to_string()),
            Token::Coma,
            Token::Ident("color".to_string()),
            Token::Coma,
            Token::Ident("number".to_string()),
            Token::From,
            Token::Ident("apples".to_string()),
        ];

        for (expected, token) in expected_tokens.into_iter().zip(tokenizer) {
            println!("|{}|", token);
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn it_should_tokenize_with_where() {
        let tokenizer =
            Tokenizer::new("SELECT name, color, number FROM apples WHERE name = 'hey';");
        let expected_tokens = [
            Token::Select,
            Token::Ident("name".to_string()),
            Token::Coma,
            Token::Ident("color".to_string()),
            Token::Coma,
            Token::Ident("number".to_string()),
            Token::From,
            Token::Ident("apples".to_string()),
            Token::Where,
            Token::Ident("name".to_string()),
            Token::Equal,
            Token::QIdent("hey".to_string()),
        ];

        for (expected, token) in expected_tokens.into_iter().zip(tokenizer) {
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn it_should_tokenize_with_where_correct_len() {
        let tokenizer =
            Tokenizer::new("SELECT name, color, number FROM apples WHERE name = 'hey';");

        assert_eq!(tokenizer.collect::<Vec<Token>>().len(), 13);
    }

    #[test]
    fn it_should_tokenize_all_token() {
        let tokenizer = Tokenizer::new(
            "SELECT COUNT(*), name, color, number FROM apples WHERE name = 'hey' +- / <= >= != NULL Not like Ilike 25;",
        );

        let expected_tokens = [
            Token::Select,
            Token::Ident("count".to_string()),
            Token::LParen,
            Token::Star,
            Token::RParen,
            Token::Coma,
            Token::Ident("name".to_string()),
            Token::Coma,
            Token::Ident("color".to_string()),
            Token::Coma,
            Token::Ident("number".to_string()),
            Token::From,
            Token::Ident("apples".to_string()),
            Token::Where,
            Token::Ident("name".to_string()),
            Token::Equal,
            Token::QIdent("hey".to_string()),
            Token::Plus,
            Token::Minus,
            Token::Div,
            Token::LTEQ,
            Token::GTEQ,
            Token::NotEq,
            Token::Null,
            Token::Not,
            Token::Like,
            Token::ILike,
            Token::Num(25),
        ];

        for (expected, token) in expected_tokens.into_iter().zip(tokenizer) {
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn it_should_tokenize_all_token_no_space() {
        let tokenizer = Tokenizer::new("name='hey'+-/<=>=!=NULL Not like Ilike 25;");

        let expected_tokens = [
            Token::Ident("name".to_string()),
            Token::Equal,
            Token::QIdent("hey".to_string()),
            Token::Plus,
            Token::Minus,
            Token::Div,
            Token::LTEQ,
            Token::GTEQ,
            Token::NotEq,
            Token::Null,
            Token::Not,
            Token::Like,
            Token::ILike,
            Token::Num(25),
        ];

        for (expected, token) in expected_tokens.into_iter().zip(tokenizer) {
            assert_eq!(token, expected);
        }
    }
}
