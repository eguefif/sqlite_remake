use crate::db::parser::identifier::Identifier;
use crate::db::parser::tokenizer::Token;
use itertools::Itertools;
use std::fmt;

#[derive(Debug)]
pub struct FuncCall {
    token: Token,
    params: Vec<Identifier>,
}

impl fmt::Display for FuncCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let identifiers = self.params.iter().join(", ");
        write!(f, "{}({})", self.token, identifiers)
    }
}
