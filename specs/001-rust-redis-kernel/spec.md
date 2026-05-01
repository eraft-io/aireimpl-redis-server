# Feature Specification: Rust Redis Kernel Implementation

**Feature Branch**: `001-rust-redis-kernel`  
**Created**: 2026-05-01  
**Status**: Implemented  
**Input**: User description: "使用 Rust 重新写 Redis 内核实现，要求实现 string hash list set zset 基本数据结构"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - String 数据操作 (Priority: P1)

用户通过 redis-cli 连接服务器，执行 String 类型的基本读写操作（GET/SET/INCR 等），服务器返回与原版 Redis 完全一致的响应。

**Why this priority**: String 是 Redis 最基础、使用最广泛的数据类型，是验证整个系统（协议解析、命令分发、存储引擎）端到端可用的基石。

**Independent Test**: 使用 redis-cli 连接 127.0.0.1:6379，执行 SET/GET/INCR 等命令并验证响应。

**Acceptance Scenarios**:

1. **Given** 服务器已启动, **When** 执行 `SET mykey "hello"`, **Then** 返回 `OK`
2. **Given** mykey 已设置为 "hello", **When** 执行 `GET mykey`, **Then** 返回 `"hello"`
3. **Given** counter 不存在, **When** 执行 `INCR counter`, **Then** 返回 `1`
4. **Given** mykey 已设置, **When** 执行 `SET mykey "world" NX`, **Then** 返回 `(nil)`（不覆盖）
5. **Given** mykey 已设置, **When** 执行 `SETEX mykey 60 "temp"`, **Then** key 在 60 秒后自动过期

**已实现命令**: GET, SET (NX/XX/EX/PX), SETNX, SETEX, PSETEX, MGET, MSET, INCR, INCRBY, INCRBYFLOAT, DECR, DECRBY, APPEND, STRLEN, GETRANGE, SETRANGE

**实现文件**:
- 数据结构: `src/storage/string.rs` - RedisString 枚举 (Raw/Integer)，支持整数编码优化
- 命令处理: `src/command/string_cmd.rs` - 16 个命令处理函数

---

### User Story 2 - Hash 数据操作 (Priority: P1)

用户通过 redis-cli 执行 Hash 类型的字段级读写操作（HSET/HGET/HGETALL 等），服务器自动在 ziplist 和 hashtable 编码之间切换。

**Why this priority**: Hash 是存储对象属性的核心数据结构，使用频率仅次于 String。

**Independent Test**: 执行 HSET/HGET/HGETALL 命令并验证字段级 CRUD 操作。

**Acceptance Scenarios**:

1. **Given** 空数据库, **When** 执行 `HSET user name "alice" age "30"`, **Then** 返回 `2`（新增 2 个字段）
2. **Given** user hash 已存在, **When** 执行 `HGET user name`, **Then** 返回 `"alice"`
3. **Given** user hash 有 2 个字段, **When** 执行 `HGETALL user`, **Then** 返回 4 个元素的数组
4. **Given** 小 hash（<128 字段）, **When** 插入第 129 个字段, **Then** 自动从 ZipList 转换为 HashMap

**已实现命令**: HSET, HGET, HDEL, HLEN, HGETALL, HMSET, HMGET, HEXISTS, HKEYS, HVALS, HINCRBY, HINCRBYFLOAT, HSETNX

**实现文件**:
- 数据结构: `src/storage/hash.rs` - RedisHash 枚举 (ZipList/HashMap)，阈值 128 条目 / 64 字节
- 命令处理: `src/command/hash_cmd.rs` - 13 个命令处理函数

---

### User Story 3 - List 数据操作 (Priority: P1)

用户通过 redis-cli 执行 List 类型的双端队列操作（LPUSH/RPUSH/LPOP/RPOP/LRANGE 等）。

**Why this priority**: List 是消息队列和任务队列的常用数据结构。

**Independent Test**: 执行 LPUSH/RPUSH/LRANGE 命令并验证双端操作和范围查询。

**Acceptance Scenarios**:

1. **Given** 空数据库, **When** 执行 `LPUSH mylist "a" "b" "c"`, **Then** 返回 `3`
2. **Given** mylist 有 3 个元素, **When** 执行 `LRANGE mylist 0 -1`, **Then** 返回 `["c", "b", "a"]`
3. **Given** mylist 有元素, **When** 执行 `RPOP mylist`, **Then** 返回最后一个元素
4. **Given** mylist 有元素, **When** 执行 `LINSERT mylist BEFORE "b" "x"`, **Then** 在 "b" 前插入 "x"

**已实现命令**: LPUSH, RPUSH, LPOP, RPOP, LLEN, LRANGE, LINDEX, LSET, LREM, LTRIM, LINSERT

