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
    query::Query,
    tokenizer::{Token, Tokenizer},
};
use anyhow::{Result, anyhow};
use std::iter::Iterator;

pub mod query;
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

    fn parse_select(&mut self, mut query: Query) -> Result<Query> {
        loop {
            // TODO: Handle function token, a function has a list of params
            // It expects no values after nor comas
            let Some(peek) = self.tokenizer.peek() else {
                return Err(anyhow!("Error: parser: unexpected EOF"));
            };
            match peek {
                Token::Ident(_) => {
                    if let Token::Ident(value) = self.tokenizer.next().unwrap() {
                        query.push_select(value);
                    } else {
                        panic!("Unreachable code reached");
                    }
                }
                Token::Coma => {
                    self.tokenizer.next().unwrap();
                }
                _ => break,
            }
        }
        Ok(query)
    }

    fn parse_from(&mut self, mut query: Query) -> Result<Query> {
        self.expect_token(Token::From)?;
        if let Some(token) = self.tokenizer.next() {
            if let Token::Ident(value) = token {
                query.set_from(value)
            } else {
                return Err(anyhow!("Error: parser: no table for FROM"));
            }
        }
        Ok(query)
    }

    fn try_parse_where(&mut self, mut query: Query) -> Result<Query> {
        let Some(peek) = self.tokenizer.peek() else {
            return Ok(query);
        };
        if *peek == Token::Where {
            self.tokenizer.next();
            let left = self.tokenizer.next().unwrap();
            let operator = self.tokenizer.next().unwrap();
            let right = self.tokenizer.next().unwrap();
            query.push_where(left, operator, right);
            return Ok(query);
        }

        Ok(query)
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
}

impl Iterator for Parser<'_> {
    type Item = Result<Query>;

    // Parse the next query
    fn next(&mut self) -> Option<Self::Item> {
        if let None = self.tokenizer.peek() {
            return None;
        }
        let query = Query::new();
        if let Err(error) = self.expect_token(Token::Select) {
            return Some(Err(error));
        }

        let Ok(query) = self.parse_select(query) else {
            return Some(Err(anyhow!("Error: parser: SELECT clause malformed")));
        };
        let Ok(query) = self.parse_from(query) else {
            return Some(Err(anyhow!("Error: parser: FROM clause malformed")));
        };

        let Ok(query) = self.try_parse_where(query) else {
            return Some(Err(anyhow!("Error: parser: WHERE clause malformed")));
        };

        // Expect a semicolon at the end of the query
        if let Err(error) = self.expect_token(Token::SemiColon) {
            // If there is token left and not semicolon, return an error
            if let Some(_) = self.tokenizer.peek() {
                return Some(Err(error));
            }
            // If there was no semicolon but no token left, then it's ok
        }
        Some(Ok(query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::parser::query::{Operator, SelectType, Statement};

    #[test]
    fn it_should_parse_query() {
        let mut parser = Parser::new("SELECT name, color FROM apples WHERE name = 'hey';");
        let expected_query = Query {
            select: vec![
                SelectType::Value("name".to_string()),
                SelectType::Value("color".to_string()),
            ],
            from: "apples".to_string(),
            wh: vec![Statement {
                left: Token::Ident("name".to_string()),
                operator: Operator::Equal,
                right: Token::QIdent("hey".to_string()),
            }],
        };
        let query = parser.next().unwrap().unwrap();

        assert_eq!(query, expected_query);
    }
}
