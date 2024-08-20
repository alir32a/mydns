use std::fs::File;
use std::io::{Result, Error, ErrorKind, Write};
use crate::header::Header;
use crate::query_type::QueryType;
use crate::question::Question;
use crate::record::Record;

#[derive(Default)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub resources: Vec<Record>,
}

impl Packet {
    pub fn new() -> Packet {
        Packet {
            ..Default::default()
        }
    }
}