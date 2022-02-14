use crate::error::*;
use crate::token::*;
use crate::token_type::*;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, LoxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::eof(self.line));

        Ok(&self.tokens)
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_token(&self) {}
}
