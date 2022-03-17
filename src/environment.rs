use crate::expressions::Var;
use crate::{object::Object, LoxError};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;


#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}


impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None, 
        }
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }


    pub fn get(&self, name: String) -> Result<Object, LoxError> {
        if let Some(x) = self.values.get(&name) {
            Ok(x.clone())
        } else if let Some(env) = &self.enclosing {
            env.borrow_mut().get(name)
        } else {
            Err(LoxError::UndefinedVariable(format!(
                "Undefined variable '{}'.",
                name
            )))
        }
    }

    pub fn assign(&mut self, name: String, value: Object) -> Result<(), LoxError> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else if let Some(ref env) = self.enclosing {
            env.borrow_mut().assign(name, value)
        } else {
            Err(LoxError::UndefinedVariable(format!(
                "Undefined variable '{}'.",
                name
            )))
        }
    }

    pub fn ancestor(env: Rc<RefCell<Environment>>, distance: usize) -> Result<Rc<RefCell<Environment>>, LoxError> {
        if distance == 0 {
            Ok(env.clone())
        } else {
            Self::ancestor(env.borrow_mut().enclosing.as_ref().unwrap().clone(), distance-1)
        }
    }

    pub fn assign_at(&mut self, var: Var, value: Object) -> Result<(), LoxError> {
        if var.hops == 0 {
            self.assign(var.name().to_string(), value)
        } else {
            Self::ancestor(self.enclosing.as_ref().unwrap().clone(), var.hops-1)?.borrow_mut().values.insert(var.name().to_string(), value);
            Ok(())
        }
    }

    pub fn get_at(&mut self, var: Var) -> Result<Object, LoxError> {
        if var.hops == 0 {
            self.get(var.name().to_string())
        } else {
            Ok(Self::ancestor(self.enclosing.as_ref().unwrap().clone(), var.hops-1)?.borrow_mut().get(var.name().to_string()).unwrap().clone())
        }
    }
}