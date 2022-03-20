use crate::expressions::Var;
use crate::tokens::Token;
use crate::Expr;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDecl(Token, Expr),
    ClassDecl(Token, Option<Var>, Vec<Statement>),
    Expr(Expr),
    Print(Expr),
    Block(VecDeque<Statement>),
    If(Expr, Box<Statement>, Option<Box<Statement>>),
    While(Expr, Box<Statement>),
    FuncDecl(Token, Vec<Token>, Box<Statement>),
    Return(Expr),
}
