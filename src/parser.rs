use crate::Expr;
use crate::expressions::{BinaryExpr, UnaryExpr, LiteralExpr, GroupingExpr};
use crate::tokens::{Token, TokenType, Object};
use crate::lox_error::LoxError;


pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Expr, LoxError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, LoxError> {
        return self.equality();
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.comparison()?;

        while self.verify(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]){
            let op = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(BinaryExpr{
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.term()?;

        while self.verify(vec![
            TokenType::GREATER, TokenType::GREATER_EQUAL,
            TokenType::LESS, TokenType::LESS_EQUAL,
        ]) 
        {
            let op = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right)
            })
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.factor()?;

        while self.verify(vec![
            TokenType::MINUS, TokenType::PLUS,
        ])
        {
            let op = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr{
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            })
        }
        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.unary()?;

        while self.verify(vec![
            TokenType::SLASH, TokenType::STAR,
        ])
        {
            let op = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(BinaryExpr{
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.verify(vec![TokenType::BANG, TokenType::MINUS]) {
            let op = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(UnaryExpr{
                operator: op,
                right: Box::new(right),
            }));
        }

        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        if self.is_match(TokenType::FALSE) {return Ok(Expr::Literal(LiteralExpr{value: Object::False}))};
        if self.is_match(TokenType::TRUE) {return Ok(Expr::Literal(LiteralExpr{value: Object::True}))};
        if self.is_match(TokenType::NIL) {return Ok(Expr::Literal(LiteralExpr{value: Object::Nil}))};

        if self.verify(vec![TokenType::NUMBER, TokenType::STRING]) {
            return Ok(Expr::Literal(LiteralExpr{value: self.previous().literal.unwrap()}));
        }

        if self.is_match(TokenType::LEFT_PAREN) {
            let expr = self.expression()?;
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Ok(Expr::Grouping(GroupingExpr{expression: Box::new(expr)}));
        }

        return Err(LoxError::ParsingError(format!("Unkown error")));
    }

    fn get_error(&self, token: Token, msg: &str) -> LoxError {
        return LoxError::ParsingError(
            format!("{} at '{}' {}", token.line, token.lexeme, msg)
        );
    }

    fn consume(&mut self, token: TokenType, msg: &str) -> Token {
        if self.is_match(token) {return self.previous()}
        else {todo!()};
    }

    fn is_match(&mut self, token: TokenType) -> bool {
        if let Some(tok) = self.tokens.get(self.current) {
            if tok.token_type == token {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn verify(&mut self, tokens: Vec<TokenType>) -> bool {
        if let Some(tok) = self.tokens.get(self.current) {
            if tokens.iter().any(|x| *x == tok.token_type) {
                self.advance();
                return true;
            }
        }

        return false;
    }

    fn advance(&mut self) -> Option<Token> {
        if let Some(tok) = self.tokens.get(self.current) {
            self.current += 1;
            Some(tok.clone())
        } else {
            None
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.current).map(|x| x.to_owned())
    }


    fn previous(&mut self) -> Token {
        return self.tokens.get(self.current - 1).unwrap().clone();
    }

}