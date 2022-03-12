use std::collections::VecDeque;

use crate::tokens::Token;
use crate::Expr;

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl(Token, Expr),
    Expr(Expr),
    Print(Expr),
    Block(VecDeque<Statement>),
    If(Expr, Box<Statement>, Option<Box<Statement>>),
    While(Expr, Box<Statement>),
    //For(Option<Box<Statement>>, Option<Box<Statement>>, Box<Statement>),
}
