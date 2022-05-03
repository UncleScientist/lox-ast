use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::interpreter::*;
use crate::lox_class::*;
use crate::object::*;
use crate::stmt::*;
use crate::token::*;

pub struct LoxFunction {
    name: Token,
    is_initializer: bool,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Rc<Stmt>>>,
    closure: Rc<RefCell<Environment>>,
}

impl fmt::Debug for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{self}")
    }
}

impl Clone for LoxFunction {
    fn clone(&self) -> Self {
        Self {
            name: self.name.dup(),
            is_initializer: self.is_initializer,
            params: Rc::clone(&self.params),
            body: Rc::clone(&self.body),
            closure: Rc::clone(&self.closure),
        }
    }
}

impl PartialEq for LoxFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name.token_type() == other.name.token_type()
            && Rc::ptr_eq(&self.params, &other.params)
            && Rc::ptr_eq(&self.body, &other.body)
            && Rc::ptr_eq(&self.closure, &other.closure)
    }
}

impl LoxFunction {
    pub fn new(
        declaration: &FunctionStmt,
        closure: &Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            name: declaration.name.dup(),
            is_initializer,
            params: Rc::clone(&declaration.params),
            body: Rc::clone(&declaration.body),
            closure: Rc::clone(closure),
        }
    }

    pub fn bind(&self, instance: &Object) -> Object {
        let environment = RefCell::new(Environment::new_with_enclosing(Rc::clone(&self.closure)));
        environment.borrow_mut().define("this", instance.clone());
        Object::Func(Rc::new(Self {
            name: self.name.dup(),
            is_initializer: self.is_initializer,
            params: Rc::clone(&self.params),
            body: Rc::clone(&self.body),
            closure: Rc::new(environment),
        }))
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        _klass: Option<Rc<LoxClass>>,
    ) -> Result<Object, LoxResult> {
        let mut e = Environment::new_with_enclosing(Rc::clone(&self.closure));

        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            e.define(&param.as_string(), arg.clone());
        }

        match interpreter.execute_block(&self.body, e) {
            Err(LoxResult::ReturnValue { value }) => {
                if self.is_initializer {
                    self.closure.borrow().get_at(0, "this")
                } else {
                    Ok(value)
                }
            }
            Err(e) => Err(e),
            Ok(_) => {
                if self.is_initializer {
                    self.closure.borrow().get_at(0, "this")
                } else {
                    Ok(Object::Nil)
                }
            }
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.name.as_string())
    }
}
