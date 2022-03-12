use crate::{object::Object, LoxError};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment(Vec<HashMap<String, Object>>);
impl Environment {
    pub fn new() -> Self {
        Self(vec![HashMap::new(), HashMap::new()])
    }
    pub fn new_scope(&mut self) {
        self.0.push(HashMap::new());
    }
    pub fn end_scope(&mut self) {
        self.0.pop().unwrap();
    }
    pub fn define(&mut self, name: String, value: Object) {
        self.get_current_scope().insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Object) -> Result<(), LoxError> {
        for env in self.0.iter_mut().rev() {
            if env.contains_key(&name) {
                env.insert(name, value);
                return Ok(());
            }
        }
        Err(LoxError::UndefinedVariable(format!(
            "Undefined variable '{}'.",
            name
        )))
    }

    pub fn get_current_scope(&mut self) -> &mut HashMap<String, Object> {
        let i = self.0.len() - 1;
        self.0.get_mut(i).unwrap()
    }

    pub fn get(&self, name: String) -> Result<Object, LoxError> {
        for env in self.0.iter().rev() {
            if let Some(x) = env.get(&name) {
                return Ok(x.clone());
            }
        }

        Err(LoxError::UndefinedVariable(format!(
            "Undefined variable '{}'.",
            name
        )))
    }

    pub fn contains(&self, name: &str) -> bool {
        for env in self.0.iter().rev() {
            if env.contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn define_global(&mut self, name: String, value: Object) {
        self.0[0].insert(name, value);
    }
}
