use crate::statements::Statement;
use crate::tokens::Token;
use crate::expressions::Expr;

use std::collections::HashMap;


pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
}


impl Resolver {
    fn stm(&mut self, stmt: Statement) {
        match stmt.clone() {
            Statement::Block(stms) => {
                for stm in stms {
                    self.resolve_stmt(stm);
                }
                self.end_scope();
            },
            Statement::VarDecl(token, exp) => {
                self.declare(token.clone());

                // exp != null ?
                self.resolve_exp(exp);
                
                self.define(token);
            },
            Statement::FuncDecl(name, args, body) => {
                self.declare(name.clone());
                self.define(name);
                self.resolve_function(stmt);
            },
            Statement::Expr(e) => {
                self.resolve_exp(e);
            },
            Statement::If(e, stmt1, stmt2) => {
                self.resolve_exp(e);
                self.resolve_stmt(*stmt1);
                if let Some(stm) = stmt2 {
                    self.resolve_stmt(*stm);
                }
            },
            Statement::Print(e) => {
                self.resolve_exp(e);
            },
            Statement::Return(e) => {
                self.resolve_exp(e);
            },
            Statement::While(e, body) => {
                self.resolve_exp(e);
                self.resolve_stmt(*body);
            }     
        }
    }


    fn exp(&mut self, exp: Expr) {
        match exp.clone() {
            Expr::Binary(e1, t, e2) => {
                self.resolve_exp(*e1);
                self.resolve_exp(*e2);
            },
            Expr::Call(e, vec_e) => {
                self.resolve_exp(*e);
                for e1 in vec_e {
                    self.resolve_exp(e1);
                }
            },
            Expr::Grouping(e) => {
                self.resolve_exp(*e);
            },
            Expr::Literal(o) => {},
            Expr::Logical(e1, t, e2) => {
                self.resolve_exp(*e1);
                self.resolve_exp(*e2);
            },
            Expr::Unary(t, e) => {
                self.resolve_exp(*e);
            },
            Expr::Assignment(t, e) => {
                self.resolve_exp(*e);
                self.resolve_local(*e, t)
            },
            Expr::Variable(t) => {
                if !self.scopes.is_empty() && *self.scopes.last().unwrap().get(&name.lexeme).unwrap() == false {
                    panic!("Cant't read local variable in its own initializer. {}", name.lexeme);
                }
        
                self.resolve_local(exp, t);
            },
        }
    }
}




impl Resolver {
    fn resolve_block(&mut self, stmts: Vec<Statement>) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_local(&mut self, expr: Expr, name: Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                // interpreter.resolve(expr, scopes.size() - 1 - 1)
                return;
            }
        }
    }

    fn resolve_stmt(&mut self, stmt: Statement) {

    }

    fn resolve_exp(&mut self, exp: Expr) {

    }

    fn resolve_function(&mut self, stmt: Statement) {
        self.begin_scope();
        let Statement::FuncDecl(name, args, body) = stmt;
        for param in args {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmt(*body);
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().unwrap();
    }

    fn declare(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last().unwrap();
        scope.insert(name.lexeme, false);
    }

    fn define(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }
        self.scopes.last().unwrap().insert(name.lexeme, true);
    }
}