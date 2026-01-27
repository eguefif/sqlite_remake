use crate::db::parser::tokenizer::Token;
use std::fmt;

#[derive(Debug)]
pub struct Identifier {
    token: Token,
    value: VType,
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Debug)]
pub enum VType {
    Num(i64),
    Str(String),
    Null,
}
