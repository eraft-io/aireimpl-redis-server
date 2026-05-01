pub mod string_cmd;
pub mod hash_cmd;
pub mod list_cmd;
pub mod set_cmd;
pub mod zset_cmd;

use crate::protocol::RespValue;
use crate::storage::db::Database;

/// Type for command handler functions
pub type CommandHandler = fn(&mut Database, &[Vec<u8>]) -> RespValue;

/// Command registry and dispatcher
pub struct CommandTable {
    commands: std::collections::HashMap<String, CommandHandler>,
}

impl CommandTable {
    pub fn new() -> Self {
        let mut table = CommandTable {
            commands: std::collections::HashMap::new(),
        };
        table.register_all();
        table
    }

    fn register(&mut self, name: &str, handler: CommandHandler) {
        self.commands.insert(name.to_uppercase(), handler);
    }

    fn register_all(&mut self) {
        // String commands
        self.register("GET", string_cmd::cmd_get);
        self.register("SET", string_cmd::cmd_set);
        self.register("SETNX", string_cmd::cmd_setnx);
        self.register("SETEX", string_cmd::cmd_setex);
        self.register("PSETEX", string_cmd::cmd_psetex);
        self.register("MGET", string_cmd::cmd_mget);
        self.register("MSET", string_cmd::cmd_mset);
        self.register("INCR", string_cmd::cmd_incr);
        self.register("INCRBY", string_cmd::cmd_incrby);
        self.register("INCRBYFLOAT", string_cmd::cmd_incrbyfloat);
        self.register("DECR", string_cmd::cmd_decr);
        self.register("DECRBY", string_cmd::cmd_decrby);
        self.register("APPEND", string_cmd::cmd_append);
        self.register("STRLEN", string_cmd::cmd_strlen);
        self.register("GETRANGE", string_cmd::cmd_getrange);
        self.register("SETRANGE", string_cmd::cmd_setrange);

        // Hash commands
        self.register("HSET", hash_cmd::cmd_hset);
        self.register("HGET", hash_cmd::cmd_hget);
        self.register("HDEL", hash_cmd::cmd_hdel);
        self.register("HLEN", hash_cmd::cmd_hlen);
        self.register("HGETALL", hash_cmd::cmd_hgetall);
        self.register("HMSET", hash_cmd::cmd_hmset);
        self.register("HMGET", hash_cmd::cmd_hmget);
        self.register("HEXISTS", hash_cmd::cmd_hexists);
        self.register("HKEYS", hash_cmd::cmd_hkeys);
        self.register("HVALS", hash_cmd::cmd_hvals);
        self.register("HINCRBY", hash_cmd::cmd_hincrby);
        self.register("HINCRBYFLOAT", hash_cmd::cmd_hincrbyfloat);
        self.register("HSETNX", hash_cmd::cmd_hsetnx);

        // List commands
        self.register("LPUSH", list_cmd::cmd_lpush);
        self.register("RPUSH", list_cmd::cmd_rpush);
        self.register("LPOP", list_cmd::cmd_lpop);
        self.register("RPOP", list_cmd::cmd_rpop);
        self.register("LLEN", list_cmd::cmd_llen);
        self.register("LRANGE", list_cmd::cmd_lrange);
        self.register("LINDEX", list_cmd::cmd_lindex);
        self.register("LSET", list_cmd::cmd_lset);
        self.register("LREM", list_cmd::cmd_lrem);
        self.register("LTRIM", list_cmd::cmd_ltrim);
        self.register("LINSERT", list_cmd::cmd_linsert);

        // Set commands
        self.register("SADD", set_cmd::cmd_sadd);
        self.register("SREM", set_cmd::cmd_srem);
        self.register("SISMEMBER", set_cmd::cmd_sismember);
        self.register("SMEMBERS", set_cmd::cmd_smembers);
        self.register("SCARD", set_cmd::cmd_scard);
        self.register("SPOP", set_cmd::cmd_spop);
        self.register("SRANDMEMBER", set_cmd::cmd_srandmember);
        self.register("SUNION", set_cmd::cmd_sunion);
        self.register("SINTER", set_cmd::cmd_sinter);
        self.register("SDIFF", set_cmd::cmd_sdiff);

        // ZSet commands
        self.register("ZADD", zset_cmd::cmd_zadd);
        self.register("ZREM", zset_cmd::cmd_zrem);
        self.register("ZSCORE", zset_cmd::cmd_zscore);
        self.register("ZRANK", zset_cmd::cmd_zrank);
        self.register("ZREVRANK", zset_cmd::cmd_zrevrank);
        self.register("ZRANGE", zset_cmd::cmd_zrange);
        self.register("ZREVRANGE", zset_cmd::cmd_zrevrange);
        self.register("ZRANGEBYSCORE", zset_cmd::cmd_zrangebyscore);
        self.register("ZCARD", zset_cmd::cmd_zcard);
        self.register("ZCOUNT", zset_cmd::cmd_zcount);
        self.register("ZINCRBY", zset_cmd::cmd_zincrby);

        // Key commands
        self.register("DEL", cmd_del);
        self.register("EXISTS", cmd_exists);
        self.register("TYPE", cmd_type);
        self.register("EXPIRE", cmd_expire);
        self.register("PEXPIRE", cmd_pexpire);
        self.register("TTL", cmd_ttl);
        self.register("PTTL", cmd_pttl);
        self.register("PERSIST", cmd_persist);
        self.register("KEYS", cmd_keys);
        self.register("RENAME", cmd_rename);
        self.register("DBSIZE", cmd_dbsize);
        self.register("FLUSHDB", cmd_flushdb);
        self.register("FLUSHALL", cmd_flushdb);

        // Server commands
        self.register("PING", cmd_ping);
        self.register("ECHO", cmd_echo);
        self.register("COMMAND", cmd_command);
        self.register("INFO", cmd_info);
        self.register("SELECT", cmd_select);
    }

