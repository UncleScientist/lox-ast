use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::error::*;
use crate::lox_class::*;
use crate::object::*;
use crate::token::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    klass: Rc<LoxClass>,
    fields: RefCell<HashMap<String, Object>>,
}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> Self {
        Self {
            klass: Rc::clone(&klass),
            fields: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &Token, this: &Rc<LoxInstance>) -> Result<Object, LoxResult> {
        if let Entry::Occupied(o) = self.fields.borrow_mut().entry(name.as_string()) {
            Ok(o.get().clone())
        } else if let Some(method) = self.klass.find_method(&name.as_string()) {
            if let Object::Func(func) = method {
                Ok(func.bind(&Object::Instance(Rc::clone(this))))
            } else {
                panic!("tried to bind 'this' to a non-function {method:?}");
            }
        } else {
            Err(LoxResult::runtime_error(
                name,
                &format!("Undefined property '{}'.", name.as_string()),
            ))
        }
    }

    pub fn set(&self, name: &Token, value: Object) {
        self.fields.borrow_mut().insert(name.as_string(), value);
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance", self.klass)
    }
}
