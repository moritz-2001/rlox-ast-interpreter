

#[derive(Debug)]
pub enum LoxError {
    IoError(std::io::Error),
    Error(String),
}

impl From<std::io::Error> for LoxError {
    fn from(err: std::io::Error) -> Self {
        LoxError::IoError(err)
    }
}