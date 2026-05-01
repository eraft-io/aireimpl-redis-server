use crate::protocol::RespValue;
use crate::storage::db::Database;
use crate::storage::list::RedisList;
use crate::storage::object::RedisObject;
use super::parse_integer;

pub fn cmd_lpush(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 2 {
        return RespValue::error("ERR wrong number of arguments for 'lpush' command");
    }
    let obj = db.get_or_insert(args[0].clone(), RedisObject::List(RedisList::new()));
    match obj {
        RedisObject::List(l) => {
            let new_len = l.lpush(args[1..].to_vec());
            RespValue::integer(new_len as i64)
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_rpush(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 2 {
        return RespValue::error("ERR wrong number of arguments for 'rpush' command");
    }
    let obj = db.get_or_insert(args[0].clone(), RedisObject::List(RedisList::new()));
    match obj {
        RedisObject::List(l) => {
            let new_len = l.rpush(args[1..].to_vec());
            RespValue::integer(new_len as i64)
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_lpop(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'lpop' command");
    }
    match db.get_mut(&args[0]) {
        Some(RedisObject::List(l)) => match l.lpop() {
            Some(v) => {
                let result = RespValue::bulk(v);
                // Don't clean up empty list here, just return
                result
            }
            None => RespValue::null(),
        },
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::null(),
    }
}

pub fn cmd_rpop(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'rpop' command");
    }
    match db.get_mut(&args[0]) {
        Some(RedisObject::List(l)) => match l.rpop() {
            Some(v) => RespValue::bulk(v),
            None => RespValue::null(),
        },
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::null(),
    }
}

pub fn cmd_llen(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'llen' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::List(l)) => RespValue::integer(l.llen() as i64),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_lrange(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'lrange' command");
    }
    let start = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    let stop = match parse_integer(&args[2]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    match db.get(&args[0]) {
        Some(RedisObject::List(l)) => {
            let items = l.lrange(start, stop);
            RespValue::array(items.into_iter().map(RespValue::bulk).collect())
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::array(Vec::new()),
    }
}

pub fn cmd_lindex(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'lindex' command");
    }
    let index = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    match db.get(&args[0]) {
        Some(RedisObject::List(l)) => match l.lindex(index) {
            Some(v) => RespValue::bulk(v.clone()),
            None => RespValue::null(),
        },
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::null(),
    }
}

pub fn cmd_lset(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'lset' command");
    }
    let index = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    match db.get_mut(&args[0]) {
        Some(RedisObject::List(l)) => {
            if l.lset(index, args[2].clone()) {
                RespValue::ok()
            } else {
                RespValue::error("ERR index out of range")
            }
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::error("ERR no such key"),
    }
}

pub fn cmd_lrem(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'lrem' command");
    }
    let count = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    match db.get_mut(&args[0]) {
        Some(RedisObject::List(l)) => {
            let removed = l.lrem(count, &args[2]);
            RespValue::integer(removed)
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_ltrim(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 3 {
        return RespValue::error("ERR wrong number of arguments for 'ltrim' command");
    }
    let start = match parse_integer(&args[1]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    let stop = match parse_integer(&args[2]) {
        Some(n) => n,
        None => return RespValue::error("ERR value is not an integer or out of range"),
    };
    match db.get_mut(&args[0]) {
        Some(RedisObject::List(l)) => {
            l.ltrim(start, stop);
            RespValue::ok()
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::ok(),
    }
}

pub fn cmd_linsert(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 4 {
        return RespValue::error("ERR wrong number of arguments for 'linsert' command");
    }
    let before = match String::from_utf8_lossy(&args[1]).to_uppercase().as_str() {
        "BEFORE" => true,
        "AFTER" => false,
        _ => return RespValue::error("ERR syntax error"),
    };
    match db.get_mut(&args[0]) {
        Some(RedisObject::List(l)) => {
            let result = l.linsert(before, &args[2], args[3].clone());
            RespValue::integer(result)
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}
