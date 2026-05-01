use std::collections::HashSet;
use rand::seq::IteratorRandom;

const SET_MAX_INTSET_ENTRIES: usize = 512;

/// Redis Set data structure with intset + hashtable encoding
#[derive(Debug, Clone)]
pub enum RedisSet {
    /// Small set of integers only
    IntSet(Vec<i64>),
    /// General set
    HashSet(HashSet<Vec<u8>>),
}

impl RedisSet {
    pub fn new() -> Self {
        RedisSet::IntSet(Vec::new())
    }

    fn maybe_convert(&mut self) {
        // IntSet stays as long as all elements are integers and count <= threshold
        if let RedisSet::IntSet(ints) = self {
            if ints.len() > SET_MAX_INTSET_ENTRIES {
                let set: HashSet<Vec<u8>> = ints.iter().map(|n| n.to_string().into_bytes()).collect();
                *self = RedisSet::HashSet(set);
            }
        }
    }

    fn try_as_int(data: &[u8]) -> Option<i64> {
        std::str::from_utf8(data).ok().and_then(|s| s.parse().ok())
    }

    pub fn sadd(&mut self, members: Vec<Vec<u8>>) -> usize {
        let mut added = 0;
        for member in members {
            match self {
                RedisSet::IntSet(ints) => {
                    if let Some(n) = Self::try_as_int(&member) {
                        if !ints.contains(&n) {
                            ints.push(n);
                            ints.sort();
                            added += 1;
                        }
                    } else {
                        // Convert to HashSet
                        let mut set: HashSet<Vec<u8>> =
                            ints.iter().map(|n| n.to_string().into_bytes()).collect();
                        if set.insert(member) {
                            added += 1;
                        }
                        *self = RedisSet::HashSet(set);
                    }
                }
                RedisSet::HashSet(set) => {
                    if set.insert(member) {
                        added += 1;
                    }
                }
            }
        }
        self.maybe_convert();
        added
    }

    pub fn srem(&mut self, members: &[Vec<u8>]) -> usize {
        let mut removed = 0;
        for member in members {
            match self {
                RedisSet::IntSet(ints) => {
                    if let Some(n) = Self::try_as_int(member) {
                        if let Some(pos) = ints.iter().position(|x| *x == n) {
                            ints.remove(pos);
                            removed += 1;
                        }
                    }
                }
                RedisSet::HashSet(set) => {
                    if set.remove(member) {
                        removed += 1;
                    }
                }
            }
        }
        removed
    }

    pub fn sismember(&self, member: &[u8]) -> bool {
        match self {
            RedisSet::IntSet(ints) => {
                Self::try_as_int(member).map_or(false, |n| ints.contains(&n))
            }
            RedisSet::HashSet(set) => set.contains(member),
        }
    }

    pub fn smembers(&self) -> Vec<Vec<u8>> {
        match self {
            RedisSet::IntSet(ints) => ints.iter().map(|n| n.to_string().into_bytes()).collect(),
            RedisSet::HashSet(set) => set.iter().cloned().collect(),
        }
    }

    pub fn scard(&self) -> usize {
        match self {
            RedisSet::IntSet(ints) => ints.len(),
            RedisSet::HashSet(set) => set.len(),
        }
    }

    pub fn spop(&mut self) -> Option<Vec<u8>> {
        match self {
            RedisSet::IntSet(ints) => {
                if ints.is_empty() {
                    None
                } else {
                    let idx = {
                        let mut rng = rand::thread_rng();
                        (0..ints.len()).choose(&mut rng).unwrap_or(0)
                    };
                    let val = ints.remove(idx);
                    Some(val.to_string().into_bytes())
                }
            }
            RedisSet::HashSet(set) => {
                let member = {
                    let mut rng = rand::thread_rng();
                    set.iter().choose(&mut rng).cloned()
                };
                if let Some(ref m) = member {
                    set.remove(m);
                }
                member
            }
        }
    }

    pub fn srandmember(&self, count: i64) -> Vec<Vec<u8>> {
        let members = self.smembers();
        if members.is_empty() {
            return Vec::new();
        }

        let mut rng = rand::thread_rng();
        if count > 0 {
            let count = (count as usize).min(members.len());
            members.into_iter().choose_multiple(&mut rng, count)
        } else {
            let count = (-count) as usize;
            (0..count)
                .filter_map(|_| members.iter().choose(&mut rng).cloned())
                .collect()
        }
    }

    /// Get members as a HashSet<Vec<u8>> for set operations
    fn to_hashset(&self) -> HashSet<Vec<u8>> {
        match self {
            RedisSet::IntSet(ints) => ints.iter().map(|n| n.to_string().into_bytes()).collect(),
            RedisSet::HashSet(set) => set.clone(),
        }
    }

    pub fn sunion(sets: &[&RedisSet]) -> Vec<Vec<u8>> {
        let mut result = HashSet::new();
        for s in sets {
            for member in s.smembers() {
                result.insert(member);
            }
        }
        result.into_iter().collect()
    }

    pub fn sinter(sets: &[&RedisSet]) -> Vec<Vec<u8>> {
        if sets.is_empty() {
            return Vec::new();
        }
        let mut result = sets[0].to_hashset();
        for s in &sets[1..] {
            let other = s.to_hashset();
            result = result.intersection(&other).cloned().collect();
        }
        result.into_iter().collect()
    }

    pub fn sdiff(sets: &[&RedisSet]) -> Vec<Vec<u8>> {
        if sets.is_empty() {
            return Vec::new();
        }
        let mut result = sets[0].to_hashset();
        for s in &sets[1..] {
            let other = s.to_hashset();
            result = result.difference(&other).cloned().collect();
        }
        result.into_iter().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.scard() == 0
    }
}
