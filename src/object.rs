use std::cmp::*;
use std::fmt;
use std::rc::Rc;

use crate::lox_class::*;
use crate::lox_function::*;
use crate::lox_instance::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Num(f64),
    Str(String),
    Bool(bool),
    Func(Rc<LoxFunction>),
    Class(Rc<LoxClass>),
    Instance(Rc<LoxInstance>),
    Nil,
    ArithmeticError,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Num(x) => write!(f, "{x}"),
            Object::Str(x) => write!(f, "{x}"),
            Object::Bool(x) => {
                if *x {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Object::Func(func) => write!(f, "{func}"),
            Object::Class(c) => write!(f, "{c}"),
            Object::Instance(i) => write!(f, "{i}"),
            Object::Nil => write!(f, "nil"),
            Object::ArithmeticError => panic!("Should not be trying to print this"),
        }
    }
}
