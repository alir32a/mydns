use std::cmp::PartialEq;
use crate::cache::DnsCache;
use crate::context::ServerMode::Authoritive;

pub struct Context {
    cache: DnsCache,
    mode: ServerMode,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ServerMode {
    Authoritive,
    Recursive,
    Proxy
}

impl ServerMode {
    pub fn recursive_available(&self) -> bool {
        *self != Authoritive
    }
}