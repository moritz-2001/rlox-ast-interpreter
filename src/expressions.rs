use crate::{tokens::{Token, Object}};


#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LiteralExpr {
    pub value: Object,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>
}







