use std::collections::{HashMap};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::time::Duration;
use std::sync::Arc;
use anyhow::{bail, Result};
use rand::{random};
use tracing::{error, info};
use crate::cache::{DnsCache, DnsCacheItem};
use crate::query_type::QueryType;
use crate::handler::{Handler, UdpHandler};
use crate::packet::Packet;
use crate::parser::PacketParser;
use crate::question::Question;
use crate::record::{Record, RecordData};
use crate::result_code::ResultCode;
use crate::root::{get_root_servers_socket_addrs};
use crate::writer::PacketWriter;

pub trait Resolver {
    fn resolve(&mut self, buf: &[u8]) -> Result<Vec<u8>>;
}

pub struct RecursiveResolver {
    pub base_handler: Box<dyn Handler>,
    cache: Arc<DnsCache>,
    max_recursion_depth: u16,
}

impl RecursiveResolver {
    pub fn new() -> Self {
        Self {
            base_handler: Box::new(UdpHandler::new(get_root_servers_socket_addrs(false), Duration::from_secs(5), false)),
            cache: Arc::new(DnsCache::new()),
            max_recursion_depth: 10
        }
    }

    pub fn recursive_lookup(
        &mut self,
        question: &Question,
        addrs: Option<Vec<SocketAddr>>,
        depth: u16
    ) -> Result<Packet> {
        if depth == self.max_recursion_depth {
            bail!("maximum resolve recursion depth exceeded")
        }

        let res = lookup(self.cache.clone(), &mut self.base_handler, &question, addrs)?;
        
        if is_resolved(question, &res) {
            return Ok(res);
        }

        let mut cnames: Vec<&Record> = res.answers.iter().filter(|&record| {
            record.rtype == QueryType::CNAME
        }).collect();

        // some servers may send cname records in additional section
        cnames.append(&mut res.resources.iter().filter(|&record| {
            record.rtype == QueryType::CNAME
        }).collect());

        if !cnames.is_empty() {
            if let Some(domain) = get_final_cname(&question.domain, cnames) {
                return self.recursive_lookup(
                    &Question::new(domain, question.qtype), 
                    None,
                    depth + 1);
            }
        }
        
        let resolved_ns = get_resolved_ns(&res.resources);
        if !resolved_ns.is_empty() {
            return self.recursive_lookup(
                question,
                Some(resolved_ns),
                depth + 1);
        }

        let ns = self.get_unresolved(&res.authorities)?;
        if !ns.is_empty() {
            return self.recursive_lookup(question, Some(ns), depth + 1);
        }

        Ok(res)
    }
    
    fn get_unresolved(&mut self, authorities: &Vec<Record>) -> Result<Vec<SocketAddr>> {
        let mut res: Vec<SocketAddr> = Vec::new();

        for authority in authorities {
            if let RecordData::NS(domain) = &authority.data {
                let ns = self.recursive_lookup(
                    &Question::new(domain.clone(), QueryType::A),
                    None,
                    0)?;

                if ns.header.code == ResultCode::NOERROR.to_u8() {
                    ns.answers.iter().for_each(|resource| {
                        match resource.rtype {
                            QueryType::A => {
                                if let RecordData::A(addr) = resource.data {
                                    res.push(SocketAddr::from(SocketAddrV4::new(addr, 53)))
                                }
                            },
                            QueryType::AAAA => {
                                if let RecordData::AAAA(addr) = resource.data {
                                    res.push(SocketAddr::from(SocketAddrV6::new(addr, 53, 0, 0)))
                                }
                            },
                            _ => {}
                        }
                    });
                }
            }
        }
        
        Ok(res)
    }
}

impl Resolver for RecursiveResolver {
    fn resolve(&mut self, buf: &[u8]) -> Result<Vec<u8>> {
        let req = PacketParser::new(buf).parse()?;
        let mut res = Packet::from(&req);
        res.header.recursion_available = true;
        res.header.response = true;

        if req.questions.len() == 0 {
            let mut res = Packet::from(&req);
            res.header.recursion_available = true;
            res.header.response = true;
            res.header.code = ResultCode::FORMERR.to_u8();

            return PacketWriter::from(res).write();
        }

        for question in req.questions {
            match self.recursive_lookup(&question, None, 0) {
                Ok(result) => {
                    res.header.code = result.header.code;
                    
                    append_results(&mut res, result);
                },
                Err(e) => {
                    error!("error occurred while serving a query: {}", e);
                    
                    res.header.code = ResultCode::SERVFAIL.to_u8();

                    break;
                }
            }
        }

        PacketWriter::from(res).write()
    }
}

pub struct ForwardResolver {
    pub base_handler: Box<dyn Handler>,
    cache: Arc<DnsCache>,
}

impl ForwardResolver {
    pub fn new(addrs: Vec<SocketAddr>) -> Self {
        Self {
            base_handler: Box::new(UdpHandler::new(addrs, Duration::from_secs(5), false)),
            cache: Arc::new(DnsCache::new())
        }
    }
}

