use crate::object::*;
use crate::token::*;
use crate::token_type::*;

pub enum LoxResult {
    ParseError { token: Token, message: String },
    RuntimeError { token: Token, message: String },
    Error { line: usize, message: String },
    SystemError { message: String },
    ReturnValue { value: Object },
    Break,
    Fail,
}

impl LoxResult {
    pub fn fail() -> LoxResult {
        LoxResult::Fail
    }

    pub fn return_value(value: Object) -> LoxResult {
        LoxResult::ReturnValue { value }
    }

    pub fn error(line: usize, message: &str) -> LoxResult {
        let err = LoxResult::Error {
            line,
            message: message.to_string(),
        };
        err.report("");
        err
    }

    pub fn parse_error(token: &Token, message: &str) -> LoxResult {
        let err = LoxResult::ParseError {
            token: token.dup(),
            message: message.to_string(),
        };
        err.report("");
        err
    }

    pub fn runtime_error(token: &Token, message: &str) -> LoxResult {
        let err = LoxResult::RuntimeError {
            token: token.dup(),
            message: message.to_string(),
        };
        err.report("");
        err
    }

    pub fn system_error(message: &str) -> LoxResult {
        let err = LoxResult::SystemError {
            message: message.to_string(),
        };
        err.report("");
        err
    }

    fn report(&self, loc: &str) {
        match self {
            LoxResult::ParseError { token, message } => {
                if token.is(TokenType::Eof) {
                    eprintln!("[line {}] Error at end: {}", token.line, message);
                } else {
                    eprintln!(
                        "[line {}] Error at '{}': {}",
                        token.line,
                        token.as_string(),
                        message
                    );
                }
            }
            LoxResult::RuntimeError { token, message } => {
                if token.is(TokenType::Eof) {
                    eprintln!("[line {}] Error at end: {}", token.line, message);
                } else {
                    eprintln!("{}\n[line {}]", message, token.line);
                }
            }
            LoxResult::Error { line, message } => {
                eprintln!("[line {}] Error{}: {}", line, loc, message);
            }
            LoxResult::SystemError { message } => {
                eprintln!("System Error: {message}");
            }
            LoxResult::Break | LoxResult::ReturnValue { .. } => {}
            LoxResult::Fail => {
                panic!("should not get here")
            }
        };
    }
}
