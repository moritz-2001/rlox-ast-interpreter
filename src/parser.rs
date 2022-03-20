use std::collections::VecDeque;

use crate::expressions::{Expr, Var};
use crate::lox_error::LoxError;
use crate::object::Object;
use crate::statements::Statement;
use crate::tokens::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, LoxError> {
        self.statements()
    }

    fn statements(&mut self) -> Result<Vec<Statement>, LoxError> {
        let mut statements: Vec<Statement> = Vec::new();

        while !self.is_at_end() && !self.is_match(TokenType::EOF) {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Statement, LoxError> {
        let try_ = {
            if self.is_match(TokenType::VAR) {
                self.var_declaration()
            } else if self.is_match(TokenType::FUN) {
                self.function("function".to_string())
            } else {
                self.statement()
            }
        };

        match try_ {
            Ok(stmt) => Ok(stmt),
            _ => {
                println!("Sync: {:?}, {:?}", try_, self.tokens[self.current]);
                self.synchronize()?;
                self.declaration()
            }
        }
    }

    fn function(&mut self, kind: String) -> Result<Statement, LoxError> {
        let name = self.consume(TokenType::IDENTIFIER, &format!("Expect {} name.", kind));

        self.consume(
            TokenType::LEFT_PAREN,
            &format!("Expect '(' after {} name.", kind),
        );
        let mut parameters = Vec::new();
        if !self.check(TokenType::RIGHT_PAREN) {
            loop {
                if parameters.len() >= 255 {
                    return Err(LoxError::Error(
                        "Can't have more than 255 parameters".to_string(),
                    ));
                }
                parameters.push(self.consume(TokenType::IDENTIFIER, "Expect parameter name."));

                if !self.is_match(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after parameters");

        self.consume(
            TokenType::LEFT_BRACE,
            &format!("Expect '{{' before {} body.", kind),
        );
        let body = self.block_statement()?;

        Ok(Statement::FuncDecl(name, parameters, Box::new(body)))
    }

    fn var_declaration(&mut self) -> Result<Statement, LoxError> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect variable name.");

        let mut init = Expr::Literal(Object::Nil);
        if self.is_match(TokenType::EQUAL) {
            init = self.expression()?;
        }

        self.consume(
            TokenType::SEMICOLON,
            "Expecct ';' after variable declaration.",
        );

        Ok(Statement::VarDecl(name, init))
    }

    fn statement(&mut self) -> Result<Statement, LoxError> {
        if self.is_match(TokenType::PRINT) {
            return self.print_statement();
        };
        if self.is_match(TokenType::LEFT_BRACE) {
            return self.block_statement();
        };
        if self.is_match(TokenType::FOR) {
            return self.for_statement();
        };
        if self.is_match(TokenType::IF) {
            return self.if_statement();
        };
        if self.is_match(TokenType::WHILE) {
            return self.while_statement();
        };
        if self.is_match(TokenType::RETURN) {
            return self.return_statement();
        }
        if self.is_match(TokenType::CLASS) {
            return self.class_statement();
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Statement, LoxError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'for'.");

        let initializer = {
            if self.is_match(TokenType::SEMICOLON) {
                None
            } else if self.is_match(TokenType::VAR) {
                Some(self.var_declaration()?)
            } else {
                Some(self.expression_statement()?)
            }
        };

        let condition = {
            if !self.check(TokenType::SEMICOLON) {
                Some(self.expression()?)
            } else {
                None
            }
        };
        self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.");

        let increment = {
            if !self.check(TokenType::RIGHT_PAREN) {
                Some(self.expression()?)
            } else {
                None
            }
        };
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after for clauses.");

        let body = {
            let stm = self.statement()?;
            if let Some(expr) = increment {
                let mut deque = VecDeque::with_capacity(2);
                deque.push_back(Statement::Expr(expr));
                deque.push_back(stm);
                Statement::Block(deque)
            } else {
                stm
            }
        };

        let while_stm = {
            if let Some(cond) = condition {
                Statement::While(cond, Box::new(body))
            } else {
                Statement::While(Expr::Literal(Object::Boolean(true)), Box::new(body))
            }
        };

        if let Some(init) = initializer {
            let mut deque = VecDeque::with_capacity(2);
            deque.push_back(init);
            deque.push_back(while_stm);
            Ok(Statement::Block(deque))
        } else {
            Ok(while_stm)
        }
    }

    fn class_statement(&mut self) -> Result<Statement, LoxError> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect class name.");

        let mut superclass = None;
        if self.is_match(TokenType::LESS) {
            self.consume(TokenType::IDENTIFIER, "Expect superclass name.");
            superclass = Some(self.previous());
        }

        self.consume(TokenType::LEFT_BRACE, "Expect '{' before class body.");

        let mut methods: Vec<Statement> = Vec::new();

        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            methods.push(self.function("method".to_string())?);
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after class body.");

        if let Some(superclass) = superclass {
            Ok(Statement::ClassDecl(
                name,
                Some(Var::new(superclass)),
                methods,
            ))
        } else {
            Ok(Statement::ClassDecl(name, None, methods))
        }
    }

    fn while_statement(&mut self) -> Result<Statement, LoxError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'while'.");
        let cond = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after condition");
        let body = self.statement()?;

        Ok(Statement::While(cond, Box::new(body)))
    }

    fn if_statement(&mut self) -> Result<Statement, LoxError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.");
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after if condition");

        let then_branch = self.statement()?;

        let mut else_branch = None;
        if self.is_match(TokenType::ELSE) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Statement::If(condition, Box::new(then_branch), else_branch))
    }

    fn block_statement(&mut self) -> Result<Statement, LoxError> {
        let mut statements: VecDeque<Statement> = VecDeque::new();

        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            statements.push_back(self.declaration()?);
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.");
        Ok(Statement::Block(statements))
    }

    fn print_statement(&mut self) -> Result<Statement, LoxError> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.");
        Ok(Statement::Print(value))
    }

    fn return_statement(&mut self) -> Result<Statement, LoxError> {
        let stm = {
            if !self.check(TokenType::SEMICOLON) {
                Statement::Return(self.expression()?)
            } else {
                Statement::Return(Expr::Literal(Object::Nil))
            }
        };

        self.consume(TokenType::SEMICOLON, "Expect ';' after return value");
        Ok(stm)
    }

    fn expression_statement(&mut self) -> Result<Statement, LoxError> {
        let expr = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression");
        Ok(Statement::Expr(expr))
    }

    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, LoxError> {
        let expr = self.or()?;

        if self.is_match(TokenType::EQUAL) {
            let _equals = self.previous();
            let val = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assignment(name, Box::new(val)));
            } else if let Expr::Get(e, n) = expr {
                return Ok(Expr::Set(Box::new(val), n, e));
            }

            return Err(LoxError::ParsingError(
                "Invalid assignment target.".to_string(),
            ));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.and()?;

        while self.is_match(TokenType::OR) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.equality()?;

        while self.is_match(TokenType::AND) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.comparison()?;

        while self.verify(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let op = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.term()?;

        while self.verify(vec![
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let op = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.factor()?;

        while self.verify(vec![TokenType::MINUS, TokenType::PLUS]) {
            let op = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.unary()?;

        while self.verify(vec![TokenType::SLASH, TokenType::STAR]) {
            let op = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.verify(vec![TokenType::BANG, TokenType::MINUS]) {
            let op = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(op, Box::new(right)));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.primary()?;

        loop {
            if self.is_match(TokenType::LEFT_PAREN) {
                expr = self.finish_call(expr)?;
            } else if self.is_match(TokenType::DOT) {
                let name = self.consume(TokenType::IDENTIFIER, "Expect property name after '.'.");
                expr = Expr::Get(Box::new(expr), name);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, LoxError> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RIGHT_PAREN) {
            loop {
                if arguments.len() >= 255 {
                    return Err(LoxError::Error(
                        "Can't have more than 
                    255 arguments."
                            .to_string(),
                    ));
                }
                arguments.push(self.expression()?);
                if !self.is_match(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after arguments");

        Ok(Expr::Call(Box::new(callee), arguments))
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        if self.is_match(TokenType::FALSE) {
            return Ok(Expr::Literal(Object::Boolean(false)));
        };
        if self.is_match(TokenType::TRUE) {
            return Ok(Expr::Literal(Object::Boolean(true)));
        };
        if self.is_match(TokenType::NIL) {
            return Ok(Expr::Literal(Object::Nil));
        };

        if self.verify(vec![TokenType::NUMBER, TokenType::STRING]) {
            return Ok(Expr::Literal(self.previous().literal.unwrap()));
        }

        if self.is_match(TokenType::LEFT_PAREN) {
            let expr = self.expression()?;
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        if self.is_match(TokenType::IDENTIFIER) {
            return Ok(Expr::Variable(Var::new(self.previous())));
        }

        if self.is_match(TokenType::THIS) {
            return Ok(Expr::This(Var::new(self.previous())));
        }

        if self.is_match(TokenType::SUPER) {
            let keyword = self.previous();
            self.consume(TokenType::DOT, "Expect '.' after 'super'.");
            let method = self.consume(TokenType::IDENTIFIER, "Expect superclass method name.");
            return Ok(Expr::Super(Var::new(keyword), method));
        }

        Err(LoxError::NotExpression)
    }

    fn synchronize(&mut self) -> Result<(), LoxError> {
        if self.advance().is_none() {
            return Err(LoxError::TokenListEmpty);
        }

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SEMICOLON {
                return Err(LoxError::Error("Sync".to_string()));
            };

            match self.peek().unwrap().token_type {
                TokenType::CLASS => (),
                TokenType::FUN => (),
                TokenType::VAR => (),
                TokenType::FOR => (),
                TokenType::IF => (),
                TokenType::WHILE => (),
                TokenType::PRINT => (),
                TokenType::RETURN => (),
                _ => unreachable!(),
            }

            self.advance();
        }
        Err(LoxError::Error("Sync".to_string()))
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn consume(&mut self, token: TokenType, msg: &str) -> Token {
        if self.is_match(token) {
            self.previous()
        } else {
            panic!("{}", msg);
        }
    }

    fn is_match(&mut self, token: TokenType) -> bool {
        if let Some(tok) = self.tokens.get(self.current) {
            if tok.token_type == token {
                self.advance();
                return true;
            }
        }
        false
    }

    fn verify(&mut self, tokens: Vec<TokenType>) -> bool {
        if let Some(tok) = self.tokens.get(self.current) {
            if tokens.iter().any(|x| *x == tok.token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn advance(&mut self) -> Option<Token> {
        if let Some(tok) = self.tokens.get(self.current) {
            self.current += 1;
            Some(tok.clone())
        } else {
            None
        }
    }

    fn check(&self, t: TokenType) -> bool {
        self.tokens.get(self.current).unwrap().token_type == t
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.current).map(|x| x.to_owned())
    }

    fn previous(&mut self) -> Token {
        return self.tokens.get(self.current - 1).unwrap().clone();
    }
}
