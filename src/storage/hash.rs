use std::collections::HashMap;

const HASH_MAX_ZIPLIST_ENTRIES: usize = 128;
const HASH_MAX_ZIPLIST_VALUE: usize = 64;

/// Redis Hash data structure with ziplist + hashtable encoding
#[derive(Debug, Clone)]
pub enum RedisHash {
    /// Small hash: vector of (field, value) pairs (ziplist encoding)
    ZipList(Vec<(Vec<u8>, Vec<u8>)>),
    /// Large hash: HashMap
    HashMap(HashMap<Vec<u8>, Vec<u8>>),
}

impl RedisHash {
    pub fn new() -> Self {
        RedisHash::ZipList(Vec::new())
    }

    /// Convert from ziplist to hashtable if thresholds exceeded
    fn maybe_convert(&mut self) {
        if let RedisHash::ZipList(entries) = self {
            let should_convert = entries.len() > HASH_MAX_ZIPLIST_ENTRIES
                || entries
                    .iter()
                    .any(|(k, v)| k.len() > HASH_MAX_ZIPLIST_VALUE || v.len() > HASH_MAX_ZIPLIST_VALUE);
            if should_convert {
                let map: HashMap<Vec<u8>, Vec<u8>> = entries.drain(..).collect();
                *self = RedisHash::HashMap(map);
            }
        }
    }

    pub fn hset(&mut self, field: Vec<u8>, value: Vec<u8>) -> bool {
        let is_new = match self {
            RedisHash::ZipList(entries) => {
                if let Some(entry) = entries.iter_mut().find(|(k, _)| k == &field) {
                    entry.1 = value;
                    false
                } else {
                    entries.push((field, value));
                    true
                }
            }
            RedisHash::HashMap(map) => map.insert(field, value).is_none(),
        };
        self.maybe_convert();
        is_new
    }

    pub fn hget(&self, field: &[u8]) -> Option<&Vec<u8>> {
        match self {
            RedisHash::ZipList(entries) => {
                entries.iter().find(|(k, _)| k.as_slice() == field).map(|(_, v)| v)
            }
            RedisHash::HashMap(map) => map.get(field),
        }
    }

    pub fn hdel(&mut self, field: &[u8]) -> bool {
        match self {
            RedisHash::ZipList(entries) => {
                let len_before = entries.len();
                entries.retain(|(k, _)| k.as_slice() != field);
                entries.len() < len_before
            }
            RedisHash::HashMap(map) => map.remove(field).is_some(),
        }
    }

    pub fn hlen(&self) -> usize {
        match self {
            RedisHash::ZipList(entries) => entries.len(),
            RedisHash::HashMap(map) => map.len(),
        }
    }

    pub fn hexists(&self, field: &[u8]) -> bool {
        self.hget(field).is_some()
    }

    pub fn hgetall(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        match self {
            RedisHash::ZipList(entries) => entries.clone(),
            RedisHash::HashMap(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        }
    }

    pub fn hkeys(&self) -> Vec<Vec<u8>> {
        match self {
            RedisHash::ZipList(entries) => entries.iter().map(|(k, _)| k.clone()).collect(),
            RedisHash::HashMap(map) => map.keys().cloned().collect(),
        }
    }

    pub fn hvals(&self) -> Vec<Vec<u8>> {
        match self {
            RedisHash::ZipList(entries) => entries.iter().map(|(_, v)| v.clone()).collect(),
            RedisHash::HashMap(map) => map.values().cloned().collect(),
        }
    }

    pub fn hincrby(&mut self, field: Vec<u8>, delta: i64) -> Result<i64, String> {
        let current = self
            .hget(&field)
            .and_then(|v| std::str::from_utf8(v).ok())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let new_val = current
            .checked_add(delta)
            .ok_or_else(|| "ERR increment or decrement would overflow".to_string())?;
        self.hset(field, new_val.to_string().into_bytes());
        Ok(new_val)
    }

    pub fn hincrbyfloat(&mut self, field: Vec<u8>, delta: f64) -> Result<f64, String> {
        let current = self
            .hget(&field)
            .and_then(|v| std::str::from_utf8(v).ok())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let new_val = current + delta;
        if new_val.is_nan() || new_val.is_infinite() {
            return Err("ERR increment would produce NaN or Infinity".to_string());
        }
        self.hset(field, format!("{}", new_val).into_bytes());
        Ok(new_val)
    }

    pub fn hsetnx(&mut self, field: Vec<u8>, value: Vec<u8>) -> bool {
        if self.hexists(&field) {
            return false;
        }
        self.hset(field, value);
        true
    }
}
