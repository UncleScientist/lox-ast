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

    pub fn get(&self, name: &Token) -> Result<Object, LoxResult> {
        if let Entry::Occupied(o) = self.fields.borrow_mut().entry(name.as_string()) {
            Ok(o.get().clone())
        } else if let Some(method) = self.klass.find_method(&name.as_string()) {
            Ok(method)
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
        let mut fields = Vec::new();

        for (k, v) in self.fields.borrow().iter() {
            fields.push(format!("{k}={v}"))
        }

        write!(
            f,
            "<Instance of {} {{ {} }}>",
            self.klass.to_string(),
            fields.join(", ")
        )
    }
}
