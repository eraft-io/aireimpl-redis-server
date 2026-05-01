use crate::protocol::RespValue;
use crate::storage::db::Database;
use crate::storage::set::RedisSet;
use crate::storage::object::RedisObject;
use super::parse_integer;

pub fn cmd_sadd(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 2 {
        return RespValue::error("ERR wrong number of arguments for 'sadd' command");
    }
    let obj = db.get_or_insert(args[0].clone(), RedisObject::Set(RedisSet::new()));
    match obj {
        RedisObject::Set(s) => {
            let added = s.sadd(args[1..].to_vec());
            RespValue::integer(added as i64)
        }
        _ => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
    }
}

pub fn cmd_srem(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() < 2 {
        return RespValue::error("ERR wrong number of arguments for 'srem' command");
    }
    match db.get_mut(&args[0]) {
        Some(RedisObject::Set(s)) => {
            let removed = s.srem(&args[1..]);
            RespValue::integer(removed as i64)
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_sismember(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'sismember' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Set(s)) => RespValue::integer(if s.sismember(&args[1]) { 1 } else { 0 }),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_smembers(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'smembers' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Set(s)) => {
            RespValue::array(s.smembers().into_iter().map(RespValue::bulk).collect())
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::array(Vec::new()),
    }
}

pub fn cmd_scard(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'scard' command");
    }
    match db.get(&args[0]) {
        Some(RedisObject::Set(s)) => RespValue::integer(s.scard() as i64),
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::integer(0),
    }
}

pub fn cmd_spop(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'spop' command");
    }
    match db.get_mut(&args[0]) {
        Some(RedisObject::Set(s)) => match s.spop() {
            Some(v) => RespValue::bulk(v),
            None => RespValue::null(),
        },
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => RespValue::null(),
    }
}

pub fn cmd_srandmember(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() || args.len() > 2 {
        return RespValue::error("ERR wrong number of arguments for 'srandmember' command");
    }
    let count = if args.len() == 2 {
        match parse_integer(&args[1]) {
            Some(n) => n,
            None => return RespValue::error("ERR value is not an integer or out of range"),
        }
    } else {
        1
    };
    match db.get(&args[0]) {
        Some(RedisObject::Set(s)) => {
            if args.len() == 1 {
                // Return single element
                let members = s.srandmember(1);
                if let Some(m) = members.into_iter().next() {
                    RespValue::bulk(m)
                } else {
                    RespValue::null()
                }
            } else {
                let members = s.srandmember(count);
                RespValue::array(members.into_iter().map(RespValue::bulk).collect())
            }
        }
        Some(_) => RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
        None => {
            if args.len() == 1 {
                RespValue::null()
            } else {
                RespValue::array(Vec::new())
            }
        }
    }
}

pub fn cmd_sunion(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() {
        return RespValue::error("ERR wrong number of arguments for 'sunion' command");
    }
    let sets: Vec<RedisSet> = args
        .iter()
        .filter_map(|key| match db.get(key) {
            Some(RedisObject::Set(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    let refs: Vec<&RedisSet> = sets.iter().collect();
    let result = RedisSet::sunion(&refs);
    RespValue::array(result.into_iter().map(RespValue::bulk).collect())
}

pub fn cmd_sinter(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() {
        return RespValue::error("ERR wrong number of arguments for 'sinter' command");
    }
    // If any key doesn't exist, intersection is empty
    let mut sets = Vec::new();
    for key in args {
        match db.get(key) {
            Some(RedisObject::Set(s)) => sets.push(s.clone()),
            Some(_) => return RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => return RespValue::array(Vec::new()),
        }
    }
    let refs: Vec<&RedisSet> = sets.iter().collect();
    let result = RedisSet::sinter(&refs);
    RespValue::array(result.into_iter().map(RespValue::bulk).collect())
}

pub fn cmd_sdiff(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() {
        return RespValue::error("ERR wrong number of arguments for 'sdiff' command");
    }
    let mut sets = Vec::new();
    for key in args {
        match db.get(key) {
            Some(RedisObject::Set(s)) => sets.push(s.clone()),
            Some(_) => return RespValue::error("WRONGTYPE Operation against a key holding the wrong kind of value"),
            None => sets.push(RedisSet::new()),
        }
    }
    let refs: Vec<&RedisSet> = sets.iter().collect();
    let result = RedisSet::sdiff(&refs);
    RespValue::array(result.into_iter().map(RespValue::bulk).collect())
}
