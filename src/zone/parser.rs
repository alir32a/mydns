use std::fs;
use crate::zone::fs::read_dir;
use std::iter::Enumerate;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::str::FromStr;
use std::vec::IntoIter;
use crate::record::{Record, RecordData};
use anyhow::{bail, Result};
use crate::query_class::QueryClass;
use crate::query_type::QueryType;
use crate::zone::error::{ParserError, ParserErrorKind};
use crate::zone::scanner::Scanner;
use crate::zone::token::{Keyword, Token, TokenType};

#[derive(Default, Debug)]
pub struct Zone {
    pub(crate) origin: String,
    pub(crate) ttl: Option<usize>,
    pub(crate) records: Vec<Record>
}

impl Zone {
    pub fn parse_file<P: AsRef<Path>>(p: P) -> Result<Zone> {
        let src = fs::read(p)?;
        
        parse(src)
    }
    
    pub fn parse_directory<P: AsRef<Path>>(p: P, recursive: bool) -> Result<Vec<Zone>> {
        let mut zones: Vec<Zone> = Vec::new();
        let files = read_dir(p, recursive)?;
        
        for file in files {
            zones.push(Self::parse_file(file)?);
        }
        
        Ok(zones)
    }
}

fn parse(src: Vec<u8>) -> Result<Zone> {
    let mut res = Zone{
        ..Default::default()
    };

    let scanner = Scanner::new(src)?;
    let mut tokens = scanner.scan()?.into_iter().enumerate();
    
    while let Some((_pos, token)) = tokens.next() {
        match token.token_type {
            TokenType::DolorSign => {
                let keyword = get_next_token(&mut tokens, token.line)?;
                let value = get_next_token(&mut tokens, token.line)?;
                
                parse_keywords(keyword, value, &mut res)?;
            },
            _ => {
                let mut record = Record::default();
                
                if let Some(ttl) = res.ttl {
                    record.ttl = ttl as u32;
                }
                
                let domain = token.clone();

                match domain.token_type {
                    TokenType::AtSign => {
                        if res.origin.is_empty() {
                            bail!("invalid use of @, origin not defined at line {}", domain.line);
                        }
                        
                        record.domain = res.origin.clone()
                    },
                    TokenType::String => {
                        record.domain = to_domain(domain.lexeme, &res.origin);
                    },
                    TokenType::WhiteSpace => {
                        record.domain = res.records.last().unwrap().domain.clone()
                    },
                    _ => {
                        bail!("invalid domain {} at line {}", domain.lexeme, domain.line)
                    }
                }

                let mut next_token = get_next_token(&mut tokens, token.line)?;
                if let Ok(ttl) = next_token.lexeme.parse::<usize>() {
                    record.ttl = ttl as u32;
                    next_token = get_next_token(&mut tokens, token.line)?;
                }

                match next_token.lexeme.as_str() {
                    "IN" => {
                        record.rclass = QueryClass::IN
                    },
                    _ => {
                        bail!("unsupported query class: {}", next_token.lexeme)
                    }
                }

                let typ = get_next_token(&mut tokens, token.line)?;
                match typ.lexeme.to_uppercase().as_str() {
                    "A" => {
                        record.rtype = QueryType::A;
                        
                        let addr_token = get_next_token(&mut tokens, token.line)?;
                        
                        match Ipv4Addr::from_str(addr_token.lexeme.as_str()) {
                            Ok(addr) => {
                                record.data = RecordData::A(addr);        
                            },
                            Err(_) => {
                                bail!("cannot parse {} as a valid ip v4 address", addr_token.lexeme);
                            }
                        }
                    },
                    "AAAA" => {
                        record.rtype = QueryType::AAAA;

                        let addr_token = get_next_token(&mut tokens, token.line)?;

                        match Ipv6Addr::from_str(addr_token.lexeme.as_str()) {
                            Ok(addr) => {
                                record.data = RecordData::AAAA(addr);
                            },
                            Err(_) => {
                                bail!("cannot parse {} as a valid ip v4 address", addr_token.lexeme);
                            }
                        }
                    },
                    "NS" => {
                        record.rtype = QueryType::NS;
                        
                        let ns = get_next_token(&mut tokens, token.line)?;
                        
                        record.data = RecordData::NS(to_domain(ns.lexeme, &res.origin));
                    },
                    "CNAME" => {
                        record.rtype = QueryType::CNAME;

                        let cname = get_next_token(&mut tokens, token.line)?;

                        record.data = RecordData::CNAME(to_domain(cname.lexeme, &res.origin));
                    },
                    "SOA" => {
                        record.rtype = QueryType::SOA;
                        
                        let mname = get_next_token(&mut tokens, token.line)?;
                        let rname = get_next_token(&mut tokens, token.line)?;

                        let left_paran = get_next_token(&mut tokens, token.line)?;
                        if left_paran.token_type != TokenType::LeftParenthesis {
                            bail!("expected ) found {}", left_paran.lexeme)
                        }
                        
                        let serial = get_next_non_empty_token(&mut tokens, token.line)?;
                        let refresh = get_next_non_empty_token(&mut tokens, token.line)?;
                        let retry = get_next_non_empty_token(&mut tokens, token.line)?;
                        let expire = get_next_non_empty_token(&mut tokens, token.line)?;
                        let minimum = get_next_non_empty_token(&mut tokens, token.line)?;
                        
                        let right_paren = get_next_non_empty_token(&mut tokens, token.line)?;
                        if right_paren.token_type != TokenType::RightParenthesis {
                            bail!("expected ( found {}", right_paren.lexeme)
                        }
                        
                        record.data = RecordData::SOA {
                            mname: to_domain(mname.lexeme, &res.origin),
                            rname: to_domain(rname.lexeme, &res.origin),
                            serial: serial.lexeme.parse()?,
                            refresh: refresh.lexeme.parse()?,
                            retry: retry.lexeme.parse()?,
                            expire: expire.lexeme.parse()?,
                            minimum: minimum.lexeme.parse()?
                        }
                    },
                    "MX" => {
                        record.rtype = QueryType::MX;

                        let preference = get_next_token(&mut tokens, token.line)?;
                        let exchange = get_next_token(&mut tokens, token.line)?;

                        record.data = RecordData::MX { 
                            preference: preference.lexeme.parse()?, 
                            exchange: to_domain(exchange.lexeme, &res.origin) 
                        };
                    },
                    "PTR" => {
                        record.rtype = QueryType::PTR;

                        let ptr = get_next_token(&mut tokens, token.line)?;

                        record.data = RecordData::PTR(to_domain(ptr.lexeme, &res.origin));
                    },
                    "HINFO" => {
                        record.rtype = QueryType::HINFO;

                        let cpu = get_next_token(&mut tokens, token.line)?;
                        let os = get_next_token(&mut tokens, token.line)?;

                        record.data = RecordData::HINFO {
                            cpu: cpu.lexeme,
                            os: os.lexeme,
                        };
                    },
                    "TXT" => {
                        record.rtype = QueryType::TXT;

                        let txt = get_next_token(&mut tokens, token.line)?;

                        record.data = RecordData::TXT(txt.lexeme);
                    },
                    _ => {
                        bail!("unsupported record type: {}", typ.lexeme)
                    }
                }
                
                res.records.push(record);
            }
        }
    }
    
    if !res.records.iter().any(|record| {
        record.rtype == QueryType::SOA
    }) {
        bail!("expected one SOA records")
    }

    Ok(res)
}

