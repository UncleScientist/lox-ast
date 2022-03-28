use crate::callable::*;
use crate::error::*;
use crate::interpreter::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: &String) -> Self {
        Self { name: name.clone() }
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
        Ok(Object::Num(45.67))
    }

    fn arity(&self) -> usize {
        0
    }

    fn to_string(&self) -> String {
        self.name.clone()
    }
}
