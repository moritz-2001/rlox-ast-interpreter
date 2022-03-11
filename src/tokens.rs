#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Object>,
    pub line: usize,
}

impl Token {
    pub fn new<T>(token_type: TokenType, lexeme: T, literal: Option<Object>, line: usize) -> Self 
    where
        T: Into<String>
    {
        Self {
            token_type,
            lexeme: lexeme.into(),
            literal,
            line
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    String(String),
    Number(f64),
    False,
    True,
    Nil
}


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    // Single-character tokens
    LEFT_PAREN, RIGHT_PAREN, LEFT_BRACE, RIGHT_BRACE, COMMA, DOT,
    MINUS, PLUS, SEMICOLON, SLASH, STAR,

    // One or two character tokens
    BANG, BANG_EQUAL,
    EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL,
    LESS, LESS_EQUAL,

    // Literals
    IDENTIFIER, STRING, NUMBER,

    // Keywords
    AND, CLASS, ELSE, FALSE, FUN, FOR, IF, NIL, OR,
    PRINT, RETURN, SUPER, THIS, TRUE, VAR, WHILE,

    EOF
}