use crate::parser::identifier::Identifier;
use crate::parser::tokenizer::Token;
use crate::parser::{function::FuncCall, where_clause::Where};
use anyhow::{Result, anyhow};
use itertools::Itertools;
use std::fmt;

#[derive(Debug)]
pub struct SelectStatement {
    pub select_clause: SelectClause,
    pub from_clause: String,
    pub where_clause: Option<Where>,
}

impl SelectStatement {
    pub fn new(
        select_clause: SelectClause,
        from_clause: String,
        where_clause: Option<Where>,
    ) -> Self {
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

        if let Some(where_clause) = &self.where_clause {
            write!(f, " {}", where_clause)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SelectClause {
    token: Token,
    pub items: Vec<SelectItem>,
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

    pub fn check_select_clause(self) -> Result<()> {
        if self.items.len() > 1 {
            match self.items[0] {
                SelectItem::Function(_) => {
                    return Err(anyhow!("Executor: select clause mixes function and values"));
                }
                SelectItem::Star => {
                    return Err(anyhow!("Executor: select clause has * and other values"));
                }
                _ => {}
            }
        }
        Ok(())
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
