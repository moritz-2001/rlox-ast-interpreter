use crate::tokens::{Token, TokenType, Object};
use crate::LoxError;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, LoxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token::new(TokenType::EOF, "", None, self.line));

        Ok(&self.tokens)

    }

    fn scan_token(&mut self) -> Result<(), LoxError> {
        match self.advance() {
            '(' => self.add_token(TokenType::LEFT_PAREN),
            ')' => self.add_token(TokenType::RIGHT_PAREN),
            '{' => self.add_token(TokenType::LEFT_BRACE),  
            '}' => self.add_token(TokenType::RIGHT_BRACE),  
            ',' => self.add_token(TokenType::COMMA),  
            '.' => self.add_token(TokenType::DOT),  
            '-' => self.add_token(TokenType::MINUS),            
            '+' => self.add_token(TokenType::PLUS),  
            ';' => self.add_token(TokenType::SEMICOLON),  
            '*' => self.add_token(TokenType::STAR),
            '!' => {
                if self.verify('=') {
                    self.add_token(TokenType::BANG_EQUAL)
                }  else {
                    self.add_token(TokenType::BANG)
                }
            },
            '=' => {
                if self.verify('=') {
                    self.add_token(TokenType::EQUAL_EQUAL)
                }  else {
                    self.add_token(TokenType::EQUAL)
                }
            },
            '<' => {
                if self.verify('=') {
                    self.add_token(TokenType::LESS_EQUAL)
                }  else {
                    self.add_token(TokenType::LESS)
                }
            },
            '>' => {
                if self.verify('=') {
                    self.add_token(TokenType::GREATER_EQUAL)
                }  else {
                    self.add_token(TokenType::GREATER)
                }
            },
            '/' => {
                if self.verify('/') {
                    while let Some(c) = self.peek() {
                        if c == '\n' {break} else {self.advance();}
                    }
                } else {
                    self.add_token(TokenType::SLASH)
                }

            },
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '\"' => self.string()?,
            ch if ch.is_digit(10) => self.number()?,
            ch if Self::is_alpha(ch) => {
                self.identifier()?;
            },
            ch => return Err(LoxError::Error(format!("Unexpected character '{}' in line {} !", ch, self.line)))

        }

        Ok(())
    }

    fn number(&mut self) -> Result<(), LoxError> {
        while Self::is_digit(self.peek()) { self.advance(); continue };
        
        if let Some(c) = self.peek() {
            if c == '.' && Self::is_digit(self.peek_next()) {
                self.advance();

                while Self::is_digit(self.peek()) { self.advance(); continue };
            }
        }

        if let Some(c) = self.peek() {
            if Self::is_alpha(c) {
                return Err(LoxError::Error(format!("Unexpected character '{}' in line {} !", c, self.line)));
            }
        }

        let n : String = self.source[self.start .. self.current].iter().collect();
        let n : f64 = n.parse().unwrap();

        self.add_token_object(TokenType::NUMBER, Object::Number(n));


        Ok(())
    }

    fn identifier(&mut self) -> Result<(), LoxError> {
        while Self::is_ascii_alphanumeric(self.peek()) {self.advance();}

        //self.add_token(TokenType::IDENTIFIER);
        
        let s: String = self.source[self.start .. self.current].iter().collect();

        match s.as_str() {
            "and" => self.add_token(TokenType::AND),
            "class" => self.add_token(TokenType::CLASS),
            "else" => self.add_token(TokenType::ELSE),
            "false" => self.add_token(TokenType::FALSE),
            "for" => self.add_token(TokenType::FOR),
            "fun" => self.add_token(TokenType::FUN),
            "if" => self.add_token(TokenType::IF),
            "nil" => self.add_token(TokenType::NIL),
            "or" => self.add_token(TokenType::OR),
            "print" => self.add_token(TokenType::PRINT),
            "return" => self.add_token(TokenType::RETURN),
            "super" => self.add_token(TokenType::SUPER),
            "this" => self.add_token(TokenType::THIS),
            "true" => self.add_token(TokenType::TRUE),
            "var" => self.add_token(TokenType::VAR),
            "while" => self.add_token(TokenType::WHILE),
            _ => self.add_token(TokenType::IDENTIFIER),
        }

        Ok(())
    }

    fn string(&mut self) -> Result<(), LoxError> {
        while let Some(ch) = self.peek() {
            if ch == '\"' {break}
            if ch == '\n' {
                self.line += 1;
            }
            self.current += 1;
        }

        if self.is_at_end() { 
            return Err(LoxError::Error(format!("Unterminated string at line {}!", self.line)))
        }   
        // The closing "
        self.advance();

        let val: String = self.source[(self.start+1) .. (self.current - 1)].iter().collect();

        self.add_token_object(TokenType::STRING, Object::String(val));

        Ok(())
    }



    fn advance(&mut self) -> char {
        let c = *self.source.get(self.current).unwrap();
        self.current += 1;

        c
    }

    fn verify(&mut self, c: char) -> bool {
        if let Some(ch) = self.source.get(self.current) {
            if *ch == c {
                self.advance();
                return true;
            }
        }
        
        false
    }

    fn is_digit(c: Option<char>) -> bool {
        if let Some(c) = c {
            if c.is_ascii_digit() {return true;}
        }
        return false;
    }

    fn is_alpha<T>(c: T) -> bool where T: Into<Option<char>> {
        let c: Option<char> = c.into();
        if let Some(c) = c {
            if c.is_ascii_alphanumeric() || c == '_' {return true;}
        }
        return false;
    }

    fn is_ascii_alphanumeric(c: Option<char>) -> bool {
        if let Some(c) = c {
            return Self::is_alpha(c) || c.is_ascii_digit();
        }
        return false;
    }

    
    fn peek(&self) -> Option<char> {
        self.source.get(self.current).map(|x| *x)
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.current + 1).map(|x| *x)
    }

    fn add_token(&mut self, token_type: TokenType) {
        let s: String = self.source[self.start .. self.current].iter().collect();
        let token = Token::new(token_type, s, None, self.line);
        println!("{:?}", token);
        self.tokens.push(token);
    }

    fn add_token_object(&mut self, token_type: TokenType, object: Object) {
        let s: String = self.source[self.start .. self.current].iter().collect();
        let token = Token::new(token_type, s, Some(object), self.line);
        println!("{:?}", token);
        self.tokens.push(token);
    } 
}