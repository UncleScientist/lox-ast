use std::rc::Rc;

use crate::callable::*;
use crate::error::*;
use crate::interpreter::*;
use crate::lox_instance::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: &String) -> Self {
        Self { name: name.clone() }
    }

    pub fn instantiate(
        &self,
        _interpreter: &Interpreter,
        _arguments: Vec<Object>,
        klass: Rc<LoxClass>,
    ) -> Result<Object, LoxResult> {
        Ok(Object::Instance(LoxInstance::new(klass)))
    }
}

/*
impl std::string::ToString for LoxClass {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
*/

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

    fn to_string(&self) -> String {
        self.name.clone()
    }
}
