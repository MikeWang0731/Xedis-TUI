use crate::backend::client::ClusterNodeInfo;
use crate::core::macro_engine::MacroEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionKind {
    Macro,
    Node,
    Command,
    Subcommand,
    #[allow(dead_code)]
    Argument,
}

impl SuggestionKind {
    pub fn badge(&self) -> &'static str {
        match self {
            SuggestionKind::Macro => "[MACRO]",
            SuggestionKind::Node => "[NODE]",
            SuggestionKind::Command => "[CMD]",
            SuggestionKind::Subcommand => "[SUB]",
            SuggestionKind::Argument => "[ARG]",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionItem {
    pub kind: SuggestionKind,
    pub completion_text: String,
    pub display_title: String,
    pub description: String,
    pub syntax: String,
    pub example: String,
}

#[derive(Debug, Clone)]
pub struct RedisCommandSpec {
    pub name: &'static str,
    pub signature: &'static str,
    pub description: &'static str,
    #[allow(dead_code)]
    pub category: &'static str,
    pub example: &'static str,
}

#[derive(Debug, Clone)]
pub struct SubcommandSpec {
    pub parent_cmd: &'static str,
    pub name: &'static str,
    pub signature: &'static str,
    pub description: &'static str,
    pub example: &'static str,
}

pub struct AutocompleteEngine;

impl AutocompleteEngine {
    pub const REDIS_COMMANDS: &'static [RedisCommandSpec] = &[
        // High-Risk & Safety Guard Commands
        RedisCommandSpec { name: "KEYS", signature: "KEYS pattern", description: "[!] 全量键空间模式匹配 (千万级数据阻塞单线程，建议 /scan)", category: "Generic", example: "KEYS user:*" },
        RedisCommandSpec { name: "FLUSHALL", signature: "FLUSHALL [ASYNC|SYNC]", description: "[!] 彻底清空 Redis 所有数据库的所有数据 (不可逆)", category: "Server", example: "FLUSHALL ASYNC" },
        RedisCommandSpec { name: "FLUSHDB", signature: "FLUSHDB [ASYNC|SYNC]", description: "[!] 彻底清空当前数据库的所有数据 (不可逆)", category: "Server", example: "FLUSHDB ASYNC" },
        RedisCommandSpec { name: "SHUTDOWN", signature: "SHUTDOWN [NOSAVE|SAVE|NOW|FORCE]", description: "[!] 同步保存并关闭终止 Redis 服务进程", category: "Server", example: "SHUTDOWN SAVE" },
        RedisCommandSpec { name: "DEBUG", signature: "DEBUG [SEGFAULT|SLEEP|OBJECT|RELOAD]", description: "[!] 内部诊断与调试命令族 (包含崩溃与强制休眠)", category: "Server", example: "DEBUG SLEEP 5" },
        RedisCommandSpec { name: "SAVE", signature: "SAVE", description: "[!] 同步阻塞生成 RDB 快照文件 (阻塞主线程所有请求)", category: "Server", example: "SAVE" },
        RedisCommandSpec { name: "BGSAVE", signature: "BGSAVE [SCHEDULE]", description: "[!] 后台异步 Fork 子进程生成 RDB 快照 (Copy-On-Write 内存翻倍)", category: "Server", example: "BGSAVE" },
        RedisCommandSpec { name: "BGREWRITEAOF", signature: "BGREWRITEAOF", description: "[!] 后台异步重写 AOF 持久化日志文件", category: "Server", example: "BGREWRITEAOF" },
        RedisCommandSpec { name: "SLAVEOF", signature: "SLAVEOF host port | NO ONE", description: "[!] 动态修改从节点复制目标或脱离集群独立", category: "Replication", example: "SLAVEOF NO ONE" },
        RedisCommandSpec { name: "REPLICAOF", signature: "REPLICAOF host port | NO ONE", description: "[!] 动态修改从节点复制目标或脱离集群独立", category: "Replication", example: "REPLICAOF NO ONE" },
        RedisCommandSpec { name: "SWAPDB", signature: "SWAPDB index1 index2", description: "[!] 原子交换两个 Redis 数据库的键空间编号映射", category: "Server", example: "SWAPDB 0 1" },
        RedisCommandSpec { name: "MIGRATE", signature: "MIGRATE host port key destination-db timeout [COPY|REPLACE]", description: "[!] 原子跨实例迁移指定 Key 至目标 Redis 实例", category: "Generic", example: "MIGRATE 127.0.0.1 6380 user:1001 0 5000" },
        RedisCommandSpec { name: "RESTORE", signature: "RESTORE key ttl serialized-value [REPLACE|ABSTTL]", description: "反序列化二进制数据并恢复写入指定 Key", category: "Generic", example: "RESTORE user:copy 0 \"\\x0a...\"" },

        // String Commands
        RedisCommandSpec { name: "GET", signature: "GET key", description: "获取指定 Key 的字符串值", category: "String", example: "GET user:1001:name" },
        RedisCommandSpec { name: "SET", signature: "SET key value [EX sec|PX ms] [NX|XX]", description: "设置 Key 的字符串值及可选过期时间", category: "String", example: "SET token:session \"xyz990\" EX 3600" },
        RedisCommandSpec { name: "SETNX", signature: "SETNX key value", description: "仅当 Key 不存在时设置值 (常用于分布式锁)", category: "String", example: "SETNX lock:order:1024 \"locked\"" },
        RedisCommandSpec { name: "MGET", signature: "MGET key [key ...]", description: "批量获取一个或多个 Key 的值", category: "String", example: "MGET k1 k2 k3" },
        RedisCommandSpec { name: "MSET", signature: "MSET key value [key value ...]", description: "批量设置多个键值对", category: "String", example: "MSET k1 \"v1\" k2 \"v2\"" },
        RedisCommandSpec { name: "INCR", signature: "INCR key", description: "将 Key 存储的整数值加 1", category: "String", example: "INCR page:view:count" },
        RedisCommandSpec { name: "DECR", signature: "DECR key", description: "将 Key 存储的整数值减 1", category: "String", example: "DECR stock:item:99" },
        RedisCommandSpec { name: "STRLEN", signature: "STRLEN key", description: "获取 Key 存储字符串的长度", category: "String", example: "STRLEN user:bio" },
        RedisCommandSpec { name: "APPEND", signature: "APPEND key value", description: "将指定值追加至已有 Key 的末尾", category: "String", example: "APPEND log:data \"\\nnewline\"" },

        // Hash Commands
        RedisCommandSpec { name: "HGETALL", signature: "HGETALL key", description: "获取哈希表中所有字段与值 (自动格式化为表格)", category: "Hash", example: "HGETALL user:profile:8801" },
        RedisCommandSpec { name: "HGET", signature: "HGET key field", description: "获取哈希表中指定字段的值", category: "Hash", example: "HGET user:profile:8801 name" },
        RedisCommandSpec { name: "HSET", signature: "HSET key field value [field value ...]", description: "设置哈希表中一个或多个字段的值", category: "Hash", example: "HSET user:profile:8801 age 28 role \"admin\"" },
        RedisCommandSpec { name: "HDEL", signature: "HDEL key field [field ...]", description: "删除哈希表中一个或多个字段", category: "Hash", example: "HDEL user:profile:8801 temp_token" },
        RedisCommandSpec { name: "HEXISTS", signature: "HEXISTS key field", description: "查看哈希表中是否存在指定字段", category: "Hash", example: "HEXISTS user:profile:8801 email" },
        RedisCommandSpec { name: "HLEN", signature: "HLEN key", description: "获取哈希表中字段的数量", category: "Hash", example: "HLEN user:profile:8801" },
        RedisCommandSpec { name: "HKEYS", signature: "HKEYS key", description: "获取哈希表中的所有字段名", category: "Hash", example: "HKEYS user:profile:8801" },
        RedisCommandSpec { name: "HVALS", signature: "HVALS key", description: "获取哈希表中的所有值", category: "Hash", example: "HVALS user:profile:8801" },
        RedisCommandSpec { name: "HINCRBY", signature: "HINCRBY key field increment", description: "为哈希表中指定字段的整数值加上增量", category: "Hash", example: "HINCRBY user:profile:8801 score 10" },

        // List Commands
        RedisCommandSpec { name: "LPUSH", signature: "LPUSH key element [element ...]", description: "将一个或多个元素插入到列表头部", category: "List", example: "LPUSH queue:jobs \"job_101\"" },
        RedisCommandSpec { name: "RPUSH", signature: "RPUSH key element [element ...]", description: "将一个或多个元素插入到列表尾部", category: "List", example: "RPUSH queue:jobs \"job_102\"" },
        RedisCommandSpec { name: "LPOP", signature: "LPOP key [count]", description: "移出并获取列表的第一个元素", category: "List", example: "LPOP queue:jobs" },
        RedisCommandSpec { name: "RPOP", signature: "RPOP key [count]", description: "移出并获取列表的最后一个元素", category: "List", example: "RPOP queue:jobs" },
        RedisCommandSpec { name: "LRANGE", signature: "LRANGE key start stop", description: "获取列表指定区间内的元素", category: "List", example: "LRANGE queue:jobs 0 9" },
        RedisCommandSpec { name: "LLEN", signature: "LLEN key", description: "获取列表的长度", category: "List", example: "LLEN queue:jobs" },
        RedisCommandSpec { name: "LINDEX", signature: "LINDEX key index", description: "通过索引获取列表中的元素", category: "List", example: "LINDEX queue:jobs 0" },

        // Set Commands
        RedisCommandSpec { name: "SADD", signature: "SADD key member [member ...]", description: "向集合中添加一个或多个成员", category: "Set", example: "SADD tag:books \"sci-fi\" \"tech\"" },
        RedisCommandSpec { name: "SMEMBERS", signature: "SMEMBERS key", description: "[!] 获取集合中所有成员 (大集合推荐用 SSCAN)", category: "Set", example: "SMEMBERS tag:books" },
        RedisCommandSpec { name: "SREM", signature: "SREM key member [member ...]", description: "从集合中移除一个或多个成员", category: "Set", example: "SREM tag:books \"deprecated\"" },
        RedisCommandSpec { name: "SISMEMBER", signature: "SISMEMBER key member", description: "判断成员元素是否是集合的成员", category: "Set", example: "SISMEMBER tag:books \"tech\"" },
        RedisCommandSpec { name: "SCARD", signature: "SCARD key", description: "获取集合中成员的数量", category: "Set", example: "SCARD tag:books" },
        RedisCommandSpec { name: "SPOP", signature: "SPOP key [count]", description: "随机移除并返回集合中的一个或多个元素", category: "Set", example: "SPOP tag:books 1" },

        // Sorted Set Commands
        RedisCommandSpec { name: "ZADD", signature: "ZADD key [NX|XX] [GT|LT] score member ...", description: "向有序集合添加一个或多个成员或更新分数", category: "ZSet", example: "ZADD rank:players 98.5 \"Alice\" 92.0 \"Bob\"" },
        RedisCommandSpec { name: "ZRANGE", signature: "ZRANGE key min max [BYSCORE|BYLEX] [REV] [LIMIT offset count] [WITHSCORES]", description: "获取有序集合指定区间的成员", category: "ZSet", example: "ZRANGE rank:players 0 9 WITHSCORES" },
        RedisCommandSpec { name: "ZREVRANGE", signature: "ZREVRANGE key start stop [WITHSCORES]", description: "返回有序集中指定区间内的成员 (从高到低排序)", category: "ZSet", example: "ZREVRANGE rank:players 0 9 WITHSCORES" },
        RedisCommandSpec { name: "ZSCORE", signature: "ZSCORE key member", description: "获取有序集合中指定成员的分数", category: "ZSet", example: "ZSCORE rank:players \"Alice\"" },
        RedisCommandSpec { name: "ZCARD", signature: "ZCARD key", description: "获取有序集合的成员数", category: "ZSet", example: "ZCARD rank:players" },
        RedisCommandSpec { name: "ZREM", signature: "ZREM key member [member ...]", description: "移除有序集合中的一个或多个成员", category: "ZSet", example: "ZREM rank:players \"Bob\"" },

        // Key & Generic Commands
        RedisCommandSpec { name: "DEL", signature: "DEL key [key ...]", description: "同步删除一个或多个指定 Key", category: "Generic", example: "DEL temp:cache:01" },
        RedisCommandSpec { name: "UNLINK", signature: "UNLINK key [key ...]", description: "非阻塞异步删除一个或多个 Key (大 Key 推荐，优于 DEL)", category: "Generic", example: "UNLINK big_cache_key" },
        RedisCommandSpec { name: "EXISTS", signature: "EXISTS key [key ...]", description: "检查一个或多个 Key 是否存在", category: "Generic", example: "EXISTS user:1001" },
        RedisCommandSpec { name: "EXPIRE", signature: "EXPIRE key seconds", description: "为 Key 设置生存时间 (秒)", category: "Generic", example: "EXPIRE session:id 1800" },
        RedisCommandSpec { name: "EXPIREAT", signature: "EXPIREAT key unix-time-seconds", description: "按 UNIX 时间戳为 Key 设置绝对过期时间", category: "Generic", example: "EXPIREAT token:1 1788228000" },
        RedisCommandSpec { name: "PEXPIRE", signature: "PEXPIRE key milliseconds", description: "以毫秒为单位设置 Key 的生存时间", category: "Generic", example: "PEXPIRE lock:order 500" },
        RedisCommandSpec { name: "TTL", signature: "TTL key", description: "获取 Key 的剩余生存时间 (秒)", category: "Generic", example: "TTL session:id" },
        RedisCommandSpec { name: "PTTL", signature: "PTTL key", description: "以毫秒为单位获取 Key 的剩余生存时间", category: "Generic", example: "PTTL session:id" },
        RedisCommandSpec { name: "PERSIST", signature: "PERSIST key", description: "移除 Key 的过期时间，使其永久有效", category: "Generic", example: "PERSIST session:id" },
        RedisCommandSpec { name: "TYPE", signature: "TYPE key", description: "返回 Key 所存储的值的类型", category: "Generic", example: "TYPE user:1001" },
        RedisCommandSpec { name: "RENAME", signature: "RENAME key newkey", description: "将指定 Key 重命名为 newkey", category: "Generic", example: "RENAME old_key new_key" },
        RedisCommandSpec { name: "RENAMENX", signature: "RENAMENX key newkey", description: "仅当 newkey 不存在时重命名 Key", category: "Generic", example: "RENAMENX old_key new_key" },
        RedisCommandSpec { name: "RANDOMKEY", signature: "RANDOMKEY", description: "从当前数据库中随机返回一个 Key", category: "Generic", example: "RANDOMKEY" },
        RedisCommandSpec { name: "TOUCH", signature: "TOUCH key [key ...]", description: "更新指定 Key 的最后访问时间 (更新 LRU)", category: "Generic", example: "TOUCH session:1001" },
        RedisCommandSpec { name: "COPY", signature: "COPY source destination [DB destination-db] [REPLACE]", description: "将源 Key 复制至目标 Key (Redis 6.2+)", category: "Generic", example: "COPY user:1001 user:1001:bak" },
        RedisCommandSpec { name: "DUMP", signature: "DUMP key", description: "获取指定 Key 的 RESP 序列化二进制内容", category: "Generic", example: "DUMP user:1001" },
        RedisCommandSpec { name: "SCAN", signature: "SCAN cursor [MATCH pattern] [COUNT count]", description: "非阻塞游标迭代当前数据库中的键", category: "Generic", example: "SCAN 0 MATCH user:* COUNT 20" },
        RedisCommandSpec { name: "MEMORY", signature: "MEMORY [USAGE|STATS|PURGE|DOCTOR]", description: "内存分析与诊断管理命令族", category: "Generic", example: "MEMORY USAGE mykey" },

        // Server, Connection & Cluster Commands
        RedisCommandSpec { name: "PING", signature: "PING [message]", description: "测试与服务端的连通性，返回 PONG", category: "Server", example: "PING" },
        RedisCommandSpec { name: "SELECT", signature: "SELECT index", description: "切换当前连接所操作的数据库编号 (0~15)", category: "Server", example: "SELECT 1" },
        RedisCommandSpec { name: "AUTH", signature: "AUTH [username] password", description: "向 Redis 服务端提交连接密码或 ACL 认证", category: "Server", example: "AUTH \"my_password\"" },
        RedisCommandSpec { name: "INFO", signature: "INFO [section]", description: "获取 Redis 服务器各项实时指标与分段统计", category: "Server", example: "INFO persistence" },
        RedisCommandSpec { name: "DBSIZE", signature: "DBSIZE", description: "返回当前数据库的 Key 总数", category: "Server", example: "DBSIZE" },
        RedisCommandSpec { name: "TIME", signature: "TIME", description: "获取 Redis 服务器当前 UNIX 时间戳与微秒数", category: "Server", example: "TIME" },
        RedisCommandSpec { name: "ROLE", signature: "ROLE", description: "获取当前实例的角色与复制偏移量状态", category: "Server", example: "ROLE" },
        RedisCommandSpec { name: "LASTSAVE", signature: "LASTSAVE", description: "返回最近一次成功持久化落盘的 UNIX 时间戳", category: "Server", example: "LASTSAVE" },
        RedisCommandSpec { name: "MONITOR", signature: "MONITOR", description: "实时监听并流式输出服务端接收到的所有命令", category: "Server", example: "MONITOR" },
        RedisCommandSpec { name: "COMMAND", signature: "COMMAND [COUNT|DOCS|INFO|LIST]", description: "查询 Redis 完整命令集元数据与统计", category: "Server", example: "COMMAND COUNT" },
        RedisCommandSpec { name: "SLOWLOG", signature: "SLOWLOG [GET|LEN|RESET]", description: "慢查询日志管理命令族", category: "Server", example: "SLOWLOG GET 10" },
        RedisCommandSpec { name: "CLIENT", signature: "CLIENT [LIST|KILL|GETNAME|SETNAME|PAUSE|ID]", description: "客户端连接管理命令族", category: "Server", example: "CLIENT LIST" },
        RedisCommandSpec { name: "CONFIG", signature: "CONFIG [GET|SET|REWRITE|RESETSTAT]", description: "运行时配置动态查询与管理命令族", category: "Server", example: "CONFIG GET maxmemory*" },
        RedisCommandSpec { name: "CLUSTER", signature: "CLUSTER [NODES|INFO|SLOTS|SHARDS|MYID|FAILOVER]", description: "集群拓扑与分片运维命令族", category: "Cluster", example: "CLUSTER NODES" },
    ];

    pub const SUBCOMMANDS: &'static [SubcommandSpec] = &[
        // FLUSHALL subcommands
        SubcommandSpec { parent_cmd: "FLUSHALL", name: "ASYNC", signature: "FLUSHALL ASYNC", description: "[推荐] 后台异步线程清空全部数据库，避免主线程阻塞", example: "FLUSHALL ASYNC" },
        SubcommandSpec { parent_cmd: "FLUSHALL", name: "SYNC", signature: "FLUSHALL SYNC", description: "同步清空全部数据库 (大实例易导致阻塞)", example: "FLUSHALL SYNC" },

        // FLUSHDB subcommands
        SubcommandSpec { parent_cmd: "FLUSHDB", name: "ASYNC", signature: "FLUSHDB ASYNC", description: "[推荐] 后台异步线程清空当前数据库，避免主线程阻塞", example: "FLUSHDB ASYNC" },
        SubcommandSpec { parent_cmd: "FLUSHDB", name: "SYNC", signature: "FLUSHDB SYNC", description: "同步清空当前数据库", example: "FLUSHDB SYNC" },

        // SHUTDOWN subcommands
        SubcommandSpec { parent_cmd: "SHUTDOWN", name: "SAVE", signature: "SHUTDOWN SAVE", description: "强制落盘保存 RDB 快照后安全退出进程", example: "SHUTDOWN SAVE" },
        SubcommandSpec { parent_cmd: "SHUTDOWN", name: "NOSAVE", signature: "SHUTDOWN NOSAVE", description: "不执行落盘保存，立即终止进程退出", example: "SHUTDOWN NOSAVE" },
        SubcommandSpec { parent_cmd: "SHUTDOWN", name: "NOW", signature: "SHUTDOWN NOW", description: "跳过等待中客户端，立即终止退出", example: "SHUTDOWN NOW" },
        SubcommandSpec { parent_cmd: "SHUTDOWN", name: "FORCE", signature: "SHUTDOWN FORCE", description: "强制忽略保存失败错误退出", example: "SHUTDOWN FORCE" },

        // DEBUG subcommands
        SubcommandSpec { parent_cmd: "DEBUG", name: "SEGFAULT", signature: "DEBUG SEGFAULT", description: "[!] 强制 Redis 进程发生段错误崩溃退出", example: "DEBUG SEGFAULT" },
        SubcommandSpec { parent_cmd: "DEBUG", name: "SLEEP", signature: "DEBUG SLEEP <seconds>", description: "[!] 强制 Redis 主线程休眠挂起指定秒数", example: "DEBUG SLEEP 5" },
        SubcommandSpec { parent_cmd: "DEBUG", name: "OBJECT", signature: "DEBUG OBJECT <key>", description: "获取指定 Key 的内部编码、引用计数与 LRU 诊断信息", example: "DEBUG OBJECT mykey" },

        // REPLICAOF / SLAVEOF subcommands
        SubcommandSpec { parent_cmd: "REPLICAOF", name: "NO ONE", signature: "REPLICAOF NO ONE", description: "停止主从复制并将当前节点提升为主节点 (Master)", example: "REPLICAOF NO ONE" },
        SubcommandSpec { parent_cmd: "SLAVEOF", name: "NO ONE", signature: "SLAVEOF NO ONE", description: "停止主从复制并将当前节点提升为主节点 (Master)", example: "SLAVEOF NO ONE" },

        // BGSAVE subcommands
        SubcommandSpec { parent_cmd: "BGSAVE", name: "SCHEDULE", signature: "BGSAVE SCHEDULE", description: "若当前有 AOF 重写正在执行，则排队等待其完成后再触发快照", example: "BGSAVE SCHEDULE" },

        // COMMAND subcommands
        SubcommandSpec { parent_cmd: "COMMAND", name: "COUNT", signature: "COMMAND COUNT", description: "返回服务端支持的命令总数", example: "COMMAND COUNT" },
        SubcommandSpec { parent_cmd: "COMMAND", name: "DOCS", signature: "COMMAND DOCS [command-name]", description: "返回指定命令的官方文档与参数元数据规格", example: "COMMAND DOCS get" },
        SubcommandSpec { parent_cmd: "COMMAND", name: "INFO", signature: "COMMAND INFO [command-name ...]", description: "返回指定命令的权限标记、Arity 参数量与键位规范", example: "COMMAND INFO mget" },
        SubcommandSpec { parent_cmd: "COMMAND", name: "LIST", signature: "COMMAND LIST", description: "列出当前服务端支持的所有命令名称", example: "COMMAND LIST" },

        // INFO sections
        SubcommandSpec { parent_cmd: "INFO", name: "PERSISTENCE", signature: "INFO persistence", description: "RDB 周期快照与 AOF 持久化状态、重写进度及落盘耗时统计", example: "INFO persistence" },
        SubcommandSpec { parent_cmd: "INFO", name: "SERVER", signature: "INFO server", description: "Redis 服务端核心版本、运行模式、操作系统、进程 ID 与运行时间", example: "INFO server" },
        SubcommandSpec { parent_cmd: "INFO", name: "CLIENTS", signature: "INFO clients", description: "已连接客户端连接数、最大输入输出缓冲队列与阻塞客户端统计", example: "INFO clients" },
        SubcommandSpec { parent_cmd: "INFO", name: "MEMORY", signature: "INFO memory", description: "内存开销明细、分配器分配量、内存碎片率及历史峰值统计", example: "INFO memory" },
        SubcommandSpec { parent_cmd: "INFO", name: "STATS", signature: "INFO stats", description: "常规运行统计、QPS 吞吐、网络流量与 Key 命中/未命中数", example: "INFO stats" },
        SubcommandSpec { parent_cmd: "INFO", name: "REPLICATION", signature: "INFO replication", description: "主从复制状态、拓扑关系、Repl Offset 与从节点同步延迟", example: "INFO replication" },
        SubcommandSpec { parent_cmd: "INFO", name: "CPU", signature: "INFO cpu", description: "Redis 主进程与后台子线程消耗的系统态/用户态 CPU 耗时", example: "INFO cpu" },
        SubcommandSpec { parent_cmd: "INFO", name: "COMMANDSTATS", signature: "INFO commandstats", description: "各项 Redis 命令的累计调用频次、总耗时与平均每条耗时", example: "INFO commandstats" },
        SubcommandSpec { parent_cmd: "INFO", name: "CLUSTER", signature: "INFO cluster", description: "集群拓扑健康度、槽位覆盖率与各节点通信状态统计", example: "INFO cluster" },
        SubcommandSpec { parent_cmd: "INFO", name: "KEYSPACE", signature: "INFO keyspace", description: "当前各数据库包含的 Key 数量、已设置过期时间及平均 TTL", example: "INFO keyspace" },
        SubcommandSpec { parent_cmd: "INFO", name: "ALL", signature: "INFO all", description: "获取 Redis 服务器所有维度的全部监控指标与统计", example: "INFO all" },
        SubcommandSpec { parent_cmd: "INFO", name: "DEFAULT", signature: "INFO default", description: "仅获取默认常规的基础监控指标集合", example: "INFO default" },
        SubcommandSpec { parent_cmd: "INFO", name: "EVERYTHING", signature: "INFO everything", description: "包含所有内置模块及扩展项在内的全量遥测信息", example: "INFO everything" },

        // CLIENT subcommands
        SubcommandSpec { parent_cmd: "CLIENT", name: "LIST", signature: "CLIENT LIST [TYPE normal|master|replica|pubsub]", description: "获取当前所有已建立连接的客户端详细连接属性与空闲时间", example: "CLIENT LIST" },
        SubcommandSpec { parent_cmd: "CLIENT", name: "KILL", signature: "CLIENT KILL [ip:port | ID client-id]", description: "强制关闭满足指定 IP:PORT 或 ID 条件的客户端连接", example: "CLIENT KILL 192.168.1.10:52410" },
        SubcommandSpec { parent_cmd: "CLIENT", name: "GETNAME", signature: "CLIENT GETNAME", description: "获取当前连接显式设置的客户端标识名称", example: "CLIENT GETNAME" },
        SubcommandSpec { parent_cmd: "CLIENT", name: "SETNAME", signature: "CLIENT SETNAME <name>", description: "为当前连接设置客户端友好名称 (便于排障识别)", example: "CLIENT SETNAME worker-pool-01" },
        SubcommandSpec { parent_cmd: "CLIENT", name: "PAUSE", signature: "CLIENT PAUSE <timeout-ms>", description: "按毫秒指定时长挂起并阻塞所有外部客户端请求", example: "CLIENT PAUSE 5000" },
        SubcommandSpec { parent_cmd: "CLIENT", name: "UNPAUSE", signature: "CLIENT UNPAUSE", description: "立即恢复被 CLIENT PAUSE 挂起的客户端连接", example: "CLIENT UNPAUSE" },
        SubcommandSpec { parent_cmd: "CLIENT", name: "ID", signature: "CLIENT ID", description: "返回当前客户端连接的全局唯一递增整数 ID", example: "CLIENT ID" },
        SubcommandSpec { parent_cmd: "CLIENT", name: "INFO", signature: "CLIENT INFO", description: "返回当前连接的完整状态属性详情", example: "CLIENT INFO" },

        // CLUSTER subcommands
        SubcommandSpec { parent_cmd: "CLUSTER", name: "NODES", signature: "CLUSTER NODES", description: "获取集群全部物理节点、角色关系、连接地址与槽位映射拓扑", example: "CLUSTER NODES" },
        SubcommandSpec { parent_cmd: "CLUSTER", name: "INFO", signature: "CLUSTER INFO", description: "获取集群整体运行健康状态、槽位分配数与故障节点统计", example: "CLUSTER INFO" },
        SubcommandSpec { parent_cmd: "CLUSTER", name: "SLOTS", signature: "CLUSTER SLOTS", description: "获取 16384 个槽位与物理 Master/Replica 节点的映射列表", example: "CLUSTER SLOTS" },
        SubcommandSpec { parent_cmd: "CLUSTER", name: "SHARDS", signature: "CLUSTER SHARDS", description: "以 RESP3 结构化格式获取集群所有分片拓扑 (Redis 7+)", example: "CLUSTER SHARDS" },
        SubcommandSpec { parent_cmd: "CLUSTER", name: "MYID", signature: "CLUSTER MYID", description: "获取当前直连集群节点的 40 位十六进制 Node ID", example: "CLUSTER MYID" },
        SubcommandSpec { parent_cmd: "CLUSTER", name: "FAILOVER", signature: "CLUSTER FAILOVER [FORCE|TAKEOVER]", description: "手动强制触发从节点发起故障转移提升为主节点", example: "CLUSTER FAILOVER" },
        SubcommandSpec { parent_cmd: "CLUSTER", name: "REPLICAS", signature: "CLUSTER REPLICAS <node-id>", description: "列出指定主节点 ID 下挂载的所有从节点信息", example: "CLUSTER REPLICAS 7de3165b..." },
        SubcommandSpec { parent_cmd: "CLUSTER", name: "MEET", signature: "CLUSTER MEET <ip> <port>", description: "将指定 IP 和端口的新节点加入当前集群", example: "CLUSTER MEET 127.0.0.1 22004" },
        SubcommandSpec { parent_cmd: "CLUSTER", name: "COUNTKEYSINSLOT", signature: "CLUSTER COUNTKEYSINSLOT <slot>", description: "查询指定槽位 (0~16383) 中当前包含的 Key 总数", example: "CLUSTER COUNTKEYSINSLOT 5460" },
        SubcommandSpec { parent_cmd: "CLUSTER", name: "KEYSLOT", signature: "CLUSTER KEYSLOT <key>", description: "计算指定 Key 对应的 CRC16 哈希槽位编号", example: "CLUSTER KEYSLOT user:1001" },

        // CONFIG subcommands
        SubcommandSpec { parent_cmd: "CONFIG", name: "GET", signature: "CONFIG GET <parameter>", description: "读取 Redis 运行时配置参数值 (支持 glob 通配符)", example: "CONFIG GET maxmemory*" },
        SubcommandSpec { parent_cmd: "CONFIG", name: "SET", signature: "CONFIG SET <parameter> <value>", description: "动态修改 Redis 运行时配置参数 (即时生效)", example: "CONFIG SET slowlog-log-slower-than 10000" },
        SubcommandSpec { parent_cmd: "CONFIG", name: "REWRITE", signature: "CONFIG REWRITE", description: "将内存中动态修改的配置持久化回写至 redis.conf 文件", example: "CONFIG REWRITE" },
        SubcommandSpec { parent_cmd: "CONFIG", name: "RESETSTAT", signature: "CONFIG RESETSTAT", description: "重置 INFO 命令中的各项历史累计统计计数器", example: "CONFIG RESETSTAT" },

        // SLOWLOG subcommands
        SubcommandSpec { parent_cmd: "SLOWLOG", name: "GET", signature: "SLOWLOG GET [count]", description: "按逆序拉取最新产生的慢查询日志记录", example: "SLOWLOG GET 10" },
        SubcommandSpec { parent_cmd: "SLOWLOG", name: "LEN", signature: "SLOWLOG LEN", description: "获取当前内存中已缓存的慢查询日志条数", example: "SLOWLOG LEN" },
        SubcommandSpec { parent_cmd: "SLOWLOG", name: "RESET", signature: "SLOWLOG RESET", description: "清空当前所有的慢查询日志历史缓存", example: "SLOWLOG RESET" },

        // MEMORY subcommands
        SubcommandSpec { parent_cmd: "MEMORY", name: "USAGE", signature: "MEMORY USAGE <key> [SAMPLES count]", description: "深度分析并计算指定 Key 在内存中占用的精确字节数", example: "MEMORY USAGE cache:hot:users" },
        SubcommandSpec { parent_cmd: "MEMORY", name: "STATS", signature: "MEMORY STATS", description: "获取 Redis 内存分配器层面的详尽开销报告与内部缓存", example: "MEMORY STATS" },
        SubcommandSpec { parent_cmd: "MEMORY", name: "PURGE", signature: "MEMORY PURGE", description: "提示内存分配器 (jemalloc) 尝试将未使用的脏页归还操作系统", example: "MEMORY PURGE" },
        SubcommandSpec { parent_cmd: "MEMORY", name: "DOCTOR", signature: "MEMORY DOCTOR", description: "运行内存健康自检引擎并输出分析诊断与优化建议", example: "MEMORY DOCTOR" },

        // /interval arguments
        SubcommandSpec { parent_cmd: "/INTERVAL", name: "500MS", signature: "/interval 500ms", description: "将后台遥测监控指标采样周期调整为 500 毫秒 (高频实时)", example: "/interval 500ms" },
        SubcommandSpec { parent_cmd: "/INTERVAL", name: "1S", signature: "/interval 1s", description: "将后台遥测监控指标采样周期调整为 1 秒 (标准默认)", example: "/interval 1s" },
        SubcommandSpec { parent_cmd: "/INTERVAL", name: "2S", signature: "/interval 2s", description: "将后台遥测监控指标采样周期调整为 2 秒 (低负载推荐)", example: "/interval 2s" },
        SubcommandSpec { parent_cmd: "/INTERVAL", name: "5S", signature: "/interval 5s", description: "将后台遥测监控指标采样周期调整为 5 秒 (超低开销)", example: "/interval 5s" },
        SubcommandSpec { parent_cmd: "/INTERVAL", name: "PAUSE", signature: "/interval pause", description: "暂停后台定时遥测数据采集轮询", example: "/interval pause" },
        SubcommandSpec { parent_cmd: "/INTERVAL", name: "RESUME", signature: "/interval resume", description: "恢复后台定时遥测数据采集轮询", example: "/interval resume" },

        // /theme arguments
        SubcommandSpec { parent_cmd: "/THEME", name: "DARK", signature: "/theme dark", description: "切换为深色背景主题 (Dark Theme - 适合深色终端)", example: "/theme dark" },
        SubcommandSpec { parent_cmd: "/THEME", name: "LIGHT", signature: "/theme light", description: "切换为浅色背景主题 (Light Theme - 高对比度适配白色终端)", example: "/theme light" },
        SubcommandSpec { parent_cmd: "/THEME", name: "TOGGLE", signature: "/theme toggle", description: "快速在深色与浅色主题之间来回切换", example: "/theme toggle" },
    ];

