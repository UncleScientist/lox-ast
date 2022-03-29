use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
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

    pub fn get(&self, name: &Token) -> Result<Object, LoxResult> {
        if let Entry::Occupied(o) = self.fields.borrow_mut().entry(name.as_string()) {
            Ok(o.get().clone())
        } else {
            Err(LoxResult::runtime_error(
                name,
                &format!("Undefined property '{}'.", name.as_string()),
            ))
        }
    }
}

impl std::string::ToString for LoxInstance {
    fn to_string(&self) -> String {
        format!("<Instance of {}>", self.klass.to_string())
    }
}
