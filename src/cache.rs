use std::collections::{HashMap};
use std::ops::{Add};
use std::sync::{RwLock, Arc};
use tokio::time::{Duration, Instant};
use crate::record::Record;
use crate::record::RecordData::SOA;

pub struct DnsCache {
    map: RwLock<HashMap<String, DnsCacheItem>>
}

impl DnsCache {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new())
        }
    }
    
    pub fn get(&self, key: &str) -> Option<Vec<Record>> {
        let mut res: Vec<Record> = Vec::new();
        
        match self.map.read().expect("dns cache lock poisoned").get(key) {
            Some(val) => {
                res = val.records.iter().filter_map(|record| -> Option<Record> {
                    let mut ttl = Duration::from_secs(record.ttl as u64);
                    
                    if let SOA {expire, ..} = record.data {
                        ttl = Duration::from_secs(expire as u64);
                    }

                    if val.timestamp.add(ttl).elapsed() == Duration::ZERO {
                        return Some(record.clone());
                    }
                    
                    None
                }).collect();
            },
            None => {
                return None;
            }
        }
        
        if res.is_empty() {
            self.map.write().expect("dns cache lock poisoned").remove(key);
            
            return None;
        }
        
        Some(res)
    }
    
    pub fn set(&self, key: &str, val: DnsCacheItem) {
        self.map.write().expect("dns cache lock poisoned").insert(key.to_string(), val);
    }
}

#[derive(Debug)]
pub struct DnsCacheItem {
    pub(crate) records: Arc<Vec<Record>>,
    pub(crate) timestamp: Instant
}

impl DnsCacheItem {
    pub fn new(records: Vec<Record>) -> Self {
        Self {
            records: Arc::new(records),
            timestamp: Instant::now()
        }
    }
}