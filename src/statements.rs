use std::collections::VecDeque;

use crate::Expr;
use crate::tokens::Token;

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl(Token, Expr),
    Expr(Expr),
    Print(Expr),
    Block(VecDeque<Statement>),
}