**实现文件**:
- 数据结构: `src/storage/list.rs` - RedisList，底层使用 VecDeque 实现 O(1) 双端操作
- 命令处理: `src/command/list_cmd.rs` - 11 个命令处理函数

---

### User Story 4 - Set 数据操作 (Priority: P2)

用户通过 redis-cli 执行 Set 类型的集合操作（SADD/SMEMBERS/SUNION/SINTER/SDIFF 等），支持 intset 优化编码。

**Why this priority**: Set 提供去重和集合运算能力，是标签系统和关系计算的常用数据结构。

**Independent Test**: 执行 SADD/SMEMBERS/SINTER 命令并验证集合运算。

**Acceptance Scenarios**:

1. **Given** 空数据库, **When** 执行 `SADD myset "a" "b" "c"`, **Then** 返回 `3`
2. **Given** myset 有 3 个元素, **When** 执行 `SISMEMBER myset "a"`, **Then** 返回 `1`
3. **Given** 两个 set, **When** 执行 `SINTER set1 set2`, **Then** 返回交集
4. **Given** 全整数小集合, **When** 添加非整数成员, **Then** 自动从 IntSet 转换为 HashSet

**已实现命令**: SADD, SREM, SISMEMBER, SMEMBERS, SCARD, SPOP, SRANDMEMBER, SUNION, SINTER, SDIFF

**实现文件**:
- 数据结构: `src/storage/set.rs` - RedisSet 枚举 (IntSet/HashSet)，阈值 512 个整数
- 命令处理: `src/command/set_cmd.rs` - 10 个命令处理函数

---

### User Story 5 - ZSet 有序集合操作 (Priority: P2)

用户通过 redis-cli 执行 ZSet 类型的有序集合操作（ZADD/ZRANGE/ZRANGEBYSCORE 等），支持按 score 排序和排名查询。

**Why this priority**: ZSet 是排行榜、优先级队列的核心数据结构，内部 SkipList 实现是 Redis 的标志性设计。

**Independent Test**: 执行 ZADD/ZRANGE/ZRANK 命令并验证有序集合操作。

**Acceptance Scenarios**:

1. **Given** 空数据库, **When** 执行 `ZADD leaderboard 100 "alice" 200 "bob"`, **Then** 返回 `2`
2. **Given** leaderboard 有 2 个成员, **When** 执行 `ZRANGE leaderboard 0 -1 WITHSCORES`, **Then** 返回按 score 升序排列的成员和分数
3. **Given** leaderboard 有成员, **When** 执行 `ZRANK leaderboard "alice"`, **Then** 返回 0（排名从 0 开始）
4. **Given** leaderboard 有成员, **When** 执行 `ZINCRBY leaderboard 50 "alice"`, **Then** alice 的 score 变为 150

**已实现命令**: ZADD, ZREM, ZSCORE, ZRANK, ZREVRANK, ZRANGE, ZREVRANGE, ZRANGEBYSCORE, ZCARD, ZCOUNT, ZINCRBY

**实现文件**:
- 数据结构: `src/storage/zset.rs` - 自实现 SkipList（MAXLEVEL=32, P=0.25）+ HashMap 双索引
- 命令处理: `src/command/zset_cmd.rs` - 11 个命令处理函数

---

### User Story 6 - 键管理与过期 (Priority: P2)

用户可以对任意类型的 key 执行通用管理操作（DEL/EXISTS/EXPIRE/TTL/RENAME 等），支持毫秒级过期精度。

**Why this priority**: 键管理是所有数据类型的公共基础设施。

**Independent Test**: 执行 SET + EXPIRE + TTL 命令验证过期机制。

**Acceptance Scenarios**:

1. **Given** mykey 存在, **When** 执行 `EXPIRE mykey 10`, **Then** 返回 `1`，10 秒后 key 自动删除
2. **Given** mykey 有过期时间, **When** 执行 `TTL mykey`, **Then** 返回剩余秒数
3. **Given** mykey 有过期时间, **When** 执行 `PERSIST mykey`, **Then** 移除过期时间
4. **Given** 数据库有多个 key, **When** 执行 `KEYS *`, **Then** 返回所有 key 列表

**已实现命令**: DEL, EXISTS, TYPE, EXPIRE, PEXPIRE, TTL, PTTL, PERSIST, KEYS, RENAME, DBSIZE, FLUSHDB, FLUSHALL

**实现文件**:
- 数据库核心: `src/storage/db.rs` - HashMap 键空间 + HashMap 过期字典
- 过期策略: 惰性删除（每次访问检查）+ 定期删除（每 100ms 随机抽样 20 个 key）
- 命令处理: `src/command/mod.rs` - 通用键命令和服务器命令

