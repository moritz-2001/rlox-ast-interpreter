use crate::environment::Environment;
use crate::expressions::Var;
use crate::{
    callable::{Callable, Clock, LoxFunction},
    class::LoxClass,
    object::Object,
    resolver::Resolver,
    tokens::TokenType,
    Expr, LoxError, Statement, Token,
};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::mem;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Interpreter {
    statements: Vec<Statement>,
    current: usize,
    env: Environment,
}

impl Interpreter {
    fn new(statements: Vec<Statement>) -> Self {
        let mut env = Environment::new();
        let clock = Clock {};
        env.define(clock.name(), Object::Callable(Rc::new(Box::new(clock))));
        Interpreter {
            statements,
            current: 0,
            env,
        }
    }

    pub fn interpret(statements: Vec<Statement>) -> Result<Object, LoxError> {
        let mut interpreter = Self::new(statements);
        Resolver::run(&mut interpreter.statements);
        interpreter.run()
    }

    pub fn run(&mut self) -> Result<Object, LoxError> {
        while !self.is_at_end() {
            let ret = self.eval_stmt(self.peek());
            if let Err(LoxError::Return(x)) = ret {
                return Ok(x);
            }
            ret?;
            self.advance()?;
        }
        Ok(Object::Nil)
    }

    fn eval_stmt(&mut self, stmt: Statement) -> Result<(), LoxError> {
        match stmt {
            Statement::Expr(e) => {
                let _ = self.eval_expr(e);
                Ok(())
            }
            Statement::Print(e) => self.eval_print(e),
            Statement::VarDecl(t, e) => self.var_dec(t, e),
            Statement::Block(stms) => self.block(stms),
            Statement::If(cond, then_stm, else_stm) => self.if_stm(cond, *then_stm, else_stm),
            Statement::While(cond, stm) => self.while_stm(cond, *stm),
            Statement::FuncDecl(name, args, stm) => self.function_decl(name, args, *stm),
            Statement::Return(e) => self.return_stm(e),
            Statement::ClassDecl(name, methods) => self.class_decl(name, methods),
        }
    }

    fn function_decl(
        &mut self,
        name: Token,
        args: Vec<Token>,
        stm: Statement,
    ) -> Result<(), LoxError> {
        let f = LoxFunction::new(name.clone(), args, stm, self.env.clone(), false);

        self.env
            .define(name.lexeme.clone(), Object::Callable(Rc::new(Box::new(f))));

        Ok(())
    }

    fn class_decl(&mut self, name: Token, methods: Vec<Statement>) -> Result<(), LoxError> {
        self.env.define(name.lexeme.clone(), Object::Nil);

        let mut method_map = HashMap::with_capacity(methods.len());

        for method in methods {
            if let Statement::FuncDecl(name, args, body) = method {
                let f = LoxFunction::new(
                    name.clone(),
                    args,
                    *body,
                    self.env.clone(),
                    name.to_string() == "init",
                );
                method_map.insert(name.lexeme, f);
            } else {
                panic!("error class_decl");
            }
        }

        let class = LoxClass::new(name.lexeme.clone(), method_map);
        self.env.assign(
            name.lexeme.clone(),
            Object::Callable(Rc::new(Box::new(class))),
        )?;
        Ok(())
    }

    fn while_stm(&mut self, cond: Expr, stm: Statement) -> Result<(), LoxError> {
        while (self.eval_expr(cond.clone())?).is_truthy() {
            self.eval_stmt(stm.clone())?;
        }
        Ok(())
    }

    fn return_stm(&mut self, e: Expr) -> Result<(), LoxError> {
        let val = self.eval_expr(e)?;
        Err(LoxError::Return(val))
    }

    fn block(&mut self, stms: VecDeque<Statement>) -> Result<(), LoxError> {
        let new_env = Environment::new_with_enclosing(&self.env);
        let vec_stms: Vec<Statement> = stms.into_iter().collect();
        self.exec_block(&vec_stms, new_env)
    }

    pub fn exec_block(&mut self, stms: &[Statement], new_env: Environment) -> Result<(), LoxError> {
        let previous_env = mem::replace(&mut self.env, new_env.clone());
        let res = stms.iter().try_for_each(|stm| self.eval_stmt(stm.clone()));
        self.env = previous_env;

        res
    }

    fn if_stm(
        &mut self,
        cond: Expr,
        then_stm: Statement,
        else_stm: Option<Box<Statement>>,
    ) -> Result<(), LoxError> {
        if (self.eval_expr(cond)?).is_truthy() {
            self.eval_stmt(then_stm)?;
        } else {
            if let Some(else_stm) = else_stm {
                self.eval_stmt(*else_stm)?;
            }
        }

        Ok(())
    }

    fn var_dec(&mut self, t: Token, e: Expr) -> Result<(), LoxError> {
        let obj = self.eval_expr(e)?;
        self.env.define(t.lexeme, obj);
        Ok(())
    }

    fn eval_print(&mut self, e: Expr) -> Result<(), LoxError> {
        let val = self.eval_expr(e)?;
        println!("{}", val);
        Ok(())
    }

