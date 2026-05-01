use std::collections::HashMap;
use rand::Rng;

const SKIPLIST_MAXLEVEL: usize = 32;
const SKIPLIST_P: f64 = 0.25;

/// A node in the skip list
#[derive(Debug, Clone)]
struct SkipListNode {
    member: Vec<u8>,
    score: f64,
    /// Forward pointers for each level, with span
    levels: Vec<SkipListLevel>,
}

#[derive(Debug, Clone)]
struct SkipListLevel {
    /// Index of the next node in the nodes array, or usize::MAX for None
    forward: usize,
    /// Span (number of nodes between this node and the forward node)
    span: usize,
}

/// Skip list sorted by (score, member)
#[derive(Debug, Clone)]
struct SkipList {
    /// All nodes; index 0 is the header sentinel
    nodes: Vec<SkipListNode>,
    /// Current max level in use
    level: usize,
    /// Number of elements (excluding header)
    length: usize,
    /// Tail node index
    tail: usize,
    /// Free list for removed nodes
    free_list: Vec<usize>,
}

const NONE: usize = usize::MAX;

impl SkipList {
    fn new() -> Self {
        let header = SkipListNode {
            member: Vec::new(),
            score: 0.0,
            levels: (0..SKIPLIST_MAXLEVEL)
                .map(|_| SkipListLevel { forward: NONE, span: 0 })
                .collect(),
        };
        SkipList {
            nodes: vec![header],
            level: 1,
            length: 0,
            tail: NONE,
            free_list: Vec::new(),
        }
    }

    fn random_level() -> usize {
        let mut level = 1;
        let mut rng = rand::thread_rng();
        while rng.gen::<f64>() < SKIPLIST_P && level < SKIPLIST_MAXLEVEL {
            level += 1;
        }
        level
    }

    fn alloc_node(&mut self, member: Vec<u8>, score: f64, level: usize) -> usize {
        let node = SkipListNode {
            member,
            score,
            levels: (0..level)
                .map(|_| SkipListLevel { forward: NONE, span: 0 })
                .collect(),
        };
        if let Some(idx) = self.free_list.pop() {
            self.nodes[idx] = node;
            idx
        } else {
            self.nodes.push(node);
            self.nodes.len() - 1
        }
    }

    fn insert(&mut self, score: f64, member: Vec<u8>) {
        let mut update = [0usize; SKIPLIST_MAXLEVEL];
        let mut rank = [0usize; SKIPLIST_MAXLEVEL];
        let mut x = 0; // header index

        for i in (0..self.level).rev() {
            rank[i] = if i == self.level - 1 { 0 } else { rank[i + 1] };
            loop {
                let fwd = self.nodes[x].levels[i].forward;
                if fwd == NONE {
                    break;
                }
                let fwd_node = &self.nodes[fwd];
                if fwd_node.score < score
                    || (fwd_node.score == score && fwd_node.member < member)
                {
                    rank[i] += self.nodes[x].levels[i].span;
                    x = fwd;
                } else {
                    break;
                }
            }
            update[i] = x;
        }

        let new_level = Self::random_level();
        if new_level > self.level {
            for i in self.level..new_level {
                rank[i] = 0;
                update[i] = 0; // header
                self.nodes[0].levels[i].span = self.length;
            }
            self.level = new_level;
        }

        let new_idx = self.alloc_node(member, score, new_level);

        for i in 0..new_level {
            let prev = update[i];
            self.nodes[new_idx].levels[i].forward = self.nodes[prev].levels[i].forward;
            self.nodes[prev].levels[i].forward = new_idx;

            let old_span = self.nodes[prev].levels[i].span;
            let rank_diff = rank[0] - rank[i];
            self.nodes[new_idx].levels[i].span = old_span - rank_diff;
            self.nodes[prev].levels[i].span = rank_diff + 1;
        }

        // Increment span for untouched levels
        for i in new_level..self.level {
            self.nodes[update[i]].levels[i].span += 1;
        }

        // Update tail
        if self.nodes[new_idx].levels[0].forward == NONE {
            self.tail = new_idx;
        }

        self.length += 1;
    }

