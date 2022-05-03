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
    superclass: Option<Rc<LoxClass>>,
}

impl LoxClass {
    pub fn new(
        name: &str,
        superclass: Option<Rc<LoxClass>>,
        methods: HashMap<String, Object>,
    ) -> Self {
        Self {
            name: name.to_string(),
            methods,
            superclass,
        }
    }

    pub fn instantiate(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        klass: Rc<LoxClass>,
    ) -> Result<Object, LoxResult> {
        let instance = Object::Instance(Rc::new(LoxInstance::new(klass)));
        if let Some(Object::Func(initializer)) = self.find_method("init") {
            if let Object::Func(init) = initializer.bind(&instance) {
                init.call(interpreter, arguments, None)?;
            }
        }
        Ok(instance)
    }

    pub fn find_method(&self, name: &str) -> Option<Object> {
        if let Some(method) = self.methods.get(name) {
            Some(method.clone())
        } else if let Some(superclass) = &self.superclass {
            superclass.find_method(name)
        } else {
            None
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        klass: Option<Rc<LoxClass>>,
    ) -> Result<Object, LoxResult> {
        self.instantiate(interpreter, arguments, klass.unwrap())
    }

    fn arity(&self) -> usize {
        if let Some(Object::Func(initializer)) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }
}
