/// Redis String data structure with integer encoding optimization
#[derive(Debug, Clone)]
pub enum RedisString {
    /// Raw bytes (binary safe)
    Raw(Vec<u8>),
    /// Integer encoding optimization
    Integer(i64),
}

impl RedisString {
    pub fn new(data: Vec<u8>) -> Self {
        // Try integer encoding
        if let Ok(s) = std::str::from_utf8(&data) {
            if let Ok(n) = s.parse::<i64>() {
                return RedisString::Integer(n);
            }
        }
        RedisString::Raw(data)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            RedisString::Raw(data) => data.clone(),
            RedisString::Integer(n) => n.to_string().into_bytes(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            RedisString::Raw(data) => data.len(),
            RedisString::Integer(n) => n.to_string().len(),
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            RedisString::Integer(n) => Some(*n),
            RedisString::Raw(data) => {
                std::str::from_utf8(data).ok().and_then(|s| s.parse().ok())
            }
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            RedisString::Integer(n) => Some(*n as f64),
            RedisString::Raw(data) => {
                std::str::from_utf8(data).ok().and_then(|s| s.parse().ok())
            }
        }
    }

    pub fn incr_by(&mut self, delta: i64) -> Result<i64, String> {
        let current = self
            .as_integer()
            .ok_or_else(|| "ERR value is not an integer or out of range".to_string())?;
        let new_val = current
            .checked_add(delta)
            .ok_or_else(|| "ERR increment or decrement would overflow".to_string())?;
        *self = RedisString::Integer(new_val);
        Ok(new_val)
    }

    pub fn incr_by_float(&mut self, delta: f64) -> Result<f64, String> {
        let current = self
            .as_float()
            .ok_or_else(|| "ERR value is not a valid float".to_string())?;
        let new_val = current + delta;
        if new_val.is_nan() || new_val.is_infinite() {
            return Err("ERR increment would produce NaN or Infinity".to_string());
        }
        *self = RedisString::Raw(format!("{}", new_val).into_bytes());
        Ok(new_val)
    }

    pub fn append(&mut self, data: &[u8]) -> usize {
        let mut current = self.as_bytes();
        current.extend_from_slice(data);
        let new_len = current.len();
        *self = RedisString::new(current);
        new_len
    }

    pub fn getrange(&self, start: i64, end: i64) -> Vec<u8> {
        let bytes = self.as_bytes();
        let len = bytes.len() as i64;
        if len == 0 {
            return Vec::new();
        }

        let start = if start < 0 {
            (len + start).max(0) as usize
        } else {
            start.min(len - 1) as usize
        };
        let end = if end < 0 {
            (len + end).max(0) as usize
        } else {
            end.min(len - 1) as usize
        };

        if start > end {
            return Vec::new();
        }
        bytes[start..=end].to_vec()
    }

    pub fn setrange(&mut self, offset: usize, data: &[u8]) -> usize {
        let mut bytes = self.as_bytes();
        let needed = offset + data.len();
        if bytes.len() < needed {
            bytes.resize(needed, 0);
        }
        bytes[offset..offset + data.len()].copy_from_slice(data);
        let new_len = bytes.len();
        *self = RedisString::new(bytes);
        new_len
    }
}
