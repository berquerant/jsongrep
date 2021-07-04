use serde_json;
use std::fmt;
use std::io;
use std::result;
use thiserror::Error;

#[derive(Debug)]
pub struct Error {
    err: Box<ErrorImpl>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

pub type Result<T> = result::Result<T, Error>;

impl Error {
    pub fn new(code: ErrorCode) -> Self {
        Error {
            err: Box::new(ErrorImpl { code }),
        }
    }
    pub fn unreachable() -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::Unreachable,
            }),
        }
    }
    pub fn is_filtered(&self) -> bool {
        matches!(self.err.code, ErrorCode::FilteredByQuery)
    }

    pub fn is_io(&self) -> bool {
        matches!(self.err.code, ErrorCode::Io(_))
    }

    pub fn is_json(&self) -> bool {
        matches!(self.err.code, ErrorCode::Json(_))
    }
}

#[derive(Debug)]
struct ErrorImpl {
    code: ErrorCode,
}

impl fmt::Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

#[derive(Error, Debug)]
pub enum ErrorCode {
    #[error(transparent)]
    Json(#[from] serde_json::error::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Invalid regex ({0})")]
    InvalidRegex(String),
    #[error("Unreachable")]
    Unreachable,
    #[error("Type mismatch (want {want:?}, got {got:?}, by {by:?})")]
    TypeMismatch {
        got: String,
        want: String,
        by: String,
    },
    #[error("No children (by: {by:?})")]
    NoChildren { by: String },
    #[error("Matcher type mismatch (matcher_type {matcher_type:?}, matcher_value {matcher_value:?}, target {target:?}, by {by:?})")]
    MatcherTypeMismatch {
        matcher_type: String,
        matcher_value: String,
        target: String,
        by: String,
    },
    #[error("Invalid target (pointer: {pointer:?}, value: {value:?})")]
    InvalidTarget { pointer: String, value: String },
    #[error("Invalid pointer (pointer: {pointer:?}, value: {value:?})")]
    InvalidPointer { pointer: String, value: String },
    #[error("Filtered by query")]
    FilteredByQuery,
    #[error("InvalidOption ({0})")]
    InvalidOption(String),
}
