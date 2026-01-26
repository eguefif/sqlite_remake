use crate::parser::{
    query::Query,
    tokenizer::{Token, Tokenizer},
};
use anyhow::{Result, anyhow};
pub mod query;
pub mod tokenizer;

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    pub queries: Vec<Query>,
}

impl<'a> Parser<'a> {
    pub fn new(query_str: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(query_str),
            queries: vec![],
        }
    }

    // Parse the input query string into a list of Query objects
    // Each Query starts with a SELECT token and ends when the 
    // tokenizer returns None
    pub fn parse(&mut self) -> Result<()> {
        loop {
            let mut query = Query::new();
            self.expect_token(Token::Select)?;

            query = self.parse_select(query)?;
            query = self.parse_from(query)?;
            self.queries.push(query);

            if let None = self.tokenizer.peek() {
                break;
            }
            self.expect_token(Token::SemiColon)?;
        }

        Ok(())
    }

    fn parse_select(&mut self, mut query: Query) -> Result<Query> {
        loop {
            let Some(peek) = self.tokenizer.peek() else {
                return Err(anyhow!("Error parsing: unexpected EOF"));
            };
            match peek {
                Token::Value(_) => {
                    let Token::Value(value) = self.tokenizer.next().unwrap() else {
                        panic!("Unreachable code");
                    };
                    query.push_select(value);
                }
                Token::Coma => {}
                _ => break,
            }
        }
        Ok(query)
    }

    fn parse_from(&mut self, mut query: Query) -> Result<Query> {
        self.expect_token(Token::From)?;
        if let Some(token) = self.tokenizer.next() {
            if let Token::Value(value) = token {
                query.set_from(value)
            } else {
                return Err(anyhow!("Error parsing: no table for FROM"));
            }
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
