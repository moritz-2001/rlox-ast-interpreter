#[derive(Debug)]
pub enum LoxError {
    IoError(std::io::Error),
    Error(String),
    ParsingError(String),
    UndefinedVariable(String),
    TokenListEmpty,
    NotExpression,
}

impl From<std::io::Error> for LoxError {
    fn from(err: std::io::Error) -> Self {
        LoxError::IoError(err)
    }
}
