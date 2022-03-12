use std::fmt;

use crate::{interpreter::Interpreter, LoxError, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
    Callable(Function),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Number(n) => write!(f, "{}", n),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Nil => write!(f, "NIL"),
            Object::Callable(o) => write!(f, "ERR"),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    name: Box<Object>,
    pub args: Vec<Token>,
    pub arity: usize,
    pub interpreter: Option<Interpreter>,
    native_function: Option<fn(Vec<Object>) -> Result<Object, LoxError>>,
}

impl Function {
    pub fn new(
        name: Object,
        args: Vec<Token>,
        arity: usize,
        interpreter: Option<Interpreter>,
        native_fn: Option<fn(Vec<Object>) -> Result<Object, LoxError>>,
    ) -> Self {
        Self {
            name: Box::new(name),
            args,
            arity,
            interpreter,
            native_function: native_fn,
        }
    }

    pub fn call(&mut self, args: Vec<Object>) -> Result<Object, LoxError> {
        if let Some(fun) = self.native_function {
            return Ok(fun(args)?);
        }

        let mut int = self.interpreter.take().unwrap();
        self.interpreter = Some(int.clone());

        for (i, v) in args.into_iter().enumerate() {
            int.env.define(self.args.get(i).unwrap().lexeme.clone(), v);
        }
         

        Ok(int.run()?)
    }
}
