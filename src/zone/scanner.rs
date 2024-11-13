use crate::zone::token::{Token, TokenType};
use anyhow::Result;

#[derive(Default)]
pub struct Scanner {
    lines: Vec<String>,
}

impl Scanner {
    pub fn new(buf: Vec<u8>) -> Result<Self> {
        let source = String::from_utf8(buf)?;

        Ok(Self {
            lines: source.lines().map(|val| val.to_string()).collect::<Vec<String>>(),
        })
    }
    
    pub fn scan(&self) -> Result<Vec<Token>> {
        let mut res = Vec::new();
        
        for line in 0..self.lines.len()  {
            res.append(&mut Self::scan_line(self.lines[line].as_str(), line as u16));
        }
        
        Ok(res)
    }
    
    fn scan_line(line: &str, line_num: u16) -> Vec<Token> {
        let mut res = Vec::new();
        let chars = line.chars();

        let mut temp_str = String::new();
        let mut chars = chars.collect::<Vec<char>>().into_iter().enumerate();
        
        while let Some((pos, ch)) = chars.next() {
            if ch.is_whitespace() {
                if pos == 0 && temp_str.is_empty() {
                    res.push(Token::new(temp_str.as_str(), TokenType::WhiteSpace, line_num));
                }

                if !temp_str.is_empty() {
                    res.push(Token::new(temp_str.as_str(), TokenType::String, line_num));
                    temp_str.clear();
                }
                
                continue;
            }

            match ch {
                '$' => {
                    if !temp_str.is_empty() {
                        res.push(Token::new(temp_str.as_str(), TokenType::String, line_num));
                        temp_str.clear();
                    }
                    res.push(Token::new(&ch.to_string(), TokenType::DolorSign, line_num));
                },
                ';' => {
                    return res
                },
                '@' => {
                    if !temp_str.is_empty() {
                        res.push(Token::new(temp_str.as_str(), TokenType::String, line_num));
                        temp_str.clear();
                    }
                    res.push(Token::new(&ch.to_string(), TokenType::AtSign, line_num));
                },
                '(' => {
                    if !temp_str.is_empty() {
                        res.push(Token::new(temp_str.as_str(), TokenType::String, line_num));
                        temp_str.clear();
                    }
                    res.push(Token::new(&ch.to_string(), TokenType::LeftParenthesis, line_num));
                },
                ')' => {
                    if !temp_str.is_empty() {
                        res.push(Token::new(temp_str.as_str(), TokenType::String, line_num));
                        temp_str.clear();
                    }
                    res.push(Token::new(&ch.to_string(), TokenType::RightParenthesis, line_num));
                },
                _ => {
                    temp_str.push(ch);
                }
            }
        }
        
        if !temp_str.is_empty() {
            res.push(Token::new(temp_str.as_str(), TokenType::String, line_num));
        }
        
        res
    }
    
    pub(crate) fn replace(&mut self, s: &String) {
        self.lines = s.lines().map(|val| val.to_string()).collect::<Vec<String>>();
    }
}
