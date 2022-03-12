use crate::{
    expressions::{AssigmentExpr, BinaryExpr, UnaryExpr},
    object::Object,
    tokens::TokenType,
    Expr, LoxError, Statement, Token,
};
use std::collections::{HashMap, VecDeque};

struct Environment(Vec<HashMap<String, Object>>);
impl Environment {
    fn new() -> Self {
        Self(vec![HashMap::new()])
    }
    fn new_scope(&mut self) {
        self.0.push(HashMap::new());
    }
    fn end_scope(&mut self) {
        self.0.pop().unwrap();
    }
    fn define(&mut self, name: String, value: Object) {
        self.get_current_scope().insert(name, value);
    }

    fn assign(&mut self, name: String, value: Object) -> Result<(), LoxError> {
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

    fn get_current_scope(&mut self) -> &mut HashMap<String, Object> {
        let i = self.0.len() - 1;
        self.0.get_mut(i).unwrap()
    }

    fn get(&self, name: String) -> Result<Object, LoxError> {
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

    fn contains(&self, name: &str) -> bool {
        for env in self.0.iter().rev() {
            if env.contains_key(name) {
                return true;
            }
        }
        false
    }
}

pub struct Interpreter {
    statements: Vec<Statement>,
    current: usize,
    env: Environment,
    print_log: Vec<Object>,
}

impl Interpreter {
    fn new(statements: Vec<Statement>) -> Self {
        Interpreter {
            statements,
            current: 0,
            env: Environment::new(),
            print_log: Vec::new(),
        }
    }

    pub fn interpret(statements: Vec<Statement>) -> Result<Vec<Object>, LoxError> {
        let mut interpreter = Self::new(statements);
        while !interpreter.is_at_end() {
            interpreter.eval_stmt(interpreter.peek())?;
            interpreter.advance()?;
        }

        Ok(interpreter.print_log)
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
        }
    }

    fn while_stm(&mut self, cond: Expr, stm: Statement) -> Result<(), LoxError> {
        while Self::is_truthy(self.eval_expr(cond.clone())?) {
            self.eval_stmt(stm.clone())?;
        }
        Ok(())
    }

    fn block(&mut self, mut stms: VecDeque<Statement>) -> Result<(), LoxError> {
        self.env.new_scope();

        while !stms.is_empty() {
            self.eval_stmt(stms.pop_front().unwrap())?;
        }
        self.env.end_scope();

        Ok(())
    }

    fn if_stm(
        &mut self,
        cond: Expr,
        then_stm: Statement,
        else_stm: Option<Box<Statement>>,
    ) -> Result<(), LoxError> {
        if Self::is_truthy(self.eval_expr(cond)?) {
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
        self.print_log.push(val);
        Ok(())
    }

    fn eval_expr(&mut self, expr: Expr) -> Result<Object, LoxError> {
        match expr {
            Expr::Literal(e) => Ok(e.value),
            Expr::Grouping(e) => self.eval_expr(*e.expression),
            Expr::Unary(e) => self.unary_expr(e),
            Expr::Binary(e) => self.binary_expr(e),
            Expr::Variable(t) => self.env.get(t.lexeme),
            Expr::Assignment(e) => self.assign_expr(e),
            Expr::Logical(e1, op, e2) => self.logical_expr(*e1, *e2, op),
        }
    }

    fn logical_expr(&mut self, e1: Expr, e2: Expr, op: Token) -> Result<Object, LoxError> {
        let left = self.eval_expr(e1)?;
        match op.token_type {
            TokenType::AND => {
                if Self::is_truthy(left.clone()) {
                    Ok(self.eval_expr(e2)?)
                } else {
                    Ok(left)
                }
            }
            TokenType::OR => {
                if Self::is_truthy(left.clone()) {
                    Ok(left)
                } else {
                    Ok(self.eval_expr(e2)?)
                }
            }
            _ => unreachable!(),
        }
    }

    fn assign_expr(&mut self, e: AssigmentExpr) -> Result<Object, LoxError> {
        if self.env.contains(&e.name.lexeme) {
            let val = self.eval_expr(*e.value)?;
            self.env.assign(e.name.lexeme, val)?;
            Ok(Object::Nil)
        } else {
            Err(LoxError::Error(format!(
                "Udefined variable '{}'.",
                e.name.lexeme
            )))
        }
    }

    fn unary_expr(&mut self, e: UnaryExpr) -> Result<Object, LoxError> {
        let right = self.eval_expr(*e.right)?;

        if e.operator.token_type == TokenType::MINUS {
            if let Object::Number(n) = right {
                return Ok(Object::Number(n * -1.0));
            }
        }

        if e.operator.token_type == TokenType::BANG {
            return Ok(Object::Boolean(!Self::is_truthy(right)));
        }

        unreachable!();
    }

    fn binary_expr(&mut self, e: BinaryExpr) -> Result<Object, LoxError> {
        let left = self.eval_expr(*e.left)?;
        let right = self.eval_expr(*e.right)?;

        let get_num = |x| Self::get_v_num(x);
        let get_str = |x| Self::get_v_string(x);

        let is_num = |x: &Object| get_num(x.clone()).is_ok();

        let to_num = |n| Object::Number(n);
        let to_str = |s| Object::String(s);
        let to_bool = |b| Object::Boolean(b);

        let obj = {
            match e.operator.token_type {
                TokenType::MINUS => to_num(get_num(left)? - get_num(right)?),
                TokenType::SLASH => to_num(get_num(left)? / get_num(right)?),
                TokenType::STAR => to_num(get_num(left)? * get_num(right)?),
                TokenType::PLUS => {
                    if is_num(&left) && is_num(&right) {
                        to_num(get_num(left)? + get_num(right)?)
                    } else {
                        to_str(get_str(left)? + &get_str(right)?)
                    }
                }
                TokenType::GREATER => to_bool(get_num(left)? > get_num(right)?),
                TokenType::GREATER_EQUAL => to_bool(get_num(left)? >= get_num(right)?),
                TokenType::LESS => to_bool(get_num(left)? < get_num(right)?),
                TokenType::LESS_EQUAL => to_bool(get_num(left)? <= get_num(right)?),
                TokenType::BANG_EQUAL => to_bool(!Self::is_equal(left, right)),
                TokenType::EQUAL_EQUAL => to_bool(Self::is_equal(left, right)),
                _ => unreachable!(),
            }
        };
        Ok(obj)
    }

    fn get_v_num(obj: Object) -> Result<f64, LoxError> {
        if let Object::Number(n) = obj {
            Ok(n)
        } else {
            Err(LoxError::Error(format!("'{:?}' must be a number.", obj)))
        }
    }

    fn get_v_string(obj: Object) -> Result<String, LoxError> {
        if let Object::String(s) = obj {
            Ok(s)
        } else {
            Err(LoxError::Error(format!("'{:?}' must be a string.", obj)))
        }
    }

    fn is_truthy(obj: Object) -> bool {
        if obj == Object::Nil {
            return false;
        }
        if let Object::Boolean(b) = obj {
            return b;
        };

        true
    }

    fn is_equal(a: Object, b: Object) -> bool {
        match (a, b) {
            (Object::Nil, Object::Nil) => true,
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::Number(a), Object::Number(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            _ => false,
        }
    }

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