---

### User Story 7 - RESP 协议兼容 (Priority: P1)

服务器实现 RESP2 协议，可通过标准 redis-cli 工具直接连接和使用，无需任何客户端修改。

**Why this priority**: 协议兼容性是整个系统可用的前提。

**Independent Test**: 使用 redis-cli 连接并执行 PING 命令。

**Acceptance Scenarios**:

1. **Given** 服务器在 6379 端口启动, **When** 使用 `redis-cli` 连接, **Then** 连接成功
2. **Given** 已连接, **When** 发送 `PING`, **Then** 返回 `PONG`
3. **Given** 已连接, **When** 发送 inline 命令 `PING\r\n`, **Then** 正确解析并返回 `PONG`
4. **Given** 已连接, **When** 发送 RESP 格式 `*1\r\n$4\r\nPING\r\n`, **Then** 正确解析并返回 `PONG`

**实现文件**:
- 协议解析: `src/protocol/parser.rs` - 增量解析器，支持 Simple String / Error / Integer / Bulk String / Array / Inline
- 协议编码: `src/protocol/writer.rs` - RESP 响应编码器
- 类型定义: `src/protocol/mod.rs` - RespValue 枚举

---

### Edge Cases

- key 不存在时 GET 返回 `(nil)`，TTL 返回 `-2`
- 对错误类型执行命令时返回 `WRONGTYPE` 错误
- INCR 对非整数值返回 `ERR value is not an integer or out of range`
- INCR 溢出时返回 `ERR increment or decrement would overflow`
- INCRBYFLOAT 产生 NaN/Infinity 时返回错误
- SET 的 NX/XX 选项互斥行为
- 负数索引支持（LRANGE/LINDEX/ZRANGE 等）
- 空列表/集合/有序集合的 POP 操作返回 `(nil)`
- KEYS 命令支持 `*` 和 `?` glob 模式匹配
- 并发连接共享数据库通过 `Arc<Mutex<Database>>` 保证线程安全

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 系统 MUST 实现完整的 RESP2 协议解析和编码，兼容标准 redis-cli
- **FR-002**: 系统 MUST 实现 String 数据结构，支持 16 个命令（GET/SET/INCR 等），含整数编码优化
- **FR-003**: 系统 MUST 实现 Hash 数据结构，支持 13 个命令（HSET/HGET/HGETALL 等），含 ziplist/hashtable 自动编码切换
- **FR-004**: 系统 MUST 实现 List 数据结构，支持 11 个命令（LPUSH/RPUSH/LRANGE 等），基于 VecDeque 双端队列
- **FR-005**: 系统 MUST 实现 Set 数据结构，支持 10 个命令（SADD/SMEMBERS/SINTER 等），含 intset/hashtable 自动编码切换
- **FR-006**: 系统 MUST 实现 ZSet 数据结构，支持 11 个命令（ZADD/ZRANGE/ZRANK 等），基于自实现 SkipList + HashMap 双索引
- **FR-007**: 系统 MUST 实现键空间管理，支持 13 个通用命令（DEL/EXISTS/EXPIRE/TTL 等）
- **FR-008**: 系统 MUST 实现惰性过期（访问时检查）和定期过期（100ms 周期随机抽样）双重过期删除策略
- **FR-009**: 系统 MUST 基于 tokio 异步运行时实现并发 TCP 服务器，每连接一个异步 task
- **FR-010**: 系统 MUST 对错误类型操作返回 WRONGTYPE 错误，与 Redis 行为一致
- **FR-011**: 系统 MUST 支持 PING/ECHO/INFO/SELECT/COMMAND 等基础服务器命令
- **FR-012**: 系统 SHOULD 提供 AOF 持久化框架（已实现 RESP 格式追加写入，支持 always/everysec/no 三种 fsync 策略）
- **FR-013**: 系统 SHOULD 提供 RDB 持久化框架（接口已定义，序列化逻辑待实现）

### Key Entities

- **RedisObject**: 统一值类型枚举，包装 String/List/Hash/Set/ZSet 五种数据结构
- **Database**: 核心数据库，管理键空间（`HashMap<Vec<u8>, RedisObject>`）和过期字典（`HashMap<Vec<u8>, Instant>`）
- **CommandTable**: 命令注册表，映射命令名到处理函数（`HashMap<String, CommandHandler>`）
- **RespValue**: RESP 协议值类型枚举（SimpleString/Error/Integer/BulkString/Array）
- **SkipList**: 自实现跳表，用于 ZSet 的有序索引（MAXLEVEL=32, P=0.25）

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 服务器可通过 `cargo build` 零错误编译（已达成，仅有 dead_code 警告）
- **SC-002**: 使用标准 redis-cli 可连接 127.0.0.1:6379 并执行所有 60+ 个已注册命令
- **SC-003**: 5 种数据结构均实现编码优化（String 整数编码、Hash ziplist、Set intset）
- **SC-004**: 过期机制正确工作：惰性删除 + 每 100ms 定期删除
- **SC-005**: 并发连接安全：通过 Arc<Mutex> 保护共享数据库状态

