use anyhow::Result;
use std::fmt;

use crate::parser::tokenizer::Token;

#[derive(Debug)]
pub enum Operator {
    Eq,
    NotEq,
    LT,
    GT,
    LTE,
    GTE,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operator::Eq => write!(f, "="),
            Operator::NotEq => write!(f, "!="),
            Operator::LT => write!(f, "<"),
            Operator::GT => write!(f, ">"),
            Operator::LTE => write!(f, "<="),
            Operator::GTE => write!(f, ">="),
        }
    }
}

#[derive(Debug)]
pub struct Where {
    left: Token,
    operator: Token,
    right: Token,
}

impl Where {
    pub fn new(left: Token, operator: Token, right: Token) -> Result<Self> {
        Ok(Self {
            left,
            operator,
            right,
        })
    }
}

impl fmt::Display for Where {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WHERE {} {} {}", self.left, self.operator, self.right)
    }
}
