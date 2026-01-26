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

    pub fn parse(&mut self) -> Result<()> {
        loop {
            let mut query = Query::new();
            if let None = self.expect_token(Token::Select)? {
                break;
            }
            query = self.parse_select(query)?;
            let Some(token) = self.tokenizer.next() else {
                return Err(anyhow!("Error parsing: unexpected EOF"));
            };
            if let Token::Value(value) = token {
                query.set_from(value)
            } else {
                return Err(anyhow!("Error parsing: no table for FROM"));
            }
            self.queries.push(query);
        }

        Ok(())
    }

    fn parse_select(&mut self, mut query: Query) -> Result<Query> {
        loop {
            let Some(token) = self.tokenizer.next() else {
                return Err(anyhow!("Error parsing: unexpected EOF"));
            };
            match token {
                Token::From => break,
                Token::Value(value) => query.push_select(value),
                Token::Coma => {}
                _ => return Err(anyhow!("Parsing: unexpected token {}", token)),
            }
        }
        Ok(query)
    }

    fn expect_token(&mut self, expected_token: Token) -> Result<Option<Token>> {
        if let Some(next) = self.tokenizer.next() {
            if next == expected_token {
                return Ok(Some(next));
            } else {
                return Err(anyhow!(
                    "Expected token {} but got {}",
                    expected_token,
                    next
                ));
            }
        }
        Ok(None)
    }

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
