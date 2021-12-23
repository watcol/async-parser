use std::error::Error;
use std::fmt;
use std::io;
use std::ops::Range;

/// An alias for `Result<T, ParseError>`.
pub type Result<T> = std::result::Result<T, ParseError>;

/// A parse error.
#[derive(Debug)]
pub enum ParseError {
    /// Expected something at the position.
    Expected {
        position: Range<usize>,
        expected: String,
    },
    /// An error while reading from `AsyncRead`.
    ReadError { inner: io::Error },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Expected { expected, .. } => write!(f, "expected: {}", expected),
            Self::ReadError { inner } => write!(f, "{}", inner),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Expected { .. } => None,
            Self::ReadError { inner } => Some(inner),
        }
    }
}
