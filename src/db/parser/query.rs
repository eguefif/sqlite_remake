//! A simple representation of a database query.
//! It supports SELECT and FROM clauses.
use crate::db::parser::tokenizer::Token;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum SelectType {
    Function(String),
    Value(String),
}

impl fmt::Display for SelectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectType::Function(func) => write!(f, "{}", func),
            SelectType::Value(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    GT,
    LT,
    GTEQ,
    LTEQ,
}

impl Operator {
    pub fn from_token(token: Token) -> Self {
        match token {
            Token::Equal => Operator::Equal,
            Token::NotEq => Operator::NotEqual,
            Token::GT => Operator::GT,
            Token::LT => Operator::LT,
            Token::LTEQ => Operator::GTEQ,
            Token::GTEQ => Operator::LTEQ,
            _ => panic!("Not a valid token operator"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Query {
    pub select: Vec<SelectType>,
    pub from: String,
    pub wh: Vec<Statement>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            select: vec![],
            from: "".to_string(),
            wh: vec![],
        }
    }

    pub fn push_select(&mut self, value: String) {
        if value.to_lowercase().contains("count") {
            self.select.push(SelectType::Function(value));
        } else {
            self.select.push(SelectType::Value(value));
        }
    }

    pub fn push_where(&mut self, left: Token, operator: Token, right: Token) {
        self.wh.push(Statement {
            left,
            operator: Operator::from_token(operator),
            right,
        })
    }

    pub fn set_from(&mut self, from: String) {
        self.from = from;
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let select = self
            .select
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "Query: SELECT {} FROM {}", select, self.from)
    }
}

// TODO: left and right need to be statement
#[derive(Debug, PartialEq)]
pub struct Statement {
    pub left: Token,
    pub operator: Operator,
    pub right: Token,
}
