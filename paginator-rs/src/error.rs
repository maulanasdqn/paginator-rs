use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PaginatorError {
    InvalidPage(u32),
    InvalidPerPage(u32),
    SerializationError(String),
    Custom(String),
}

impl fmt::Display for PaginatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaginatorError::InvalidPage(page) => {
                write!(f, "Invalid page number: {}. Page must be >= 1", page)
            }
            PaginatorError::InvalidPerPage(per_page) => {
                write!(
                    f,
                    "Invalid per_page value: {}. Must be between 1 and 100",
                    per_page
                )
            }
            PaginatorError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            PaginatorError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for PaginatorError {}

pub type PaginatorResult<T> = Result<T, PaginatorError>;
