use crate::class::LoxInstance;
use crate::expressions::Var;
use crate::object::Object;
use crate::lox_error::LoxError;
use crate::interpreter::Interpreter;
use crate::environment::Environment;
use crate::tokens::{Token, TokenType};
use crate::statements::Statement;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::rc::Rc;
use std::cell::RefCell;



pub trait Callable: fmt::Debug  {
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, LoxError>;
    fn arity(&self) -> usize;
    fn name(&self) -> String;
}


impl fmt::Display for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl PartialEq for dyn Callable {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.arity() == other.arity()
    }
}


#[derive(Debug)]
pub struct Clock;


impl Callable for Clock {
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, LoxError> {
        let start = SystemTime::now();
        Ok(Object::Number(start.duration_since(UNIX_EPOCH).unwrap().as_secs_f64()))
    }
    fn arity(&self) -> usize {
        0
    }
    fn name(&self) -> String {
        "clock".to_string()
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct LoxFunction {
    name: Token,
    paras: Vec<Token>,
    body: Statement,
    env: Rc<RefCell<Environment>>,
    is_init: bool,
}

impl LoxFunction {
    pub fn new(name: Token, paras: Vec<Token>, body: Statement, env: Rc<RefCell<Environment>>, is_init: bool) -> Self {
        Self {
            name,
            paras,
            body,
            env,
            is_init,
        }
    }

    pub fn bind(&self, instance: &LoxInstance) -> LoxFunction {
        let mut env = Environment::new_with_enclosing(Rc::clone(&self.env));
        env.define("this".to_string(), Object::Instance(instance.clone()));

        LoxFunction::new(self.name.clone(), self.paras.clone(), self.body.clone(), Rc::new(RefCell::new(env)), self.is_init)
    }
}

impl Callable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, LoxError> {
        let mut env = Environment::new_with_enclosing(Rc::clone(&self.env));
        for (param, arg) in self.paras.iter().zip(args) {
            env.define(param.lexeme.clone(), arg.clone());
        }
        if let Err(LoxError::Return(o)) = interpreter.exec_block(&vec![self.body.clone()], env) {
            Ok(o)
        } else {
            if self.is_init {
                self.env.borrow_mut().get_at(Var { 
                    identifier: Token::new(TokenType::IDENTIFIER, 
                        "this".to_string(), 
                        None, 
                        0), hops: 0 })
            } else {
                Ok(Object::Nil)
            }
        }
    }
    fn arity(&self) -> usize {
        self.paras.len()
    }
    fn name(&self) -> String {
        self.name.lexeme.clone()
    }
}   