fn parse_keywords(keyword: Token, value: Token, res: &mut Zone) -> Result<()> {
    match Keyword::from(keyword.lexeme.as_str()) {
        Some(keyword) => {
            match keyword {
                Keyword::Origin => {
                    res.origin = trim_end(value.lexeme.to_owned());
                },
                Keyword::TTL => {
                    match usize::from_str(value.lexeme.as_str()) {
                        Ok(ttl) => {
                            res.ttl = Some(ttl)
                        },
                        Err(e) => {
                            bail!("invalid input for ttl {} at line {}", value.lexeme, value.line)
                        }
                    }
                },
                _ => {
                    bail!("unsupported keyword {} at line {}", keyword.to_string(), value.line)
                }
            }
        },
        None => {
            bail!("unknown keyword {} at line {}", keyword.lexeme, keyword.line)
        }
    }
    
    Ok(())
}

fn to_domain(mut s: String, origin: &String) -> String {
    if s.ends_with(".") {
        s.remove(s.len()-1);
        return s
    }
    
    [s, origin.clone()].join(".")
}

fn trim_end(mut s: String) -> String {
    if s.ends_with(".") {
        s.remove(s.len()-1);
    }
    
    s
}

fn get_next_token(t: &mut Enumerate<IntoIter<Token>>, line: u16) -> Result<Token> {
    let (_, token) = t.
        next().
        ok_or::<anyhow::Error>(ParserError::new(line, ParserErrorKind::UnexpectedEOF).into())?;
    
    Ok(token)
}

fn get_next_non_empty_token(t: &mut Enumerate<IntoIter<Token>>, line: u16) -> Result<Token> {
    while let Some((_, token)) = t.next() {
        if token.token_type != TokenType::WhiteSpace {
            return Ok(token)
        }
    }

    Err(ParserError::new(line, ParserErrorKind::UnexpectedEOF).into())
}
