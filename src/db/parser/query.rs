//! A simple representation of a database query.
//! It supports SELECT and FROM clauses.
use crate::db::db_response::RType;
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
        let Token::Ident(left) = left else {
            panic!("Error: Query push where: invalid left Token");
        };
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
    pub left: String,
    pub operator: Operator,
    pub right: Token,
}

impl Statement {
    pub fn cmp(&self, rhs: RType) -> bool {
        match self.operator {
            Operator::Equal => self.eq(rhs),
            Operator::NotEqual => !self.eq(rhs),
            Operator::GT => self.gt(rhs),
            Operator::LT => self.lt(rhs),
            Operator::GTEQ => self.gte(rhs),
            Operator::LTEQ => self.lte(rhs),
        }
    }

    fn eq(&self, rhs: RType) -> bool {
        match self.right {
            Token::Num(right) => match rhs {
                RType::Num(left) => left == right,
                _ => false,
            },
            Token::QIdent(ref right) => match rhs {
                RType::Str(left) => left == *right,
                _ => false,
            },
            _ => panic!("Should not compare here"),
        }
    }

    fn gt(&self, rhs: RType) -> bool {
        match self.right {
            Token::Num(right) => match rhs {
                RType::Num(left) => left > right,
                _ => false,
            },
            Token::QIdent(ref right) => match rhs {
                RType::Str(ref left) => left < right,
                _ => false,
            },
            _ => panic!("Should not compare here"),
        }
    }

    fn lt(&self, rhs: RType) -> bool {
        match self.right {
            Token::Num(right) => match rhs {
                RType::Num(left) => left < right,
                _ => false,
            },
            Token::QIdent(ref right) => match rhs {
                RType::Str(left) => left < *right,
                _ => false,
            },
            _ => panic!("Should not compare here"),
        }
    }

    fn lte(&self, rhs: RType) -> bool {
        match self.right {
            Token::Num(right) => match rhs {
                RType::Num(left) => left <= right,
                _ => false,
            },
            Token::QIdent(ref right) => match rhs {
                RType::Str(left) => left <= *right,
                _ => false,
            },
            _ => panic!("Should not compare here"),
        }
    }

    fn gte(&self, rhs: RType) -> bool {
        match self.right {
            Token::Num(right) => match rhs {
                RType::Num(value) => value >= right,
                _ => false,
            },
            Token::QIdent(ref right) => match rhs {
                RType::Str(left) => left <= *right,
                _ => false,
            },
            _ => panic!("Should not compare here"),
        }
    }
}
