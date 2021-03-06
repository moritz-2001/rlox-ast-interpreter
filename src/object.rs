use crate::callable::Callable;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::class::LoxInstance;
use crate::{LoxClass, LoxError};

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
    Callable(Rc<Box<dyn Callable>>),
    Instance(LoxInstance),
    Class(LoxClass),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Number(n) => write!(f, "{}", n),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Nil => write!(f, "NIL"),
            Object::Callable(o) => write!(f, "{}", o),
            Object::Instance(c) => write!(f, "Instance <{:?}>", c),
            Object::Class(c) => write!(f, "Class <{}>", c.name()),
        };
    }
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        if *self == Object::Nil {
            return false;
        }
        if let Object::Boolean(b) = self {
            return *b;
        };

        true
    }

    pub fn is_equal(a: Object, b: Object) -> bool {
        match (a, b) {
            (Object::Nil, Object::Nil) => true,
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::Number(a), Object::Number(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            _ => false,
        }
    }

    pub fn get_v_num(&self) -> Result<f64, LoxError> {
        if let Object::Number(n) = self {
            Ok(*n)
        } else {
            Err(LoxError::Error(format!("'{:?}' must be a number.", self)))
        }
    }

    pub fn get_v_string(&self) -> Result<String, LoxError> {
        if let Object::String(s) = self {
            Ok(s.clone())
        } else {
            Err(LoxError::Error(format!("'{:?}' must be a string.", self)))
        }
    }
}