impl Resolver for ForwardResolver {
    fn resolve(&mut self, buf: &[u8]) -> Result<Vec<u8>> {
        let req = PacketParser::new(buf).parse()?;
        let mut res = Packet::from(&req);
        res.header.recursion_available = true;
        res.header.response = true;

        if req.questions.len() == 0 {
            let mut res = Packet::from(&req);
            res.header.recursion_available = true;
            res.header.response = true;
            res.header.code = ResultCode::FORMERR.to_u8();

            return PacketWriter::from(res).write();
        }

        for question in req.questions {
            if let Ok(result) = lookup(self.cache.clone(), &mut self.base_handler, &question, None) {
                append_results(&mut res, result);
            } else {
                res.header.code = ResultCode::SERVFAIL.to_u8();

                break;
            }
        }

        PacketWriter::from(res).write()
    }
}

pub fn lookup(
    cache: Arc<DnsCache>,
    handler: &mut Box<dyn Handler>, 
    question: &Question, 
    addrs: Option<Vec<SocketAddr>>)
    -> Result<Packet> {
    let req = new_query_packet(question.clone());
    
    if let Some(records) = cache.get(question.domain.as_str()) {
        return Ok(create_resp_packet(&req, records));
    }

    let req_buf = PacketWriter::from(req).write()?;

    let mut res_buf = Vec::new();
    match addrs { 
        Some(addrs) => {
            res_buf = handler.send_to(req_buf.as_slice(), addrs.as_slice())?;
        },
        None => {
            res_buf = handler.send(req_buf.as_slice())?;
        }
    }

    let mut res = PacketParser::new(&res_buf).parse()?;
    res.header.recursion_available = true;
    res.header.response = true;
    
    if !res.answers.is_empty() {
        let filter = |record: &Record| {
            if record.domain != question.domain {
                return None;    
            }
            
            return match record.rtype {
                QueryType::A | QueryType::AAAA | QueryType::SOA => {
                    Some(record.clone())
                },
                _ => {
                    None
                }
            }
        };
        
        let mut resolved: Vec<Record> = res.answers.iter().filter_map(filter).collect();
        resolved.append(&mut res.resources.iter().filter_map(filter).collect());
        
        cache.set(question.domain.as_str(), DnsCacheItem::new(resolved));
    }
    
    Ok(res)
}

fn is_resolved(question: &Question, result: &Packet) -> bool {
    if result.answers.is_empty() {
        return false;
    }

    if result.header.code == ResultCode::NXDOMAIN.to_u8() {
        return true;
    }

    result.answers.iter().any(|record| {
        record.rtype == question.qtype
    })
}

// sometimes there is more than one cname records, this function will get the final
// cname record to resolve.
fn get_final_cname(query: &String, cnames: Vec<&Record>) -> Option<String> {
    let mut resolved = HashMap::new();

    cnames.iter().for_each(|record| {
        if let RecordData::CNAME(domain) = &record.data {
            resolved.insert(&record.domain, domain);
        }
    });

    let mut result = resolved.remove(&query)?;
    for _ in 0..resolved.len() {
        if let Some(res) = resolved.get(result) {
            result = res;
        }
    }

    Some(result.clone())
}

fn get_resolved_ns(resources: &Vec<Record>) -> Vec<SocketAddr> {
    let mut res: Vec<SocketAddr> = Vec::new();

    resources.iter().for_each(|resource| {
        match resource.rtype {
            QueryType::A => {
                if let RecordData::A(addr) = resource.data {
                    res.push(SocketAddr::from(SocketAddrV4::new(addr, 53)))
                }
            },
            QueryType::AAAA => {
                if let RecordData::AAAA(addr) = resource.data {
                    res.push(SocketAddr::from(SocketAddrV6::new(addr, 53, 0, 0)))
                }
            },
            _ => ()
        }
    });

    res
}

fn append_results(dst: &mut Packet, src: Packet) {
    dst.header.answer_count += src.header.answer_count;
    dst.header.authority_count += src.header.authority_count;
    dst.header.resource_count += src.header.resource_count;

    for answer in src.answers {
        dst.answers.push(answer);
    }

    for authority in src.authorities {
        dst.authorities.push(authority);
    }

    for resource in src.resources {
        dst.resources.push(resource);
    }
}

fn new_query_packet(question: Question) -> Packet {
    let mut req = Packet::new();
    req.header.id = random();
    req.header.recursion_desired = true;
    req.header.question_count = 1;
    req.questions.push(question);

    req
}

fn create_resp_packet(req: &Packet, records: Vec<Record>) -> Packet {
    let mut packet = Packet::from(req);
    packet.header.recursion_available = true;
    packet.header.response = true;
    packet.header.answer_count = records.len() as u16;

    packet.answers = records;
    
    packet
}