    /// Generate suggestion items based on current input buffer, cursor position, and cluster nodes
    pub fn get_suggestions(
        input: &str,
        cursor_pos: usize,
        nodes: &[ClusterNodeInfo],
    ) -> (Vec<SuggestionItem>, (usize, usize)) {
        let text_up_to_cursor = if cursor_pos <= input.len() {
            &input[..cursor_pos]
        } else {
            input
        };

        // Find current token prefix around cursor
        let (word_start, prefix) = match text_up_to_cursor.rfind(char::is_whitespace) {
            Some(idx) => (idx + 1, &text_up_to_cursor[idx + 1..]),
            None => (0, text_up_to_cursor),
        };

        let mut suggestions = Vec::new();

        // 1. Macro triggers: Starts with '/'
        if prefix.starts_with('/') {
            let filter = &prefix[1..].to_lowercase();
            for m in MacroEngine::ALL_MACROS {
                let m_name_sub = &m.name[1..];
                if filter.is_empty() || m_name_sub.starts_with(filter) {
                    suggestions.push(SuggestionItem {
                        kind: SuggestionKind::Macro,
                        completion_text: format!("{} ", m.name),
                        display_title: m.name.to_string(),
                        description: m.description.to_string(),
                        syntax: m.signature.to_string(),
                        example: m.example.to_string(),
                    });
                }
            }
            return (suggestions, (word_start, cursor_pos));
        }

        // 2. Node triggers: Starts with '@'
        if prefix.starts_with('@') {
            let filter = &prefix[1..].to_lowercase();
            for node in nodes {
                if filter.is_empty()
                    || node.id.to_lowercase().contains(filter)
                    || node.address.to_lowercase().contains(filter)
                {
                    suggestions.push(SuggestionItem {
                        kind: SuggestionKind::Node,
                        completion_text: format!("@{} ", node.id),
                        display_title: format!("@{}", node.id),
                        description: format!("Node: {} | Role: {} | Slots: {} | Ping: {:.2}ms", node.address, node.role, node.slots_raw, node.ping_ms),
                        syntax: format!("@{} <command>", node.id),
                        example: format!("@{} INFO memory", node.id),
                    });
                }
            }
            // Add broadcast suggestion
            if filter.is_empty() || "all".starts_with(filter) || "cluster".starts_with(filter) {
                suggestions.push(SuggestionItem {
                    kind: SuggestionKind::Node,
                    completion_text: "@all ".to_string(),
                    display_title: "@all".to_string(),
                    description: "向集群所有 Master 分片并发广播命令并聚合结果".to_string(),
                    syntax: "@all <command>".to_string(),
                    example: "@all /scan order:*".to_string(),
                });
            }
            return (suggestions, (word_start, cursor_pos));
        }

        // 3. Context-aware Analysis: Check if the user is typing subcommands or arguments
        let tokens: Vec<&str> = text_up_to_cursor.split_whitespace().collect();
        let is_typing_new_token = text_up_to_cursor.ends_with(char::is_whitespace);

        let mut effective_tokens = tokens.clone();
        if let Some(first) = effective_tokens.first() {
            if first.starts_with('@') {
                effective_tokens.remove(0);
            }
        }

        // If there is already a parent command (e.g. "INFO", "CLUSTER", "CLIENT", "/interval", "FLUSHALL", "DEBUG", etc.)
        if !effective_tokens.is_empty() && (effective_tokens.len() > 1 || is_typing_new_token) {
            let parent_cmd = effective_tokens[0].to_uppercase();
            let sub_prefix = if is_typing_new_token {
                ""
            } else {
                prefix
            }.to_uppercase();

            for sub in Self::SUBCOMMANDS {
                if sub.parent_cmd == parent_cmd && (sub_prefix.is_empty() || sub.name.starts_with(&sub_prefix)) {
                    suggestions.push(SuggestionItem {
                        kind: SuggestionKind::Subcommand,
                        completion_text: format!("{} ", sub.name.to_lowercase()),
                        display_title: format!("{} {}", sub.parent_cmd, sub.name),
                        description: sub.description.to_string(),
                        syntax: sub.signature.to_string(),
                        example: sub.example.to_string(),
                    });
                }
            }

            if !suggestions.is_empty() {
                return (suggestions, (word_start, cursor_pos));
            }
        }

        // 4. Root Command suggestions (when typing the first command word)
        let upper_prefix = prefix.to_uppercase();
        if !upper_prefix.is_empty() && (effective_tokens.is_empty() || (effective_tokens.len() == 1 && !is_typing_new_token)) {
            for spec in Self::REDIS_COMMANDS {
                if spec.name.starts_with(&upper_prefix) {
                    suggestions.push(SuggestionItem {
                        kind: SuggestionKind::Command,
                        completion_text: format!("{} ", spec.name),
                        display_title: spec.name.to_string(),
                        description: spec.description.to_string(),
                        syntax: spec.signature.to_string(),
                        example: spec.example.to_string(),
                    });
                }
            }
        }

        (suggestions, (word_start, cursor_pos))
    }
}
