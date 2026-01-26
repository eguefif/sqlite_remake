use crate::parser::{
    query::Query,
    tokenizer::{Token, Tokenizer},
};
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

    pub fn parse(&mut self) {
        loop {
            let mut query = Query::new();
            let Some(token) = self.tokenizer.next() else {
                break;
            };

            if Token::Select != token {
                panic!("Expect Select");
            }
            loop {
                let Some(token) = self.tokenizer.next() else {
                    panic!("Unexpected end of stream token in select");
                };
                match token {
                    Token::From => break,
                    Token::Value(value) => query.push_select(value),
                    Token::Coma => {}
                    _ => panic!("Don't expect {:?}", token),
                }
            }
            let Some(token) = self.tokenizer.next() else {
                panic!("Expect a table after FROM");
            };
            if let Token::Value(value) = token {
                query.set_from(value)
            } else {
                panic!("No table for FROM");
            }
            self.queries.push(query);
        }
    }
}
