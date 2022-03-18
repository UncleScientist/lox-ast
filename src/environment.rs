use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::*;
use crate::object::*;
use crate::token::*;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxResult> {
        if let Some(object) = self.values.get(&name.as_string()) {
            Ok(object.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(LoxResult::runtime_error(
                name,
                &format!("Undefined variable '{}'.", name.as_string()),
            ))
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), LoxResult> {
        if let Entry::Occupied(mut object) = self.values.entry(name.as_string()) {
            object.insert(value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(LoxResult::runtime_error(
                name,
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
        assert_eq!(e.get(&three_tok).ok(), Some(Object::Str("foo".to_string())));
    }

    #[test]
    fn error_when_variable_undefined() {
        let e = Environment::new();
        let three_tok = Token::new(TokenType::Identifier, "Three".to_string(), None, 0);
        assert!(e.get(&three_tok).is_err());
    }

    #[test]
    fn error_when_assigning_to_undefined_variable() {
        let mut e = Environment::new();
        let four_tok = Token::new(TokenType::Identifier, "Four".to_string(), None, 0);
        assert!(e.assign(&four_tok, Object::Nil).is_err());
    }

    #[test]
    fn can_reassign_existing_variable() {
        let mut e = Environment::new();
        let four_tok = Token::new(TokenType::Identifier, "Four".to_string(), None, 0);
        e.define(&"Four".to_string(), Object::Num(73.1));
        assert!(e.assign(&four_tok, Object::Num(89.5)).is_ok());
        assert_eq!(e.get(&four_tok).ok(), Some(Object::Num(89.5)));
    }

    #[test]
    fn can_enclose_an_environment() {
        let e = Rc::new(RefCell::new(Environment::new()));
        let f = Environment::new_with_enclosing(Rc::clone(&e));
        assert_eq!(f.enclosing.unwrap().borrow().values, e.borrow().values);
    }

    #[test]
    fn can_read_from_enclosed_environment() {
        let e = Rc::new(RefCell::new(Environment::new()));
        e.borrow_mut()
            .define(&"Five".to_string(), Object::Num(77.8));

        let f = Environment::new_with_enclosing(Rc::clone(&e));
        let five_tok = Token::new(TokenType::Identifier, "Five".to_string(), None, 0);
        assert_eq!(f.get(&five_tok).ok(), Some(Object::Num(77.8)));
    }

    #[test]
    fn can_assign_to_enclosed_environment() {
        let e = Rc::new(RefCell::new(Environment::new()));
        e.borrow_mut()
            .define(&"Five".to_string(), Object::Num(77.8));

        let mut f = Environment::new_with_enclosing(Rc::clone(&e));
        let five_tok = Token::new(TokenType::Identifier, "Five".to_string(), None, 0);
        assert!(f.assign(&five_tok, Object::Num(91.2)).is_ok());
        assert_eq!(f.get(&five_tok).ok(), Some(Object::Num(91.2)));
    }
}
