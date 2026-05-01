use super::string::RedisString;
use super::list::RedisList;
use super::hash::RedisHash;
use super::set::RedisSet;
use super::zset::RedisZSet;

/// Unified Redis object type wrapping all data structures
#[derive(Debug, Clone)]
pub enum RedisObject {
    String(RedisString),
    List(RedisList),
    Hash(RedisHash),
    Set(RedisSet),
    ZSet(RedisZSet),
}

impl RedisObject {
    pub fn type_name(&self) -> &'static str {
        match self {
            RedisObject::String(_) => "string",
            RedisObject::List(_) => "list",
            RedisObject::Hash(_) => "hash",
            RedisObject::Set(_) => "set",
            RedisObject::ZSet(_) => "zset",
        }
    }

    pub fn as_string(&self) -> Option<&RedisString> {
        match self {
            RedisObject::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_string_mut(&mut self) -> Option<&mut RedisString> {
        match self {
            RedisObject::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&RedisList> {
        match self {
            RedisObject::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut RedisList> {
        match self {
            RedisObject::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_hash(&self) -> Option<&RedisHash> {
        match self {
            RedisObject::Hash(h) => Some(h),
            _ => None,
        }
    }

    pub fn as_hash_mut(&mut self) -> Option<&mut RedisHash> {
        match self {
            RedisObject::Hash(h) => Some(h),
            _ => None,
        }
    }

    pub fn as_set(&self) -> Option<&RedisSet> {
        match self {
            RedisObject::Set(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_set_mut(&mut self) -> Option<&mut RedisSet> {
        match self {
            RedisObject::Set(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_zset(&self) -> Option<&RedisZSet> {
        match self {
            RedisObject::ZSet(z) => Some(z),
            _ => None,
        }
    }

    pub fn as_zset_mut(&mut self) -> Option<&mut RedisZSet> {
        match self {
            RedisObject::ZSet(z) => Some(z),
            _ => None,
        }
    }
}
