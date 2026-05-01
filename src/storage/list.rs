use std::collections::VecDeque;

/// Redis List data structure using VecDeque for O(1) push/pop on both ends
#[derive(Debug, Clone)]
pub struct RedisList {
    data: VecDeque<Vec<u8>>,
}

impl RedisList {
    pub fn new() -> Self {
        RedisList {
            data: VecDeque::new(),
        }
    }

    pub fn lpush(&mut self, values: Vec<Vec<u8>>) -> usize {
        for v in values {
            self.data.push_front(v);
        }
        self.data.len()
    }

    pub fn rpush(&mut self, values: Vec<Vec<u8>>) -> usize {
        for v in values {
            self.data.push_back(v);
        }
        self.data.len()
    }

    pub fn lpop(&mut self) -> Option<Vec<u8>> {
        self.data.pop_front()
    }

    pub fn rpop(&mut self) -> Option<Vec<u8>> {
        self.data.pop_back()
    }

    pub fn llen(&self) -> usize {
        self.data.len()
    }

    pub fn lrange(&self, start: i64, stop: i64) -> Vec<Vec<u8>> {
        let len = self.data.len() as i64;
        if len == 0 {
            return Vec::new();
        }

        let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
        let stop = if stop < 0 { (len + stop).max(0) } else { stop.min(len - 1) } as usize;

        if start > stop {
            return Vec::new();
        }
        self.data.iter().skip(start).take(stop - start + 1).cloned().collect()
    }

    pub fn lindex(&self, index: i64) -> Option<&Vec<u8>> {
        let len = self.data.len() as i64;
        let idx = if index < 0 { len + index } else { index };
        if idx < 0 || idx >= len {
            return None;
        }
        self.data.get(idx as usize)
    }

    pub fn lset(&mut self, index: i64, value: Vec<u8>) -> bool {
        let len = self.data.len() as i64;
        let idx = if index < 0 { len + index } else { index };
        if idx < 0 || idx >= len {
            return false;
        }
        self.data[idx as usize] = value;
        true
    }

    pub fn lrem(&mut self, count: i64, value: &[u8]) -> i64 {
        let mut removed = 0i64;
        if count > 0 {
            // Remove first N occurrences from head
            let mut i = 0;
            while i < self.data.len() && removed < count {
                if self.data[i] == value {
                    self.data.remove(i);
                    removed += 1;
                } else {
                    i += 1;
                }
            }
        } else if count < 0 {
            // Remove first N occurrences from tail
            let target = -count;
            let mut i = self.data.len();
            while i > 0 && removed < target {
                i -= 1;
                if self.data[i] == value {
                    self.data.remove(i);
                    removed += 1;
                }
            }
        } else {
            // Remove all occurrences
            let len_before = self.data.len();
            self.data.retain(|v| v.as_slice() != value);
            removed = (len_before - self.data.len()) as i64;
        }
        removed
    }

    pub fn ltrim(&mut self, start: i64, stop: i64) {
        let len = self.data.len() as i64;
        let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
        let stop = if stop < 0 { (len + stop).max(0) } else { stop.min(len - 1) } as usize;

        if start > stop || start >= self.data.len() {
            self.data.clear();
            return;
        }

        let new_data: VecDeque<Vec<u8>> = self.data.iter().skip(start).take(stop - start + 1).cloned().collect();
        self.data = new_data;
    }

    pub fn linsert(&mut self, before: bool, pivot: &[u8], value: Vec<u8>) -> i64 {
        if let Some(pos) = self.data.iter().position(|v| v.as_slice() == pivot) {
            let insert_pos = if before { pos } else { pos + 1 };
            self.data.insert(insert_pos, value);
            self.data.len() as i64
        } else {
            -1
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