## Assumptions

- 目标平台为 macOS/Linux，Rust 1.75+ stable 工具链
- 单机单实例部署，不涉及集群/主从复制
- 仅支持 db 0（SELECT 0），不支持多数据库
- 持久化模块（RDB/AOF）为框架实现，RDB 序列化待后续完善
- 性能目标为单机 10万+ QPS（GET/SET），与原版 Redis 同一量级
- 不支持 Lua 脚本、Pub/Sub、事务（MULTI/EXEC）等高级特性
- 不支持 RESP3 协议，仅兼容 RESP2

## Technical Context

**Language/Version**: Rust 1.75+ (stable, edition 2021)  
**Primary Dependencies**: tokio 1.x (async runtime), bytes 1.x (buffer management), thiserror 1.x (error types), rand 0.8.x (random), log + env_logger (logging)  
**Storage**: In-memory HashMap keyspace  
**Testing**: cargo test  
**Target Platform**: macOS / Linux  
**Project Type**: CLI server application  

## Project Structure

```text
src/
├── main.rs                 # 入口，初始化日志和配置，启动服务器
├── server.rs               # tokio TCP 服务器，后台过期清理任务
├── connection.rs           # 单连接处理：读取 -> 解析 -> 执行 -> 响应
├── config.rs               # 配置管理（bind/port/maxclients/loglevel）
├── protocol/
│   ├── mod.rs              # RespValue 枚举定义和辅助方法
│   ├── parser.rs           # RESP2 增量解析器（含 inline 命令支持）
│   └── writer.rs           # RESP 响应编码器
├── command/
│   ├── mod.rs              # CommandTable 注册表 + 通用键命令 + 服务器命令
│   ├── string_cmd.rs       # String 类型 16 个命令
│   ├── hash_cmd.rs         # Hash 类型 13 个命令
│   ├── list_cmd.rs         # List 类型 11 个命令
│   ├── set_cmd.rs          # Set 类型 10 个命令
│   └── zset_cmd.rs         # ZSet 类型 11 个命令
├── storage/
│   ├── mod.rs              # 模块导出
│   ├── db.rs               # Database 核心（键空间 + 过期管理 + glob 匹配）
│   ├── object.rs           # RedisObject 统一值类型枚举
│   ├── string.rs           # RedisString（Raw/Integer 编码）
│   ├── hash.rs             # RedisHash（ZipList/HashMap 自动切换）
│   ├── list.rs             # RedisList（VecDeque 双端队列）
│   ├── set.rs              # RedisSet（IntSet/HashSet 自动切换）
│   └── zset.rs             # RedisZSet（SkipList + HashMap 双索引）
└── persistence/
    ├── mod.rs              # 模块导出
    ├── rdb.rs              # RDB 快照持久化（框架，待完善）
    └── aof.rs              # AOF 日志持久化（已实现 RESP 格式写入）
```

## Command Summary (60+ commands)

| 类型 | 数量 | 命令列表 |
|------|------|----------|
| String | 16 | GET, SET, SETNX, SETEX, PSETEX, MGET, MSET, INCR, INCRBY, INCRBYFLOAT, DECR, DECRBY, APPEND, STRLEN, GETRANGE, SETRANGE |
| Hash | 13 | HSET, HGET, HDEL, HLEN, HGETALL, HMSET, HMGET, HEXISTS, HKEYS, HVALS, HINCRBY, HINCRBYFLOAT, HSETNX |
| List | 11 | LPUSH, RPUSH, LPOP, RPOP, LLEN, LRANGE, LINDEX, LSET, LREM, LTRIM, LINSERT |
| Set | 10 | SADD, SREM, SISMEMBER, SMEMBERS, SCARD, SPOP, SRANDMEMBER, SUNION, SINTER, SDIFF |
| ZSet | 11 | ZADD, ZREM, ZSCORE, ZRANK, ZREVRANK, ZRANGE, ZREVRANGE, ZRANGEBYSCORE, ZCARD, ZCOUNT, ZINCRBY |
| Key | 13 | DEL, EXISTS, TYPE, EXPIRE, PEXPIRE, TTL, PTTL, PERSIST, KEYS, RENAME, DBSIZE, FLUSHDB, FLUSHALL |
| Server | 5 | PING, ECHO, COMMAND, INFO, SELECT |
