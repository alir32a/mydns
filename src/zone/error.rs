use std::io::{Result, Error, ErrorKind};
use anyhow::anyhow;

pub struct ParserError {
    line: u16,
    err: ParserErrorKind
}

impl ParserError {
    pub fn new(line: u16, err: ParserErrorKind) -> Self {
        Self {
            line,
            err
        }
    }
    
    pub fn to_string(&self) -> String {
        format!("line {}: {}", self.line, self.err.to_string())
    }
}

impl Into<Error> for ParserError {
    fn into(self) -> Error {
        Error::new(ErrorKind::Other, self.to_string())
    }
}

impl Into<anyhow::Error> for ParserError {
    fn into(self) -> anyhow::Error {
        anyhow!(self.to_string())
    }
}

pub enum ParserErrorKind {
    UnexpectedEOF,
}

impl ParserErrorKind {
    pub fn to_string(&self) -> String {
        match self {
            ParserErrorKind::UnexpectedEOF => "ParseError: unexpected end of file".to_string()
        }
    }
}