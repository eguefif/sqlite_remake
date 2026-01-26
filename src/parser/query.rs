//! A simple representation of a database query.
//! It supports SELECT and FROM clauses.
use std::fmt;

#[allow(dead_code)]
pub struct Query {
    pub select: Vec<String>,
    pub from: String,
}

impl Query {
    pub fn new() -> Self {
        Self {
            select: vec![],
            from: "".to_string(),
        }
    }

    pub fn push_select(&mut self, value: String) {
        self.select.push(value);
    }

    pub fn set_from(&mut self, from: String) {
        self.from = from;
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut select = String::new();
        for (i, value) in self.select.iter().enumerate() {
            if i != 0 {
                select.push(' ');
            }
            select.push_str(value);
        }
        write!(f, "Query: SELECT {} FROM {}", select, self.from)
    }
}
