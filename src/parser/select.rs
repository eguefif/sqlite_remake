use crate::parser::function::FuncCall;
use crate::parser::identifier::Identifier;
use crate::parser::tokenizer::Token;
use itertools::Itertools;
use std::fmt;

#[derive(Debug)]
pub struct SelectStatement {
    select_clause: SelectClause,
    from_clause: String,
    where_clause: String,
}

impl SelectStatement {
    pub fn new(select_clause: SelectClause, from_clause: String, where_clause: String) -> Self {
        Self {
            select_clause,
            from_clause,
            where_clause,
        }
    }

    pub fn add_from(&mut self, value: String) {
        self.from_clause = value;
    }
}

impl fmt::Display for SelectStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.select_clause)?;
        if self.from_clause.len() > 0 {
            write!(f, " FROM {}", self.from_clause)?;
        }

        if self.where_clause.len() > 0 {
            write!(f, " WHERE {}", self.where_clause)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SelectClause {
    token: Token,
    items: Vec<SelectItem>,
}

impl SelectClause {
    pub fn new(token: Token) -> Self {
        Self {
            token: token,
            items: vec![],
        }
    }

    pub fn push_item(&mut self, item: SelectItem) {
        self.items.push(item);
    }
}

impl fmt::Display for SelectClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let identifiers = self.items.iter().join(", ");
        write!(f, "{} {}", self.token, identifiers)
    }
}

#[derive(Debug)]
pub enum SelectItem {
    Function(FuncCall),
    Identifier(Identifier),
    Star,
}

impl fmt::Display for SelectItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectItem::Function(func) => write!(f, "{}", func),
            SelectItem::Identifier(ident) => write!(f, "{}", ident),
            SelectItem::Star => write!(f, "*"),
        }
    }
}
