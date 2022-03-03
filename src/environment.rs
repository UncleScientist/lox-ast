use std::collections::HashMap;

use crate::error::*;
use crate::object::*;
use crate::token::*;

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &String, value: Object) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxError> {
        if let Some(object) = self.values.get(name.as_string()) {
            Ok(object.clone())
        } else {
            Err(LoxError::runtime_error(
                &name,
                &format!("Undefined variable '{}'.", name.as_string()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_type::*;

    #[test]
    fn can_define_a_variable() {
        let mut e = Environment::new();

        e.define(&"One".to_string(), Object::Bool(true));

        assert!(e.values.contains_key(&"One".to_string()));
        assert_eq!(
            e.values.get(&"One".to_string()).unwrap(),
            &Object::Bool(true)
        );
    }

    #[test]
    fn can_redefine_a_variable() {
        let mut e = Environment::new();
        e.define(&"Two".to_string(), Object::Bool(true));
        e.define(&"Two".to_string(), Object::Num(12.0));
        assert_eq!(
            e.values.get(&"Two".to_string()).unwrap(),
            &Object::Num(12.0)
        );
    }

    #[test]
    fn can_look_up_a_variable() {
        let mut e = Environment::new();
        e.define(&"Three".to_string(), Object::Str("foo".to_string()));

        let three_tok = Token::new(TokenType::Identifier, "Three".to_string(), None, 0);
        assert_eq!(e.get(&three_tok).unwrap(), Object::Str("foo".to_string()));
    }

    #[test]
    fn error_when_variable_undefined() {
        let e = Environment::new();
        let three_tok = Token::new(TokenType::Identifier, "Three".to_string(), None, 0);
        assert!(e.get(&three_tok).is_err());
    }
}
