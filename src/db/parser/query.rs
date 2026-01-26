//! A simple representation of a database query.
//! It supports SELECT and FROM clauses.
use std::fmt;

#[derive(Debug)]
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

pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Query {
    pub select: Vec<SelectType>,
    pub from: String,
    //pub where: Option<(String, String, Operator)>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            select: vec![],
            from: "".to_string(),
            //where: None
        }
    }

    pub fn push_select(&mut self, value: String) {
        if value.to_lowercase().contains("count") {
            self.select.push(SelectType::Function(value));
        } else {
            self.select.push(SelectType::Value(value));
        }
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
