use std::fmt;

#[derive(Debug, Clone)]
pub enum Object {
    Num(f64),
    Str(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Num(x) => write!(f, "{x}"),
            Object::Str(x) => write!(f, "\"{x}\""),
            Object::Bool(x) => {
                if *x {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Object::Nil => write!(f, "nil"),
        }
    }
}
