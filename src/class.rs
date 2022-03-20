use crate::callable::{Callable, LoxFunction};
use crate::interpreter::Interpreter;
use crate::lox_error::LoxError;
use crate::tokens::Token;
use crate::Object;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, LoxFunction>,
    super_class: Option<Box<LoxClass>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        super_class: Option<LoxClass>,
        methods: HashMap<String, LoxFunction>,
    ) -> Self {
        Self {
            name,
            methods,
            super_class: super_class.map(Box::new),
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        if let Some(method) = self.methods.get(name) {
            Some(method)
        } else if let Some(superclass) = &self.super_class {
            superclass.find_method(name)
        } else {
            None
        }
    }
}

impl Callable for LoxClass {
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, LoxError> {
        let instance = LoxInstance::new(self.clone());
        if let Some(initializer) = self.find_method("init") {
            initializer.bind(instance.clone()).call(interpreter, args)?;
        }
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
    inner: Rc<RefCell<InnerLoxInstance>>,
}


#[derive(Debug, PartialEq, Clone)]
struct InnerLoxInstance {
    class: LoxClass,
    fields: HashMap<String, Object>,
}


impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            inner: Rc::new(RefCell::new(
                InnerLoxInstance {
                    class,
                    fields: HashMap::new(),
                }
            ))
        }
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxError> {
        if let Some(o) = self.inner.borrow().fields.get(&name.lexeme) {
            Ok(o.clone())
        } else if let Some(o) = self.inner.borrow().class.find_method(&name.lexeme) {
            Ok(Object::Callable(Rc::new(Box::new(o.bind(self.clone())))))
        } else {
            Err(LoxError::Error(format!(
                "Undefined property '{}'.",
                name.lexeme
            )))
        }
    }

    pub fn set(&mut self, name: &Token, value: Object) -> Result<(), LoxError> {
        self.inner.borrow_mut().fields.insert(name.lexeme.clone(), value);
        Ok(())
    }
}

impl ToString for LoxInstance {
    fn to_string(&self) -> String {
        self.inner.borrow().class.name()
    }
}
