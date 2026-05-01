use crate::protocol::RespValue;
use crate::storage::db::Database;
use crate::storage::object::RedisObject;
use crate::storage::string::RedisString;
use super::{parse_integer, parse_float};

pub fn cmd_get(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'get' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::String(s)) => RespValue::bulk(s.as_bytes()),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::null(),
    }
}

pub fn cmd_set(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() {
        return RespValue::error("ERR wrong number of arguments for 'set' command");
    }
    if args.len() < 2 {
        return RespValue::error("ERR wrong number of arguments for 'set' command");
    }

    let key = args[0].clone();
    let value = args[1].clone();
    let mut nx = false;
    let mut xx = false;
    let mut ex: Option<u64> = None;
    let mut px: Option<u64> = None;

    let mut i = 2;
    while i < args.len() {
        let opt = String::from_utf8_lossy(&args[i]).to_uppercase();
        match opt.as_str() {
            "NX" => { nx = true; i += 1; }
            "XX" => { xx = true; i += 1; }
            "EX" => {
                if i + 1 >= args.len() {
                    return RespValue::error("ERR syntax error");
                }
                ex = parse_integer(&args[i + 1]).map(|n| n as u64);
                i += 2;
            }
            "PX" => {
                if i + 1 >= args.len() {
                    return RespValue::error("ERR syntax error");
                }
                px = parse_integer(&args[i + 1]).map(|n| n as u64);
                i += 2;
            }
            _ => return RespValue::error("ERR syntax error"),
        }
    }

    if nx && db.exists(&key) {
        return RespValue::null();
    }
    if xx && !db.exists(&key) {
        return RespValue::null();
    }

    let obj = RedisObject::String(RedisString::new(value));
    if let Some(ms) = px {
        db.set_with_expiry(key, obj, ms);
    } else if let Some(secs) = ex {
        db.set_with_expiry(key, obj, secs * 1000);
    } else {
        db.set(key, obj);
    }
    RespValue::ok()
}

pub fn cmd_setnx(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'setnx' command");
    }
    if db.exists(&args[0]) {
        return RespValue::integer(0);
    }
    db.set(args[0].clone(), RedisObject::String(RedisString::new(args[1].clone())));
    RespValue::integer(1)
}

pub fn cmd_setex(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'setex' command");
    }
    let seconds = match parse_integer(&args[1]) {
        Some(n) if n > 0 => n as u64,
        _ => return RespValue::error("ERR invalid expire time in 'setex' command"),
    };
    let obj = RedisObject::String(RedisString::new(args[2].clone()));
    db.set_with_expiry(args[0].clone(), obj, seconds * 1000);
    RespValue::ok()
}

pub fn cmd_psetex(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'psetex' command");
    }
    let ms = match parse_integer(&args[1]) {
        Some(n) if n > 0 => n as u64,
        _ => return RespValue::error("ERR invalid expire time in 'psetex' command"),
    };
    let obj = RedisObject::String(RedisString::new(args[2].clone()));
    db.set_with_expiry(args[0].clone(), obj, ms);
    RespValue::ok()
}

pub fn cmd_mget(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() {
        return RespValue::error("ERR wrong number of arguments for 'mget' command");
    }
    let values: Vec<RespValue> = args
        .iter()
        .map(|key| match db.get(key) {
            Some(RedisObject::String(s)) => RespValue::bulk(s.as_bytes()),
            _ => RespValue::null(),
        })
        .collect();
    RespValue::array(values)
}

pub fn cmd_mset(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() || args.len() % 2 != 0 {
        return RespValue::error("ERR wrong number of arguments for 'mset' command");
    }
    for chunk in args.chunks(2) {
        db.set(
            chunk[0].clone(),
            RedisObject::String(RedisString::new(chunk[1].clone())),
        );
    }
    RespValue::ok()
}

pub fn cmd_incr(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'incr' command");
    }
    incr_by_helper(db, &args[0], 1)
}

pub fn cmd_incrby(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'incrby' command");
    }
    let delta = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    incr_by_helper(db, &args[0], delta)
}

pub fn cmd_incrbyfloat(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'incrbyfloat' command");
    }
    let delta = match parse_float(&args[1]) {
        Some(f) => f,
        None => return RespValue::error("ERR value is not a valid float"),
    };
    let key = &args[0];
    let obj = db.get_or_insert(
        key.clone(),
        RedisObject::String(RedisString::new(b"0".to_vec())),
    );
    match obj {
        RedisObject::String(s) => match s.incr_by_float(delta) {
            Ok(new_val) => RespValue::bulk(format!("{}", new_val).into_bytes()),
            Err(e) => RespValue::error(e),
        },
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_decr(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'decr' command");
    }
    incr_by_helper(db, &args[0], -1)
}

pub fn cmd_decrby(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'decrby' command");
    }
    let delta = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    incr_by_helper(db, &args[0], -delta)
}

fn incr_by_helper(db: &mut Database, key: &[u8], delta: i64) -> RespValue {
    let obj = db.get_or_insert(
        key.to_vec(),
        RedisObject::String(RedisString::new(b"0".to_vec())),
    );
    match obj {
        RedisObject::String(s) => match s.incr_by(delta) {
            Ok(new_val) => RespValue::integer(new_val),
            Err(e) => RespValue::error(e),
        },
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_append(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'append' command");
    }
    let obj = db.get_or_insert(
        args[0].clone(),
        RedisObject::String(RedisString::new(Vec::new())),
    );
    match obj {
        RedisObject::String(s) => {
            let new_len = s.append(&args[1]);
            RespValue::integer(new_len as i64)
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_strlen(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'strlen' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::String(s)) => RespValue::integer(s.len() as i64),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_getrange(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'getrange' command");
    }
    let start = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    let end = match parse_integer(&args[2]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    match db.get(&args[0]) {
        Some(RedisObject::String(s)) => RespValue::bulk(s.getrange(start, end)),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::bulk(Vec::new()),
    }
}

pub fn cmd_setrange(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'setrange' command");
    }
    let offset = match parse_integer(&args[1]) {
        Some(n) if n >= 0 => n as usize,
        _ => return RespValue::error("ERR offset is out of range"),
    };
    let obj = db.get_or_insert(
        args[0].clone(),
        RedisObject::String(RedisString::new(Vec::new())),
    );
    match obj {
        RedisObject::String(s) => {
            let new_len = s.setrange(offset, &args[2]);
            RespValue::integer(new_len as i64)
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}
