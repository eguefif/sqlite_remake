use anyhow::Result;
use std::fmt;

use crate::{executor::db_response::RType, parser::token::Token};

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

// TODO: refactor for a more complete way of parsing where
// Introduce the concept of condition
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

    pub fn get_identifier(&self) -> Option<&str> {
        if let Token::Ident(ident) = &self.right {
            return Some(ident);
        };
        if let Token::Ident(ident) = &self.left {
            return Some(ident);
        };
        None
    }

    pub fn evaluate(&self, value: Option<&RType>) -> bool {
        if let Some(value) = value {
            let left: RType = self.right.into_rtype();
            &left == value
        } else {
            self.left == self.right
        }
    }
}

impl fmt::Display for Where {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WHERE {} {} {}", self.left, self.operator, self.right)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn it_should_evaluate_none_1() {
        let where_clause = Where::new(
            Token::Num(5),
            Token::Equal,
            Token::QIdent("Hello".to_string()),
        )
        .unwrap();
        let result = where_clause.evaluate(None);
        assert_eq!(result, false)
    }

    #[test]
    fn it_should_evaluate_none_2() {
        let where_clause = Where::new(Token::Num(5), Token::Equal, Token::Num(1)).unwrap();
        let result = where_clause.evaluate(None);
        assert_eq!(result, true)
    }
}
