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

#[derive(PartialEq, Debug)]
pub enum Token {
    Illegal(String),
    Select,
    From,
    Where,
    Null,
    Not,
    Like,
    ILike,
    Ident(String),
    QIdent(String),
    Num(i64),
    Coma,
    SemiColon,
    RParen,
    LParen,
    Star,
    Equal,
    NotEq,
    GT,
    LT,
    GTEQ,
    LTEQ,
    Plus,
    Minus,
    Div,
}

impl Token {
    pub fn from_str(str: &str) -> Self {
        let lower_str = str.to_lowercase();
        match lower_str.as_str() {
            "where" => Token::Where,
            "select" => Token::Select,
            "from" => Token::From,
            "null" => Token::Null,
            "not" => Token::Not,
            "like" => Token::Like,
            "ilike" => Token::ILike,
            "," => Token::Coma,
            ";" => Token::SemiColon,
            "(" => Token::LParen,
            ")" => Token::RParen,
            "*" => Token::Star,
            "=" => Token::Equal,
            "!=" => Token::NotEq,
            ">" => Token::GT,
            "<" => Token::LT,
            ">=" => Token::GTEQ,
            "<=" => Token::LTEQ,
            "+" => Token::Plus,
            "-" => Token::Minus,
            "/" => Token::Div,
            _ => {
                // TODO: handle error
                if lower_str.chars().next().unwrap().is_numeric() {
                    return Token::Num(lower_str.parse::<i64>().unwrap());
                } else {
                    if lower_str.starts_with("\'") {
                        return Token::QIdent(str.trim_matches('\'').to_string());
                    }
                    return Token::Ident(lower_str);
                }
            }
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Select => write!(f, "SELECT"),
            Token::Where => write!(f, "WHERE"),
            Token::From => write!(f, "FROM"),
            Token::Not => write!(f, "NOT"),
            Token::Like => write!(f, "LIKE"),
            Token::ILike => write!(f, "ILIKE"),
            Token::RParen => write!(f, "("),
            Token::LParen => write!(f, ")"),
            Token::Coma => write!(f, ","),
            Token::SemiColon => write!(f, ";"),
            Token::Ident(value) => write!(f, "{}", value),
            Token::QIdent(value) => write!(f, "'{}'", value),
            Token::Num(value) => write!(f, "{}", value),
            Token::Star => write!(f, "*"),
            Token::Null => write!(f, "NULL"),
            Token::Equal => write!(f, "="),
            Token::NotEq => write!(f, "!="),
            Token::GT => write!(f, ">"),
            Token::LT => write!(f, "<"),
            Token::GTEQ => write!(f, ">="),
            Token::LTEQ => write!(f, "<="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Div => write!(f, "/"),
            Token::Illegal(value) => write!(f, "Illegal token: {}", value),
        }
    }
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
