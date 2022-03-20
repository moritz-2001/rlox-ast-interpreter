use crate::expressions::Var;
use crate::{object::Object, LoxError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    inner: Rc<RefCell<Env>>,
}

#[derive(Debug, PartialEq)]
struct Env {
    values: HashMap<String, Object>,
    enclosing: Option<Environment>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Env {
                values: HashMap::new(),
                enclosing: None,
            })),
        }
    }

    pub fn new_with_enclosing(enclosing: &Environment) -> Environment {
        Self {
            inner: Rc::new(RefCell::new(Env {
                values: HashMap::new(),
                enclosing: Some(enclosing.clone()),
            })),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.inner.borrow_mut().values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Object, LoxError> {
        if let Some(v) = self.inner.borrow().values.get(name) {
            Ok(v.clone())
        } else if let Some(env) = self.inner.borrow().enclosing.as_ref() {
            env.get(name)
        } else {
            Err(LoxError::UndefinedVariable(format!(
                "Undefined variable '{}'.",
                name
            )))
        }
    }

    pub fn assign(&mut self, name: String, value: Object) -> Result<(), LoxError> {
        if self.inner.borrow().values.contains_key(&name) {
            self.inner.borrow_mut().values.insert(name, value);
            Ok(())
        } else if let Some(env) = self.inner.borrow_mut().enclosing.as_mut() {
            env.assign(name, value)
        } else {
            Err(LoxError::UndefinedVariable(format!(
                "Undefined variable '{}'.",
                name
            )))
        }
    }

    fn ancestor(&self, distance: usize) -> Result<Environment, LoxError> {
        if distance == 0 {
            Ok(self.clone())
        } else {
            self.inner
                .borrow()
                .enclosing
                .as_ref()
                .unwrap()
                .ancestor(distance - 1)
        }
    }

    pub fn assign_at(&mut self, var: &Var, value: Object) -> Result<(), LoxError> {
        let mut env = self.ancestor(var.hops)?;
        env.assign(var.name().to_string(), value)?;
        Ok(())
    }

    pub fn get_at(&self, var: &Var) -> Result<Object, LoxError> {
        let env = self.ancestor(var.hops)?;
        env.get(var.name())
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_define() {
        let mut env = Environment::new();
        env.define("x".to_string(), Object::Number(10.0));
        assert_eq!(env.get("x").unwrap(), Object::Number(10.0));
    }

    #[test]
    fn env_assign() {
        let mut env = Environment::new();
        env.define("x".to_string(), Object::Number(10.0));
        env.assign("x".to_string(), Object::Number(102.0)).unwrap();
        assert_eq!(env.get("x").unwrap(), Object::Number(102.0));
    }

    #[test]
    #[should_panic]
    fn env_assign_without_define() {
        let mut env = Environment::new();
        env.assign("x".to_string(), Object::Number(102.0)).unwrap();
        assert_eq!(env.get("x").unwrap(), Object::Number(102.0));
    }

    #[test]
    fn env_get_at() {
        let mut env = Environment::new();
        env.define("x".to_string(), Object::Number(10.0));
        let mut env = Environment::new_with_enclosing(&env);

        assert_eq!(
            env.get_at(&Var::new_wo_token("x", 1)).unwrap(),
            Object::Number(10.0)
        );
    }

    #[test]
    fn env_assign_at() {
        let mut env = Environment::new();
        env.define("x".to_string(), Object::Number(10.0));
        let mut env = Environment::new_with_enclosing(&env);

        env.assign_at(&Var::new_wo_token("x", 1), Object::Number(100.0))
            .unwrap();

        assert_eq!(
            env.get_at(&Var::new_wo_token("x", 1)).unwrap(),
            Object::Number(100.0)
        );
    }
}
