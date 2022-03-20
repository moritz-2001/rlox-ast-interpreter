use crate::callable::{Callable, LoxFunction};
use crate::interpreter::Interpreter;
use crate::lox_error::LoxError;
use crate::tokens::Token;
use crate::Environment;
use crate::Object;

use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
        Self { name, methods }
    }

    fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        self.methods.get(name)
    }
}

impl Callable for LoxClass {
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, LoxError> {
        let instance = LoxInstance::new(self.clone());
        Ok(Object::Instance(instance))
    }

    fn arity(&self) -> usize {
        if let Some(init) = self.find_method("init") {
            init.arity()
        } else {
            0
        }
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, Object>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxError> {
        if let Some(o) = self.fields.get(&name.lexeme) {
            Ok(o.clone())
        } else if let Some(o) = self.class.methods.get(&name.lexeme) {
            Ok(Object::Callable(Rc::new(Box::new(o.bind(self)))))
        } else {
            Err(LoxError::Error(format!(
                "Undefined property '{}'.",
                name.lexeme
            )))
        }
    }

    pub fn set(&mut self, name: &Token, value: Object) -> Result<(), LoxError> {
        self.fields.insert(name.lexeme.clone(), value);
        Ok(())
    }
}

impl ToString for LoxInstance {
    fn to_string(&self) -> String {
        format!("{} instance", self.class.name())
    }
}
