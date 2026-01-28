//! Internal module to parse SQL and build queries.
//! It supports SELECT and FROM clauses.
//!
//! # Example
//! ```
//! use crate::parser::Parser;
//! let query_str = "SELECT name, age FROM users;";
//! let mut parser = Parser::new(query_str);
//! for query in parser {
//!    let query = query.unwrap();
//!    println!("{}", query);
//!    }
//! ```
use crate::db::parser::{
    function::FuncCall,
    identifier::{Identifier, VType},
    select::{SelectClause, SelectItem, SelectStatement},
    statement::Statement,
    tokenizer::{Token, Tokenizer},
};
use anyhow::{Result, anyhow};
use std::iter::Iterator;

pub mod function;
pub mod identifier;
pub mod select;
pub mod statement;
pub mod tokenizer;

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}
impl<'a> Parser<'a> {
    pub fn new(query_str: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(query_str),
        }
    }

    // If the next token matches the expected token, consume it and return it
    // if not, returns an error
    // If there is no next token, return None
    fn expect_token(&mut self, expected_token: Token) -> Result<Token> {
        if let Some(next) = self.tokenizer.next() {
            if next == expected_token {
                return Ok(next);
            } else {
                return Err(anyhow!(
                    "Expected token {} but got {}",
                    expected_token,
                    next
                ));
            }
        }
        Err(anyhow!("Expected token {} but got EOF", expected_token))
    }

    // Peek the next token and check if it matches the expected token
    // returns an error if it does not match or if there is no next token
    // or EOF
    #[allow(dead_code)]
    fn expect_token_peek(&mut self, expected_token: Token) -> Result<()> {
        if let Some(peeked) = self.tokenizer.peek() {
            if *peeked == expected_token {
                return Ok(());
            } else {
                return Err(anyhow!(
                    "Expected token {} but got {}",
                    expected_token,
                    peeked
                ));
            }
        }
        Err(anyhow!("Expected token {} but got EOF", expected_token))
    }

    fn parse_select_statement(&mut self, token: Token) -> Result<Statement> {
        let select_clause = self.parse_select_clause(token)?;
        let mut select_statement =
            SelectStatement::new(select_clause, "".to_string(), "".to_string());

        let Some(next) = self.tokenizer.next() else {
            return Ok(Statement::Select(select_statement));
        };

        if next == Token::From {
            self.parse_from(&mut select_statement)?;
        } else {
            return Err(anyhow!("Parsing: expected From got {}", next));
        }

        Ok(Statement::Select(select_statement))
    }

    fn parse_select_clause(&mut self, token: Token) -> Result<SelectClause> {
        let mut select = SelectClause::new(token);

        let select_items = self.parse_select_values()?;
        for item in select_items {
            select.push_item(item);
        }

        Ok(select)
    }

    fn parse_select_values(&mut self) -> Result<Vec<SelectItem>> {
        let mut select_items = vec![];
        loop {
            let Some(next) = self.tokenizer.next() else {
                return Err(anyhow!("Parsing Select Clause: expect token got EOF"));
            };
            match next {
                Token::Ident(value) => {
                    if self.is_function(&value) {
                        let function = self.parse_function(value)?;
                        select_items.push(function);
                    } else {
                        let identifier = Identifier {
                            value: VType::Str(value),
                        };
                        select_items.push(SelectItem::Identifier(identifier));
                    }
                }
                Token::Star => select_items.push(SelectItem::Star),
                _ => {
                    return Err(anyhow!(
                        "Parsing Select Clause: expect column name got {}",
                        next
                    ));
                }
            }

            if let Err(_) = self.expect_token_peek(Token::Coma) {
                break;
            }
            self.tokenizer.next();
        }
        Ok(select_items)
    }

    fn is_function(&self, function_name: &str) -> bool {
        match function_name {
            "count" => true,
            _ => false,
        }
    }

    fn parse_function(&mut self, function_name: String) -> Result<SelectItem> {
        self.expect_token(Token::LParen)?;
        let items = self.parse_select_values()?;
        self.expect_token(Token::RParen)?;
        Ok(SelectItem::Function(FuncCall::new(function_name, items)))
    }

    fn parse_from(&mut self, select_statement: &mut SelectStatement) -> Result<()> {
        let Some(next) = self.tokenizer.next() else {
            return Err(anyhow!("Parsing: expected table in FROM statement got EOF",));
        };
        if let Token::Ident(value) = next {
            select_statement.add_from(value);
        } else {
            return Err(anyhow!("Parsing:: expect table identifier got: {}", next));
        }
        Ok(())
    }
}

impl Iterator for Parser<'_> {
    type Item = Result<Statement>;

    // Parse the next query
    fn next(&mut self) -> Option<Self::Item> {
        let Some(token) = self.tokenizer.next() else {
            return None;
        };
        let stmt = match token {
            Token::Select => self.parse_select_statement(token),
            _ => todo!("Do update and insert ..."),
        };

        Some(stmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_parse_select_regular_cols_name() {
        let query = "SELECT name, color, age";
        let mut parser = Parser::new("SELECT name, color, age");

        let parsed_query = parser.next().unwrap().unwrap();
        let result = format!("{}", parsed_query);
        assert_eq!(query, result)
    }

    #[test]
    fn it_should_parse_select_with_count() {
        let query = "SELECT COUNT(*)";
        let mut parser = Parser::new("SELECT COUNT(*)");

        let parsed_query = parser.next().unwrap().unwrap();
        let result = format!("{}", parsed_query);
        assert_eq!(query, result)
    }

    #[test]
    fn it_should_parse_from() {
        let query = "SELECT COUNT(*) FROM apples";
        let mut parser = Parser::new("SELECT COUNT(*) FROM apples");

        let parsed_query = parser.next().unwrap().unwrap();
        let result = format!("{}", parsed_query);
        assert_eq!(query, result)
    }
}
