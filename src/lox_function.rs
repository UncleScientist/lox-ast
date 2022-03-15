use std::ops::Deref;
use std::rc::Rc;

use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::interpreter::*;
use crate::object::*;
use crate::stmt::*;

pub struct LoxFunction {
    declaration: Rc<FunctionStmt>,
}

impl LoxFunction {
    pub fn new(declaration: &Rc<FunctionStmt>) -> Self {
        Self {
            declaration: Rc::clone(declaration),
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult> {
        let mut e = Environment::new_with_enclosing(Rc::clone(&interpreter.globals));

        for (param, arg) in self.declaration.params.iter().zip(arguments.iter()) {
            e.define(param.as_string(), arg.clone());
        }

        interpreter.execute_block(&self.declaration.body, e)?;
        Ok(Object::Nil)
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn to_string(&self) -> String {
        self.declaration.name.as_string().into()
    }
}
