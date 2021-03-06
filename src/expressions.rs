use crate::object::Object;
use crate::tokens::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Object),
    Unary(Token, Box<Expr>),
    Variable(Var),
    Assignment(Var, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Get(Box<Expr>, Token),
    Set(Box<Expr>, Token, Box<Expr>),
    This(Var),
    Super(Var, Token),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub identifier: Token,
    pub hops: usize,
}

impl Var {
    pub fn new(identifier: Token) -> Self {
        Self {
            identifier,
            hops: 0,
        }
    }
    pub fn name(&self) -> &str {
        &self.identifier.lexeme
    }

    pub fn new_wo_token(name: &str, hops: usize) -> Self {
        Self {
            identifier: Token::new(TokenType::IDENTIFIER, name, None, 0),
            hops,
        }
    }
}
