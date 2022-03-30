use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::interpreter::*;
use crate::object::*;
use crate::stmt::*;
use crate::token::*;

pub struct LoxFunction {
    name: Token,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Rc<Stmt>>>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(declaration: &FunctionStmt, closure: &Rc<RefCell<Environment>>) -> Self {
        Self {
            name: declaration.name.dup(),
            params: Rc::clone(&declaration.params),
            body: Rc::clone(&declaration.body),
            closure: Rc::clone(closure),
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult> {
        let mut e = Environment::new_with_enclosing(Rc::clone(&self.closure));

        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            e.define(&param.as_string(), arg.clone());
        }

        match interpreter.execute_block(&self.body, e) {
            Err(LoxResult::ReturnValue { value }) => Ok(value),
            Err(e) => Err(e),
            Ok(_) => Ok(Object::Nil),
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let paramlist = self
            .params
            .iter()
            .map(|p| p.as_string())
            .collect::<Vec<String>>()
            .join(", ");

        // <Function foo(a, b, c)>
        write!(f, "<Function {}({paramlist})>", self.name.as_string())
    }
}
