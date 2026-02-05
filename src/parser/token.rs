use anyhow::{Result, anyhow};
use std::fmt;

use crate::executor::db_response::RType;

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
    Command(Command),
}

impl Token {
    pub fn from_str(str: &str) -> Result<Self> {
        let lower_str = str.to_lowercase();
        let token = match lower_str.as_str() {
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
                let first_char = lower_str
                    .chars()
                    .next()
                    .expect("We know that there are at least one char");
                if first_char.is_numeric() {
                    Token::Num(lower_str.parse::<i64>()?)
                } else if first_char == '.' {
                    let cmd = Command::from_str(&lower_str)?;
                    Token::Command(cmd)
                } else {
                    if lower_str.starts_with("\'") {
                        Token::QIdent(str.trim_matches('\'').to_string())
                    } else {
                        Token::Ident(lower_str)
                    }
                }
            }
        };
        Ok(token)
    }

    pub fn into_rtype(&self) -> RType {
        match self {
            Token::Num(value) => RType::Num(*value),
            Token::QIdent(value) => RType::Str(value.to_string()),
            _ => panic!("Should never transform {} into RType", self),
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
            Token::Command(command) => write!(f, "Command: {}", command),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    DBinfo,
    Tables,
}

impl Command {
    pub fn from_str(command: &str) -> Result<Self> {
        let command = match command {
            ".dbinfo" => Command::DBinfo,
            ".tables" => Command::Tables,
            _ => return Err(anyhow!("{}: command does not exists", command)),
        };
        Ok(command)
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::DBinfo => write!(f, ".dbinfo"),
            Command::Tables => write!(f, ".tables"),
        }
    }
}