    fn eval_expr(&mut self, expr: Expr) -> Result<Object, LoxError> {
        match expr {
            Expr::Literal(o) => Ok(o),
            Expr::Grouping(e) => self.eval_expr(*e),
            Expr::Unary(t, e) => self.unary_expr(*e, t),
            Expr::Binary(e1, t, e2) => self.binary_expr(*e1, *e2, t),
            Expr::Variable(var) => Ok(self.env.get_at(&var)?.clone()),
            Expr::Assignment(var, e) => self.assign_expr(*e, var),
            Expr::Logical(e1, op, e2) => self.logical_expr(*e1, *e2, op),
            Expr::Call(callee, args) => self.call_expr(*callee, args),
            Expr::Get(e, name) => self.get_expr(*e, name),
            Expr::Set(e1, name, e2) => self.set_expr(*e1, *e2, name),
            Expr::This(var) => Ok(self.env.get_at(&var)?.clone()),
        }
    }

    fn get_expr(&mut self, e: Expr, name: Token) -> Result<Object, LoxError> {
        let object = self.eval_expr(e)?;
        if let Object::Instance(instance) = &object {
            return instance.get(&name);
        }
        Err(LoxError::Error(format!("{} is not a instance", object)))
    }

    fn set_expr(&mut self, e1: Expr, e2: Expr, name: Token) -> Result<Object, LoxError> {
        let value = self.eval_expr(e1)?;
        let (mut instance, var) = {
            if let Expr::Variable(var) = e2 {
                (self.env.get_at(&var)?, var)
            } else {
                return Err(LoxError::Error(format!("err")));
            }
        };

        if let Object::Instance(lox_instance) = &mut instance {
            lox_instance.set(&name, value.clone())?;
            self.env.assign(var.name().to_string(), instance.clone())?;
            return Ok(value);
        }
        Err(LoxError::Error(format!("{} is not a instance", value)))
    }

    fn logical_expr(&mut self, e1: Expr, e2: Expr, op: Token) -> Result<Object, LoxError> {
        let left = self.eval_expr(e1)?;
        match op.token_type {
            TokenType::AND => {
                if left.clone().is_truthy() {
                    Ok(self.eval_expr(e2)?)
                } else {
                    Ok(left)
                }
            }
            TokenType::OR => {
                if left.clone().is_truthy() {
                    Ok(left)
                } else {
                    Ok(self.eval_expr(e2)?)
                }
            }
            _ => unreachable!(),
        }
    }

    fn assign_expr(&mut self, e: Expr, var: Var) -> Result<Object, LoxError> {
        let val = self.eval_expr(e)?;

        self.env.assign_at(&var, val)?;
        Ok(Object::Nil)
    }

    fn unary_expr(&mut self, e: Expr, operator: Token) -> Result<Object, LoxError> {
        let right = self.eval_expr(e)?;

        if operator.token_type == TokenType::MINUS {
            if let Object::Number(n) = right {
                return Ok(Object::Number(n * -1.0));
            }
        }

        if operator.token_type == TokenType::BANG {
            return Ok(Object::Boolean(!right.is_truthy()));
        }

        unreachable!();
    }

    fn binary_expr(
        &mut self,
        left: Expr,
        right: Expr,
        operator: Token,
    ) -> Result<Object, LoxError> {
        let left = self.eval_expr(left)?;
        let right = self.eval_expr(right)?;

        let is_num = |x: &Object| x.clone().get_v_num().is_ok();

        let to_num = |n| Object::Number(n);
        let to_str = |s| Object::String(s);
        let to_bool = |b| Object::Boolean(b);

        let obj = {
            match operator.token_type {
                TokenType::MINUS => to_num(left.get_v_num()? - right.get_v_num()?),
                TokenType::SLASH => to_num(left.get_v_num()? / right.get_v_num()?),
                TokenType::STAR => to_num(left.get_v_num()? * right.get_v_num()?),
                TokenType::PLUS => {
                    if is_num(&left) && is_num(&right) {
                        to_num(left.get_v_num()? + right.get_v_num()?)
                    } else {
                        to_str(left.get_v_string()? + &right.get_v_string()?)
                    }
                }
                TokenType::GREATER => to_bool(left.get_v_num()? > right.get_v_num()?),
                TokenType::GREATER_EQUAL => to_bool(left.get_v_num()? >= right.get_v_num()?),
                TokenType::LESS => to_bool(left.get_v_num()? < right.get_v_num()?),
                TokenType::LESS_EQUAL => to_bool(left.get_v_num()? <= right.get_v_num()?),
                TokenType::BANG_EQUAL => to_bool(!Object::is_equal(left, right)),
                TokenType::EQUAL_EQUAL => to_bool(Object::is_equal(left, right)),
                _ => unreachable!(),
            }
        };
        Ok(obj)
    }

    fn call_expr(&mut self, callee: Expr, args: Vec<Expr>) -> Result<Object, LoxError> {
        let callee = self.eval_expr(callee)?;

        let mut arguments = Vec::with_capacity(args.len());
        for e in args {
            arguments.push(self.eval_expr(e)?);
        }

        if let Object::Callable(function) = callee {
            if function.arity() != arguments.len() {
                return Err(LoxError::Error(format!(
                    "Expected {} arguments but got {}.",
                    function.arity(),
                    arguments.len()
                )));
            }
            return function.call(self, &arguments);
        }

        unreachable!();
    }
}

impl Interpreter {
    fn is_at_end(&self) -> bool {
        self.current >= self.statements.len()
    }

    fn advance(&mut self) -> Result<Statement, LoxError> {
        if let Some(statement) = self.statements.get(self.current) {
            self.current += 1;
            Ok(statement.clone())
        } else {
            Err(LoxError::Error(format!(
                "Interpreter error: advance not possible"
            )))
        }
    }

    fn peek(&self) -> Statement {
        self.statements.get(self.current).unwrap().clone()
    }
}
