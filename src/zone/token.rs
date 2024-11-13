use crate::zone::token::Keyword::{Generate, Include, Origin, TTL};

pub enum Keyword {
    Origin,
    Include,
    TTL,
    Generate
}

impl Keyword {
    pub fn to_string(&self) -> String {
        match self { 
            Origin => "origin".to_string(),
            TTL => "ttl".to_string(),
            Include => "include".to_string(),
            Generate => "generate".to_string()
        }
    }
    
    pub fn from(s: &str) -> Option<Keyword> {
        match s.to_lowercase().as_str() { 
            "origin" => Some(Origin),
            "ttl" => Some(TTL),
            "include" => Some(Include),
            "generate" => Some(Generate),
            _ => None
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TokenType {
    DolorSign,
    SemiColon,
    LeftParenthesis,
    RightParenthesis,
    AtSign,
    String,
    WhiteSpace,
    EOL // End Of Line
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: u16,
}

impl Token {
    pub fn new(lexeme: &str, token_type: TokenType, line: u16) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_string(),
            line
        }
    }
}