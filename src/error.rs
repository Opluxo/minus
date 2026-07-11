use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MinusError {
    #[error("Lexer error at line {line}, col {col}: {message}")]
    LexerError {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Parser error at line {line}, col {col}: {message}")]
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Type error: {message}")]
    TypeError { message: String },

    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },

    #[error("Undefined function: {name}")]
    UndefinedFunction { name: String },

    #[error("Duplicate variable: {name}")]
    DuplicateVariable { name: String },

    #[error("Duplicate function: {name}")]
    DuplicateFunction { name: String },

    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    #[error("Missing return statement in function that returns {ty}")]
    MissingReturn { ty: String },

    #[error("Main function not found")]
    MainNotFound,

    #[error("Import error: {message}")]
    ImportError { message: String },

    #[error("Code generation error: {message}")]
    CodegenError { message: String },

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for MinusError {
    fn from(err: std::io::Error) -> Self {
        MinusError::IoError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MinusError>;
