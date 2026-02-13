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

#[derive(Debug)]
pub struct Where {
    pub left: Token,
    operator: Token,
    pub right: Token,
}

impl Where {
    pub fn new(left: Token, operator: Token, right: Token) -> Result<Self> {
        Ok(Self {
            left,
            operator,
            right,
        })
    }

    pub fn from_where(where_clause: &Self, operator: Token) -> Self {
        Self {
            left: where_clause.left.clone(),
            operator,
            right: where_clause.right.clone(),
        }
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

    pub fn get_value(&self) -> &str {
        if let Token::QIdent(ident) = &self.right {
            return ident;
        };
        if let Token::QIdent(ident) = &self.left {
            return ident;
        };
        panic!();
    }

    pub fn evaluate(&self, value: Option<&RType>) -> bool {
        match self.operator {
            Token::Equal => {
                if let Some(value) = value {
                    let left: RType = self.right.into_rtype();
                    return &left == value;
                } else {
                    return self.left == self.right;
                }
            }
            Token::GT => {
                if let Some(value) = value {
                    let left: RType = self.right.into_rtype();
                    &left > value
                } else {
                    self.left == self.right
                }
            }

            Token::LT => {
                if let Some(value) = value {
                    let left: RType = self.right.into_rtype();
                    &left < value
                } else {
                    self.left == self.right
                }
            }

            Token::LTEQ => {
                if let Some(value) = value {
                    let left: RType = self.right.into_rtype();
                    &left <= value
                } else {
                    self.left == self.right
                }
            }
            _ => panic!(),
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
