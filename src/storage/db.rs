use std::collections::HashMap;
use std::time::Instant;

use super::object::RedisObject;

/// Redis database: keyspace + expiry management
pub struct Database {
    /// Main keyspace
    keyspace: HashMap<Vec<u8>, RedisObject>,
    /// Expiry dictionary: key -> expiry time
    expires: HashMap<Vec<u8>, Instant>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            keyspace: HashMap::new(),
            expires: HashMap::new(),
        }
    }

    /// Check if a key is expired and remove it if so (lazy expiration)
    fn check_expired(&mut self, key: &[u8]) -> bool {
        if let Some(&expiry) = self.expires.get(key) {
            if Instant::now() >= expiry {
                self.keyspace.remove(key);
                self.expires.remove(key);
                return true;
            }
        }
        false
    }

    /// Get a reference to the value for a key
    pub fn get(&mut self, key: &[u8]) -> Option<&RedisObject> {
        self.check_expired(key);
        self.keyspace.get(key)
    }

    /// Get a mutable reference to the value for a key
    pub fn get_mut(&mut self, key: &[u8]) -> Option<&mut RedisObject> {
        self.check_expired(key);
        self.keyspace.get_mut(key)
    }

    /// Set a key-value pair
    pub fn set(&mut self, key: Vec<u8>, value: RedisObject) {
        self.expires.remove(&key);
        self.keyspace.insert(key, value);
    }

    /// Set a key-value pair with expiry in milliseconds
    pub fn set_with_expiry(&mut self, key: Vec<u8>, value: RedisObject, ms: u64) {
        let expiry = Instant::now() + std::time::Duration::from_millis(ms);
        self.keyspace.insert(key.clone(), value);
        self.expires.insert(key, expiry);
    }

    /// Delete a key
    pub fn del(&mut self, key: &[u8]) -> bool {
        self.expires.remove(key);
        self.keyspace.remove(key).is_some()
    }

    /// Check if a key exists
    pub fn exists(&mut self, key: &[u8]) -> bool {
        self.check_expired(key);
        self.keyspace.contains_key(key)
    }

    /// Get the type name of a key's value
    pub fn key_type(&mut self, key: &[u8]) -> &'static str {
        self.check_expired(key);
        match self.keyspace.get(key) {
            Some(obj) => obj.type_name(),
            None => "none",
        }
    }

    /// Set expiry on a key (seconds)
    pub fn expire(&mut self, key: &[u8], seconds: u64) -> bool {
        self.check_expired(key);
        if self.keyspace.contains_key(key) {
            let expiry = Instant::now() + std::time::Duration::from_secs(seconds);
            self.expires.insert(key.to_vec(), expiry);
            true
        } else {
            false
        }
    }

    /// Set expiry on a key (milliseconds)
    pub fn pexpire(&mut self, key: &[u8], ms: u64) -> bool {
        self.check_expired(key);
        if self.keyspace.contains_key(key) {
            let expiry = Instant::now() + std::time::Duration::from_millis(ms);
            self.expires.insert(key.to_vec(), expiry);
            true
        } else {
            false
        }
    }

    /// Get TTL in seconds (-1 if no expiry, -2 if key doesn't exist)
    pub fn ttl(&mut self, key: &[u8]) -> i64 {
        self.check_expired(key);
        if !self.keyspace.contains_key(key) {
            return -2;
        }
        match self.expires.get(key) {
            Some(&expiry) => {
                let now = Instant::now();
                if expiry > now {
                    (expiry - now).as_secs() as i64
                } else {
                    -2
                }
            }
            None => -1,
        }
    }

    /// Get TTL in milliseconds
    pub fn pttl(&mut self, key: &[u8]) -> i64 {
        self.check_expired(key);
        if !self.keyspace.contains_key(key) {
            return -2;
        }
        match self.expires.get(key) {
            Some(&expiry) => {
                let now = Instant::now();
                if expiry > now {
                    (expiry - now).as_millis() as i64
                } else {
                    -2
                }
            }
            None => -1,
        }
    }

    /// Remove expiry from a key
    pub fn persist(&mut self, key: &[u8]) -> bool {
        self.expires.remove(key).is_some()
    }

    /// Get all keys matching a pattern (simplified: only supports "*")
    pub fn keys(&mut self, pattern: &str) -> Vec<Vec<u8>> {
        // Active expire some keys first
        self.active_expire_cycle(20);

        if pattern == "*" {
            self.keyspace.keys().cloned().collect()
        } else {
            // Simple glob matching
            self.keyspace
                .keys()
                .filter(|k| {
                    let key_str = String::from_utf8_lossy(k);
                    glob_match(pattern, &key_str)
                })
                .cloned()
                .collect()
        }
    }

    /// Rename a key
    pub fn rename(&mut self, old_key: &[u8], new_key: Vec<u8>) -> bool {
        self.check_expired(old_key);
        if let Some(value) = self.keyspace.remove(old_key) {
            let expiry = self.expires.remove(old_key);
            self.keyspace.insert(new_key.clone(), value);
            if let Some(exp) = expiry {
                self.expires.insert(new_key, exp);
            }
            true
        } else {
            false
        }
    }

    /// Get the number of keys
    pub fn dbsize(&mut self) -> usize {
        self.active_expire_cycle(20);
        self.keyspace.len()
    }

    /// Flush all keys
    pub fn flushdb(&mut self) {
        self.keyspace.clear();
        self.expires.clear();
    }

    /// Periodic active expiration: randomly sample and delete expired keys
    pub fn active_expire_cycle(&mut self, samples: usize) {
        use rand::seq::IteratorRandom;
        let expired_keys: Vec<Vec<u8>> = self
            .expires
            .iter()
            .choose_multiple(&mut rand::thread_rng(), samples)
            .into_iter()
            .filter(|(_, &expiry)| Instant::now() >= expiry)
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            self.keyspace.remove(&key);
            self.expires.remove(&key);
        }
    }

    /// Get or create a value for a key, returning mutable reference
    pub fn get_or_insert(&mut self, key: Vec<u8>, default: RedisObject) -> &mut RedisObject {
        self.check_expired(&key);
        self.keyspace.entry(key).or_insert(default)
    }
}

/// Simple glob pattern matching (supports * and ?)
fn glob_match(pattern: &str, s: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let s: Vec<char> = s.chars().collect();
    let mut dp = vec![vec![false; s.len() + 1]; p.len() + 1];
    dp[0][0] = true;

    for i in 1..=p.len() {
        if p[i - 1] == '*' {
            dp[i][0] = dp[i - 1][0];
        }
    }

    for i in 1..=p.len() {
        for j in 1..=s.len() {
            if p[i - 1] == '*' {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if p[i - 1] == '?' || p[i - 1] == s[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }

    dp[p.len()][s.len()]
}
