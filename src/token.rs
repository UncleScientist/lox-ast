use crate::object::Object;
use crate::token_type::*;
use std::fmt;

#[derive(Debug)]
pub struct Token {
    ttype: TokenType,
    lexeme: String,
    pub literal: Option<Object>,
    pub line: usize,
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, literal: Option<Object>, line: usize) -> Token {
        Token {
            ttype,
            lexeme,
            literal,
            line,
        }
    }

    pub fn is(&self, ttype: TokenType) -> bool {
        self.ttype == ttype
    }

    pub fn token_type(&self) -> TokenType {
        self.ttype
    }

    pub fn as_string(&self) -> String {
        self.lexeme.clone()
    }

    pub fn dup(&self) -> Token {
        Token {
            ttype: self.ttype,
            lexeme: self.lexeme.to_string(),
            literal: self.literal.clone(),
            line: self.line,
        }
    }

    pub fn eof(line: usize) -> Token {
        Token {
            ttype: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} {} {}",
            self.ttype,
            self.lexeme,
            if let Some(literal) = &self.literal {
                literal.to_string()
            } else {
                "None".to_string()
            }
        )
    }
}

/*
 * possibly:
pub enum Token {
    Literal { lexeme: String, literal: <...> },
    Keyword { lexeme: String, ttype: String, },
}
*/