    fn delete(&mut self, score: f64, member: &[u8]) -> bool {
        let mut update = [0usize; SKIPLIST_MAXLEVEL];
        let mut x = 0;

        for i in (0..self.level).rev() {
            loop {
                let fwd = self.nodes[x].levels[i].forward;
                if fwd == NONE {
                    break;
                }
                let fwd_node = &self.nodes[fwd];
                if fwd_node.score < score
                    || (fwd_node.score == score && fwd_node.member.as_slice() < member)
                {
                    x = fwd;
                } else {
                    break;
                }
            }
            update[i] = x;
        }

        let target = self.nodes[x].levels[0].forward;
        if target == NONE {
            return false;
        }
        if self.nodes[target].score != score || self.nodes[target].member != member {
            return false;
        }

        // Remove target from all levels
        let target_levels = self.nodes[target].levels.len();
        for i in 0..self.level {
            if i < target_levels && self.nodes[update[i]].levels[i].forward == target {
                self.nodes[update[i]].levels[i].span += self.nodes[target].levels[i].span - 1;
                self.nodes[update[i]].levels[i].forward = self.nodes[target].levels[i].forward;
            } else {
                if i < self.nodes[update[i]].levels.len() {
                    self.nodes[update[i]].levels[i].span -= 1;
                }
            }
        }

        // Update tail
        if self.tail == target {
            if self.nodes[target].levels[0].forward == NONE {
                // Find new tail
                if update[0] == 0 {
                    self.tail = NONE;
                } else {
                    self.tail = update[0];
                }
            }
        }

        self.free_list.push(target);
        self.length -= 1;

        // Reduce level
        while self.level > 1 && self.nodes[0].levels[self.level - 1].forward == NONE {
            self.level -= 1;
        }

        true
    }

    /// Get 0-based rank of member with given score
    fn get_rank(&self, score: f64, member: &[u8]) -> Option<usize> {
        let mut rank = 0usize;
        let mut x = 0;

        for i in (0..self.level).rev() {
            loop {
                let fwd = self.nodes[x].levels[i].forward;
                if fwd == NONE {
                    break;
                }
                let fwd_node = &self.nodes[fwd];
                if fwd_node.score < score
                    || (fwd_node.score == score && fwd_node.member.as_slice() <= member)
                {
                    rank += self.nodes[x].levels[i].span;
                    x = fwd;
                    if fwd_node.score == score && fwd_node.member == member {
                        return Some(rank - 1);
                    }
                } else {
                    break;
                }
            }
        }
        None
    }

    /// Get elements by rank range [start, end] (0-based, inclusive)
    fn get_range_by_rank(&self, start: usize, end: usize) -> Vec<(f64, Vec<u8>)> {
        let mut result = Vec::new();
        let mut x = 0;
        let mut traversed = 0usize;

        // Find the start node
        for i in (0..self.level).rev() {
            loop {
                let fwd = self.nodes[x].levels[i].forward;
                if fwd == NONE {
                    break;
                }
                if traversed + self.nodes[x].levels[i].span <= start {
                    traversed += self.nodes[x].levels[i].span;
                    x = fwd;
                } else {
                    break;
                }
            }
        }

        // Now x is at position `traversed`, move to start via level 0
        let mut x = self.nodes[x].levels[0].forward;
        let mut rank = traversed;

        while x != NONE && rank <= end {
            result.push((self.nodes[x].score, self.nodes[x].member.clone()));
            x = self.nodes[x].levels[0].forward;
            rank += 1;
        }
        result
    }

    /// Get elements by score range
    fn get_range_by_score(&self, min: f64, max: f64) -> Vec<(f64, Vec<u8>)> {
        let mut result = Vec::new();
        let mut x = 0;

        // Find first node with score >= min
        for i in (0..self.level).rev() {
            loop {
                let fwd = self.nodes[x].levels[i].forward;
                if fwd == NONE {
                    break;
                }
                if self.nodes[fwd].score < min {
                    x = fwd;
                } else {
                    break;
                }
            }
        }

        x = self.nodes[x].levels[0].forward;
        while x != NONE && self.nodes[x].score <= max {
            result.push((self.nodes[x].score, self.nodes[x].member.clone()));
            x = self.nodes[x].levels[0].forward;
        }
        result
    }

