use std::collections::VecDeque;

use crate::resolver::ClassType;
use crate::tokens::Token;
use crate::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDecl(Token, Expr),
    ClassDecl(Token, Vec<Statement>),
    Expr(Expr),
    Print(Expr),
    Block(VecDeque<Statement>),
    If(Expr, Box<Statement>, Option<Box<Statement>>),
    While(Expr, Box<Statement>),
    FuncDecl(Token, Vec<Token>, Box<Statement>),
    Return(Expr),
}
