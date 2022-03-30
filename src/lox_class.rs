use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::callable::*;
use crate::error::*;
use crate::interpreter::*;
use crate::lox_instance::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Object>,
}

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, Object>) -> Self {
        Self {
            name: name.to_string(),
            methods,
        }
    }

    pub fn instantiate(
        &self,
        _interpreter: &Interpreter,
        _arguments: Vec<Object>,
        klass: Rc<LoxClass>,
    ) -> Result<Object, LoxResult> {
        Ok(Object::Instance(Rc::new(LoxInstance::new(klass))))
    }

    pub fn find_method(&self, name: &str) -> Option<Object> {
        self.methods.get(name).cloned()
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let methods = self
            .methods
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "<Class {} {{ {methods} }}>", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn call(
        &self,
        _interpreter: &Interpreter,
        _arguments: Vec<Object>,
    ) -> Result<Object, LoxResult> {
        Err(LoxResult::system_error("tried to call a class"))
    }

    fn arity(&self) -> usize {
        0
    }
}
