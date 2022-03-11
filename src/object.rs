use std::fmt;


#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}



impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            Object::String(s) => write!(f, "String: \"{}\"", s),
            Object::Number(n) => write!(f, "Number: {}", n),
            Object::Boolean(b) => write!(f, "Boolean: {}", b),
            Object::Nil => write!(f, "NIL"),
        };
    }
}