    fn count_in_score_range(&self, min: f64, max: f64) -> usize {
        self.get_range_by_score(min, max).len()
    }
}

/// Redis Sorted Set: SkipList + HashMap dual index
#[derive(Debug, Clone)]
pub struct RedisZSet {
    skiplist: SkipList,
    dict: HashMap<Vec<u8>, f64>,
}

impl RedisZSet {
    pub fn new() -> Self {
        RedisZSet {
            skiplist: SkipList::new(),
            dict: HashMap::new(),
        }
    }

    pub fn zadd(&mut self, score: f64, member: Vec<u8>) -> bool {
        if let Some(&old_score) = self.dict.get(&member) {
            if (old_score - score).abs() > f64::EPSILON || old_score != score {
                self.skiplist.delete(old_score, &member);
                self.skiplist.insert(score, member.clone());
                self.dict.insert(member, score);
            }
            false // not new
        } else {
            self.skiplist.insert(score, member.clone());
            self.dict.insert(member, score);
            true // new
        }
    }

    pub fn zrem(&mut self, member: &[u8]) -> bool {
        if let Some(score) = self.dict.remove(member) {
            self.skiplist.delete(score, member);
            true
        } else {
            false
        }
    }

    pub fn zscore(&self, member: &[u8]) -> Option<f64> {
        self.dict.get(member).copied()
    }

    pub fn zrank(&self, member: &[u8]) -> Option<usize> {
        let score = self.dict.get(member)?;
        self.skiplist.get_rank(*score, member)
    }

    pub fn zrevrank(&self, member: &[u8]) -> Option<usize> {
        self.zrank(member).map(|r| self.skiplist.length - 1 - r)
    }

    pub fn zrange(&self, start: i64, stop: i64) -> Vec<(f64, Vec<u8>)> {
        let len = self.skiplist.length as i64;
        if len == 0 {
            return Vec::new();
        }
        let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
        let stop = if stop < 0 { (len + stop).max(0) } else { stop.min(len - 1) } as usize;
        if start > stop {
            return Vec::new();
        }
        self.skiplist.get_range_by_rank(start, stop)
    }

    pub fn zrevrange(&self, start: i64, stop: i64) -> Vec<(f64, Vec<u8>)> {
        let len = self.skiplist.length as i64;
        if len == 0 {
            return Vec::new();
        }
        let start_idx = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
        let stop_idx = if stop < 0 { (len + stop).max(0) } else { stop.min(len - 1) } as usize;
        if start_idx > stop_idx {
            return Vec::new();
        }
        // Get all then reverse
        let real_start = self.skiplist.length - 1 - stop_idx;
        let real_stop = self.skiplist.length - 1 - start_idx;
        let mut result = self.skiplist.get_range_by_rank(real_start, real_stop);
        result.reverse();
        result
    }

    pub fn zrangebyscore(&self, min: f64, max: f64) -> Vec<(f64, Vec<u8>)> {
        self.skiplist.get_range_by_score(min, max)
    }

    pub fn zcard(&self) -> usize {
        self.skiplist.length
    }

    pub fn zcount(&self, min: f64, max: f64) -> usize {
        self.skiplist.count_in_score_range(min, max)
    }

    pub fn zincrby(&mut self, member: Vec<u8>, delta: f64) -> f64 {
        let new_score = self.dict.get(&member).copied().unwrap_or(0.0) + delta;
        if let Some(&old_score) = self.dict.get(&member) {
            self.skiplist.delete(old_score, &member);
        }
        self.skiplist.insert(new_score, member.clone());
        self.dict.insert(member, new_score);
        new_score
    }

    pub fn is_empty(&self) -> bool {
        self.skiplist.length == 0
    }
}
