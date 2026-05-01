use crate::protocol::RespValue;
use crate::storage::db::Database;
use crate::storage::hash::RedisHash;
use crate::storage::object::RedisObject;
use super::{parse_integer, parse_float};

pub fn cmd_hset(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 3 || args.len() % 2 == 0 {
        return RespValue::error("ERR wrong number of arguments for 'hset' command");
    }
    let obj = db.get_or_insert(args[0].clone(), RedisObject::Hash(RedisHash::new()));
    match obj {
        RedisObject::Hash(h) => {
            let mut added = 0i64;
            for chunk in args[1..].chunks(2) {
                if h.hset(chunk[0].clone(), chunk[1].clone()) {
                    added += 1;
                }
            }
            RespValue::integer(added)
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_hget(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'hget' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Hash(h)) => match h.hget(&args[1]) {
            Some(v) => RespValue::bulk(v.clone()),
            None => RespValue::null(),
        },
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::null(),
    }
}

pub fn cmd_hdel(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 2 {
        return RespValue::error("ERR wrong number of arguments for 'hdel' command");
    }
    match db.get_mut(&args[0]) {
        Some(RedisObject::Hash(h)) => {
            let mut removed = 0i64;
            for field in &args[1..] {
                if h.hdel(field) {
                    removed += 1;
                }
            }
            RespValue::integer(removed)
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_hlen(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'hlen' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Hash(h)) => RespValue::integer(h.hlen() as i64),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_hgetall(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'hgetall' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Hash(h)) => {
            let pairs = h.hgetall();
            let mut result = Vec::with_capacity(pairs.len() * 2);
            for (k, v) in pairs {
                result.push(RespValue::bulk(k));
                result.push(RespValue::bulk(v));
            }
            RespValue::array(result)
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::array(Vec::new()),
    }
}

pub fn cmd_hmset(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 3 || args.len() % 2 == 0 {
        return RespValue::error("ERR wrong number of arguments for 'hmset' command");
    }
    let obj = db.get_or_insert(args[0].clone(), RedisObject::Hash(RedisHash::new()));
    match obj {
        RedisObject::Hash(h) => {
            for chunk in args[1..].chunks(2) {
                h.hset(chunk[0].clone(), chunk[1].clone());
            }
            RespValue::ok()
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_hmget(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 2 {
        return RespValue::error("ERR wrong number of arguments for 'hmget' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Hash(h)) => {
            let values: Vec<RespValue> = args[1..]
                .iter()
                .map(|field| match h.hget(field) {
                    Some(v) => RespValue::bulk(v.clone()),
                    None => RespValue::null(),
                })
                .collect();
            RespValue::array(values)
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => {
            let values: Vec<RespValue> = args[1..].iter().map(|_| RespValue::null()).collect();
            RespValue::array(values)
        }
    }
}

pub fn cmd_hexists(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'hexists' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Hash(h)) => RespValue::integer(if h.hexists(&args[1]) { 1 } else { 0 }),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_hkeys(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'hkeys' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Hash(h)) => {
            RespValue::array(h.hkeys().into_iter().map(RespValue::bulk).collect())
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::array(Vec::new()),
    }
}

pub fn cmd_hvals(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'hvals' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Hash(h)) => {
            RespValue::array(h.hvals().into_iter().map(RespValue::bulk).collect())
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::array(Vec::new()),
    }
}

pub fn cmd_hincrby(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'hincrby' command");
    }
    let delta = match parse_integer(&args[2]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    let obj = db.get_or_insert(args[0].clone(), RedisObject::Hash(RedisHash::new()));
    match obj {
        RedisObject::Hash(h) => match h.hincrby(args[1].clone(), delta) {
            Ok(n) => RespValue::integer(n),
            Err(e) => RespValue::error(e),
        },
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_hincrbyfloat(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'hincrbyfloat' command");
    }
    let delta = match parse_float(&args[2]) {
        Some(f) => f,
        None => return RespValue::error("ERR value is not a valid float"),
    };
    let obj = db.get_or_insert(args[0].clone(), RedisObject::Hash(RedisHash::new()));
    match obj {
        RedisObject::Hash(h) => match h.hincrbyfloat(args[1].clone(), delta) {
            Ok(f) => RespValue::bulk(format!("{}", f).into_bytes()),
            Err(e) => RespValue::error(e),
        },
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_hsetnx(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'hsetnx' command");
    }
    let obj = db.get_or_insert(args[0].clone(), RedisObject::Hash(RedisHash::new()));
    match obj {
        RedisObject::Hash(h) => {
            RespValue::integer(if h.hsetnx(args[1].clone(), args[2].clone()) { 1 } else { 0 })
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}
