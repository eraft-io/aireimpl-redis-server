use crate::protocol::RespValue;
use crate::storage::db::Database;
use crate::storage::zset::RedisZSet;
use crate::storage::object::RedisObject;
use super::{parse_integer, parse_float};

pub fn cmd_zadd(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 3 || (args.len() - 1) % 2 != 0 {
        return RespValue::error("ERR wrong number of arguments for 'zadd' command");
    }
    let obj = db.get_or_insert(args[0].clone(), RedisObject::ZSet(RedisZSet::new()));
    match obj {
        RedisObject::ZSet(z) => {
            let mut added = 0i64;
            for chunk in args[1..].chunks(2) {
                let score = match parse_float(&chunk[0]) {
                    Some(f) => f,
                    None => return RespValue::error("ERR value is not a valid float"),
                };
                if z.zadd(score, chunk[1].clone()) {
                    added += 1;
                }
            }
            RespValue::integer(added)
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_zrem(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 2 {
        return RespValue::error("ERR wrong number of arguments for 'zrem' command");
    }
    match db.get_mut(&args[0]) {
        Some(RedisObject::ZSet(z)) => {
            let mut removed = 0i64;
            for member in &args[1..] {
                if z.zrem(member) {
                    removed += 1;
                }
            }
            RespValue::integer(removed)
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_zscore(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'zscore' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::ZSet(z)) => match z.zscore(&args[1]) {
            Some(score) => RespValue::bulk(format!("{}", score).into_bytes()),
            None => RespValue::null(),
        },
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::null(),
    }
}

pub fn cmd_zrank(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'zrank' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::ZSet(z)) => match z.zrank(&args[1]) {
            Some(rank) => RespValue::integer(rank as i64),
            None => RespValue::null(),
        },
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::null(),
    }
}

pub fn cmd_zrevrank(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'zrevrank' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::ZSet(z)) => match z.zrevrank(&args[1]) {
            Some(rank) => RespValue::integer(rank as i64),
            None => RespValue::null(),
        },
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::null(),
    }
}

pub fn cmd_zrange(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 3 || args.len() > 4 {
        return RespValue::error("ERR wrong number of arguments for 'zrange' command");
    }
    let start = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    let stop = match parse_integer(&args[2]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    let withscores = args.len() == 4
        && String::from_utf8_lossy(&args[3]).to_uppercase() == "WITHSCORES";

    match db.get(&args[0]) {
        Some(RedisObject::ZSet(z)) => {
            let items = z.zrange(start, stop);
            if withscores {
                let mut result = Vec::with_capacity(items.len() * 2);
                for (score, member) in items {
                    result.push(RespValue::bulk(member));
                    result.push(RespValue::bulk(format!("{}", score).into_bytes()));
                }
                RespValue::array(result)
            } else {
                RespValue::array(items.into_iter().map(|(_, m)| RespValue::bulk(m)).collect())
            }
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::array(Vec::new()),
    }
}

pub fn cmd_zrevrange(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 3 || args.len() > 4 {
        return RespValue::error("ERR wrong number of arguments for 'zrevrange' command");
    }
    let start = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    let stop = match parse_integer(&args[2]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    let withscores = args.len() == 4
        && String::from_utf8_lossy(&args[3]).to_uppercase() == "WITHSCORES";

    match db.get(&args[0]) {
        Some(RedisObject::ZSet(z)) => {
            let items = z.zrevrange(start, stop);
            if withscores {
                let mut result = Vec::with_capacity(items.len() * 2);
                for (score, member) in items {
                    result.push(RespValue::bulk(member));
                    result.push(RespValue::bulk(format!("{}", score).into_bytes()));
                }
                RespValue::array(result)
            } else {
                RespValue::array(items.into_iter().map(|(_, m)| RespValue::bulk(m)).collect())
            }
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::array(Vec::new()),
    }
}

pub fn cmd_zrangebyscore(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 3 {
        return RespValue::error("ERR wrong number of arguments for 'zrangebyscore' command");
    }
    let min = parse_score_bound(&args[1], f64::NEG_INFINITY);
    let max = parse_score_bound(&args[2], f64::INFINITY);
    let (min, max) = match (min, max) {
        (Some(a), Some(b)) => (a, b),
        _ => return RespValue::error("ERR min or max is not a float"),
    };
    let withscores = args.len() > 3
        && String::from_utf8_lossy(&args[3]).to_uppercase() == "WITHSCORES";

    match db.get(&args[0]) {
        Some(RedisObject::ZSet(z)) => {
            let items = z.zrangebyscore(min, max);
            if withscores {
                let mut result = Vec::with_capacity(items.len() * 2);
                for (score, member) in items {
                    result.push(RespValue::bulk(member));
                    result.push(RespValue::bulk(format!("{}", score).into_bytes()));
                }
                RespValue::array(result)
            } else {
                RespValue::array(items.into_iter().map(|(_, m)| RespValue::bulk(m)).collect())
            }
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::array(Vec::new()),
    }
}

pub fn cmd_zcard(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'zcard' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::ZSet(z)) => RespValue::integer(z.zcard() as i64),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_zcount(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'zcount' command");
    }
    let min = parse_score_bound(&args[1], f64::NEG_INFINITY);
    let max = parse_score_bound(&args[2], f64::INFINITY);
    let (min, max) = match (min, max) {
        (Some(a), Some(b)) => (a, b),
        _ => return RespValue::error("ERR min or max is not a float"),
    };
    match db.get(&args[0]) {
        Some(RedisObject::ZSet(z)) => RespValue::integer(z.zcount(min, max) as i64),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_zincrby(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'zincrby' command");
    }
    let delta = match parse_float(&args[1]) {
        Some(f) => f,
        None => return RespValue::error("ERR value is not a valid float"),
    };
    let obj = db.get_or_insert(args[0].clone(), RedisObject::ZSet(RedisZSet::new()));
    match obj {
        RedisObject::ZSet(z) => {
            let new_score = z.zincrby(args[2].clone(), delta);
            RespValue::bulk(format!("{}", new_score).into_bytes())
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

/// Parse score bound: supports "-inf", "+inf", "inf", and numeric values
fn parse_score_bound(data: &[u8], _default: f64) -> Option<f64> {
    let s = std::str::from_utf8(data).ok()?;
    let s = s.trim();
    match s.to_lowercase().as_str() {
        "-inf" => Some(f64::NEG_INFINITY),
        "+inf" | "inf" => Some(f64::INFINITY),
        _ => {
            // Handle exclusive bounds (e.g., "(1.5")
            if s.starts_with('(') {
                s[1..].parse::<f64>().ok()
            } else {
                s.parse::<f64>().ok()
            }
        }
    }
}
