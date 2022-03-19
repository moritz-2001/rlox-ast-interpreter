use crate::class;
use crate::statements::Statement;
use crate::tokens::Token;
use crate::expressions::{Expr, Var};

use std::collections::HashMap;


#[derive(Debug, Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassType {
    None,
    Class
}




#[derive(Debug)]
pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    current_class: ClassType,
    current_function: FunctionType,
}


impl Resolver {
    pub fn run(stms: &mut [Statement]) {
        let mut resolver = Resolver{scopes:vec![HashMap::new()], current_class: ClassType::None, current_function: FunctionType::None};
        for stm in stms {
            resolver.resolve_stmt(stm);
        }
    }
}

impl Resolver {
    fn resolve_stmt(&mut self, stmt: &mut Statement) {
        match stmt {
            Statement::Block(stms) => {
                self.begin_scope();
                for stm in stms {
                    self.resolve_stmt(stm);
                }
                self.end_scope();
            },
            Statement::VarDecl(token, exp) => {
                self.declare(token);
                self.resolve_exp(exp);
                self.define(token);
            },
            Statement::FuncDecl(name, args, body) => {
                self.declare(name);
                self.define(name);
                self.resolve_function(name, args, body);
            },
            Statement::Expr(e) => {
                self.resolve_exp(e);
            },
            Statement::If(e, stmt1, stmt2) => {
                self.resolve_exp(e);
                self.resolve_stmt(stmt1);
                if let Some(stm) = stmt2 {
                    self.resolve_stmt(stm);
                }
            },
            Statement::Print(e) => {
                self.resolve_exp(e);
            },
            Statement::Return(e) => {
                if self.current_function == FunctionType::Initializer {
                    panic!("Cant't return a value from an initializer.");
                }
                self.resolve_exp(e);
            },
            Statement::While(e, body) => {
                self.resolve_exp(e);
                self.resolve_stmt(body);
            },
            Statement::ClassDecl(name, methods) => {
                let enclosing_class = self.current_class.clone();
                self.current_class = ClassType::Class;

                self.declare(name);
                self.define(name);

                self.begin_scope();
                self.scopes.last_mut().unwrap().insert("this".to_string(), true);

                for method in methods {
                    let declaration = self.current_function.clone();
                    self.current_function = FunctionType::Method;
                    if let Statement::FuncDecl(name, args, body) = method {
                        if name.to_string() == "init" {
                            self.current_function = FunctionType::Initializer;
                        }
                        self.resolve_function(name, args, body);
                        self.current_function = declaration; 
                    } else {
                        panic!("Resolver error ClassDecl");
                    }
                }

                self.end_scope();
                self.current_class = enclosing_class;
            }
        }
    }


    fn resolve_exp(&mut self, exp: &mut Expr) {
        match exp {
            Expr::Binary(e1, t, e2) => {
                self.resolve_exp(e1);
                self.resolve_exp(e2);
            },
            Expr::Call(e, vec_e) => {
                self.resolve_exp(e);
                for e1 in vec_e {
                    self.resolve_exp(e1);
                }
            },
            Expr::Grouping(e) => {
                self.resolve_exp(e);
            },
            Expr::Literal(o) => {},
            Expr::Logical(e1, t, e2) => {
                self.resolve_exp(e1);
                self.resolve_exp(e2);
            },
            Expr::Unary(t, e) => {
                self.resolve_exp(e);
            },
            Expr::Assignment(var, e) => {
                self.resolve_exp(e);
                self.resolve_var(var)
            },
            Expr::Variable(var) => {
                if let Some(b) = self.scopes.last().unwrap().get(var.name()) {
                    if *b == false {
                        panic!("Cant't read local variable in its own initializer. {}", var.name());
                    }
                }
                self.resolve_var(var);
            },
            Expr::Get(e, name) => {
                self.resolve_exp(e);
            },
            Expr::Set(e1, name, e2) => {
                self.resolve_exp(e1);
                self.resolve_exp(e2);
            },
            Expr::This(keyword) => {
                if self.current_class == ClassType::None {
                    panic!("Can't use 'this' outside of a class.");
                }
                self.resolve_var(keyword);
            }
        }
    }
}




impl Resolver {
    fn resolve_var(&mut self, var: &mut Var) {
        for (i, scope) in self.scopes.iter_mut().rev().enumerate() {
            if scope.contains_key(var.name()) {
                var.hops = i;
                return;
            }
        }
        var.hops = self.scopes.len();
    }



    fn resolve_function(&mut self, name: &mut Token, args: &mut [Token], body: &mut Box<Statement>) {
        self.begin_scope();
        for param in args {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmt(body);
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().unwrap();
    }


    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        self.scopes.last_mut().unwrap().insert(name.lexeme.clone(), true);
    }
}