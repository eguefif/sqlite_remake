use crate::db::parser::function::FuncCall;
use crate::db::parser::identifier::Identifier;
use crate::db::parser::tokenizer::Token;
use itertools::Itertools;
use std::fmt;

#[derive(Debug)]
pub struct SelectStatement {
    select_clause: Select,
    from_clause: String,
    where_clause: String,
}

impl SelectStatement {
    pub fn new(select_clause: Select, from_clause: String, where_clause: String) -> Self {
        Self {
            select_clause,
            from_clause,
            where_clause,
        }
    }
}

impl fmt::Display for SelectStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.select_clause, self.from_clause, self.where_clause
        )
    }
}

#[derive(Debug)]
pub struct Select {
    token: Token,
    values: Vec<SelectItem>,
}

impl Select {
    pub fn new(token: Token) -> Self {
        Self {
            token: token,
            values: vec![],
        }
    }
}

impl fmt::Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let identifiers = self.values.iter().join(", ");
        write!(f, "{} {}", self.token, identifiers)
    }
}

#[derive(Debug)]
pub enum SelectItem {
    Function(FuncCall),
    Identifier(Identifier),
}

impl fmt::Display for SelectItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectItem::Function(func) => write!(f, "{}", func),
            SelectItem::Identifier(ident) => write!(f, "{}", ident),
        }
    }
}