    pub fn execute(&self, db: &mut Database, args: &[Vec<u8>]) -> RespValue {
        if args.is_empty() {
            return RespValue::error("ERR empty command");
        }

        let cmd_name = String::from_utf8_lossy(&args[0]).to_uppercase();
        match self.commands.get(&cmd_name) {
            Some(handler) => handler(db, &args[1..]),
            None => RespValue::error(format!("ERR unknown command '{}'", cmd_name)),
        }
    }
}

// --- Key commands ---

fn cmd_del(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() {
        return RespValue::error("ERR wrong number of arguments for 'del' command");
    }
    let mut count = 0i64;
    for key in args {
        if db.del(key) {
            count += 1;
        }
    }
    RespValue::integer(count)
}

fn cmd_exists(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() {
        return RespValue::error("ERR wrong number of arguments for 'exists' command");
    }
    let mut count = 0i64;
    for key in args {
        if db.exists(key) {
            count += 1;
        }
    }
    RespValue::integer(count)
}

fn cmd_type(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'type' command");
    }
    RespValue::SimpleString(db.key_type(&args[0]).to_string())
}

fn cmd_expire(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'expire' command");
    }
    let seconds = match parse_integer(&args[1]) {
        Some(n) if n > 0 => n as u64,
        _ => return RespValue::error("ERR invalid expire time in 'expire' command"),
    };
    RespValue::integer(if db.expire(&args[0], seconds) { 1 } else { 0 })
}

fn cmd_pexpire(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'pexpire' command");
    }
    let ms = match parse_integer(&args[1]) {
        Some(n) if n > 0 => n as u64,
        _ => return RespValue::error("ERR invalid expire time in 'pexpire' command"),
    };
    RespValue::integer(if db.pexpire(&args[0], ms) { 1 } else { 0 })
}

fn cmd_ttl(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'ttl' command");
    }
    RespValue::integer(db.ttl(&args[0]))
}

fn cmd_pttl(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'pttl' command");
    }
    RespValue::integer(db.pttl(&args[0]))
}

fn cmd_persist(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'persist' command");
    }
    RespValue::integer(if db.persist(&args[0]) { 1 } else { 0 })
}

fn cmd_keys(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'keys' command");
    }
    let pattern = String::from_utf8_lossy(&args[0]);
    let keys = db.keys(&pattern);
    RespValue::array(keys.into_iter().map(RespValue::bulk).collect())
}

fn cmd_rename(db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 2 {
        return RespValue::error("ERR wrong number of arguments for 'rename' command");
    }
    if db.rename(&args[0], args[1].clone()) {
        RespValue::ok()
    } else {
        RespValue::error("ERR no such key")
    }
}

fn cmd_dbsize(db: &mut Database, _args: &[Vec<u8>]) -> RespValue {
    RespValue::integer(db.dbsize() as i64)
}

fn cmd_flushdb(db: &mut Database, _args: &[Vec<u8>]) -> RespValue {
    db.flushdb();
    RespValue::ok()
}

fn cmd_ping(_db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.is_empty() {
        RespValue::SimpleString("PONG".to_string())
    } else {
        RespValue::bulk(args[0].clone())
    }
}

fn cmd_echo(_db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'echo' command");
    }
    RespValue::bulk(args[0].clone())
}

fn cmd_command(_db: &mut Database, _args: &[Vec<u8>]) -> RespValue {
    // Simplified: just return OK
    RespValue::ok()
}

fn cmd_info(_db: &mut Database, _args: &[Vec<u8>]) -> RespValue {
    let info = "# Server\r\nredis_version:0.1.0-rust\r\nredis_mode:standalone\r\n";
    RespValue::bulk(info.as_bytes().to_vec())
}

fn cmd_select(_db: &mut Database, args: &[Vec<u8>]) -> RespValue {
    if args.len() != 1 {
        return RespValue::error("ERR wrong number of arguments for 'select' command");
    }
    // Only support db 0 for now
    let db_num = parse_integer(&args[0]).unwrap_or(-1);
    if db_num == 0 {
        RespValue::ok()
    } else {
        RespValue::error("ERR DB index is out of range")
    }
}

/// Helper: parse bytes as i64
pub fn parse_integer(data: &[u8]) -> Option<i64> {
    std::str::from_utf8(data).ok().and_then(|s| s.parse().ok())
}

/// Helper: parse bytes as f64
pub fn parse_float(data: &[u8]) -> Option<f64> {
    std::str::from_utf8(data).ok().and_then(|s| s.parse().ok())
}
