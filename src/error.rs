use crate::token::*;
use crate::token_type::*;

#[derive(Debug)]
pub struct LoxError {
    token: Option<Token>,
    line: usize,
    message: String,
}

impl LoxError {
    pub fn error(line: usize, message: &str) -> LoxError {
        let err = LoxError {
            token: None,
            line,
            message: message.to_string(),
        };
        err.report("");
        err
    }

    pub fn parse_error(token: &Token, message: &str) -> LoxError {
        let err = LoxError {
            token: Some(token.dup()),
            line: token.line,
            message: message.to_string(),
        };
        err.report("");
        err
    }

    pub fn runtime_error(token: &Token, message: &str) -> LoxError {
        let err = LoxError {
            token: Some(token.dup()),
            line: token.line,
            message: message.to_string(),
        };
        err.report("");
        err
    }

    pub fn report(&self, loc: &str) {
        if let Some(token) = &self.token {
            if token.is(TokenType::Eof) {
                eprintln!("{} at end {}", token.line, self.message);
            } else {
                eprintln!("{} at '{}' {}", token.line, token.as_string(), self.message);
            }
        } else {
            eprintln!("[line {}] Error{}: {}", self.line, loc, self.message);
        }
    }
}
