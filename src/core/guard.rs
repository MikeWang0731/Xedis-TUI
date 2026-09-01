use crate::core::router::{CommandType, ParsedCommand};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Level 3: Absolute Blocking / High Danger (FLUSHALL, SHUTDOWN, KEYS *, CONFIG REWRITE, etc.)
    Level3Blocking,
    /// Level 2: Caution & Warning (SMEMBERS, HGETALL on huge structures, SLAVEOF, SWAPDB, etc.)
    Level2Warning,
}

impl RiskLevel {
    pub fn badge_title(&self) -> &'static str {
        match self {
            RiskLevel::Level3Blocking => "HIGH RISK BLOCKED - LEVEL 3",
            RiskLevel::Level2Warning => "RISK WARNING - LEVEL 2",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DangerAssessment {
    pub command_str: String,
    pub level: RiskLevel,
    pub title: String,
    pub reason: String,
    pub suggestion: String,
}

pub struct SafetyGuard;

impl SafetyGuard {
    /// Inspect a parsed command to see if it triggers novice safety guard rules
    pub fn inspect(parsed: &ParsedCommand) -> Option<DangerAssessment> {
        match &parsed.command_type {
            CommandType::Native { cmd, args } => {
                let upper_cmd = cmd.to_uppercase();

                // 1. FLUSHALL / FLUSHDB
                if upper_cmd == "FLUSHALL" || upper_cmd == "FLUSHDB" {
                    return Some(DangerAssessment {
                        command_str: parsed.raw_input.clone(),
                        level: RiskLevel::Level3Blocking,
                        title: "全库数据清空操作 (FLUSHALL / FLUSHDB)".to_string(),
                        reason: "该指令会彻底清空当前或全部数据库的所有键值数据，且操作不可逆，在生产环境会造成灾难性数据丢失。".to_string(),
                        suggestion: "如仅需清理测试数据，请使用 DEL 单独删除指定测试 Key；若必须全量清空且数据量巨大，推荐在低峰期使用 FLUSHALL ASYNC 避免主线程阻塞。".to_string(),
                    });
                }

                // 2. SHUTDOWN
                if upper_cmd == "SHUTDOWN" {
                    return Some(DangerAssessment {
                        command_str: parsed.raw_input.clone(),
                        level: RiskLevel::Level3Blocking,
                        title: "关闭 Redis 服务器 (SHUTDOWN)".to_string(),
                        reason: "该指令将立即终止 Redis 服务进程并断开所有外部客户端连接，导致线上业务不可用或触发集群故障转移。".to_string(),
                        suggestion: "非运维下线场景严禁在交互终端执行 SHUTDOWN；如需检查服务状态，请使用 PING 或 INFO server。".to_string(),
                    });
                }

                // 3. KEYS * or KEYS with wildcards
                if upper_cmd == "KEYS" {
                    let pattern = args.first().map(|s| s.as_str()).unwrap_or("*");
                    return Some(DangerAssessment {
                        command_str: parsed.raw_input.clone(),
                        level: RiskLevel::Level3Blocking,
                        title: "全量键遍历阻断 (KEYS pattern)".to_string(),
                        reason: format!(
                            "KEYS '{}' 命令时间复杂度为 O(N)，在百万级以上数据量下会长时间阻塞 Redis 单线程事件循环，导致其他请求超时雪崩。",
                            pattern
                        ),
                        suggestion: "强烈推荐使用内置非阻塞宏 '/scan' 或原生 'SCAN 0 MATCH <pattern> COUNT 50' 进行游标分批拉取。".to_string(),
                    });
                }

                // 4. CONFIG REWRITE / CONFIG SET dangerous parameters
                if upper_cmd == "CONFIG" {
                    if let Some(sub) = args.first() {
                        let sub_upper = sub.to_uppercase();
                        if sub_upper == "REWRITE" {
                            return Some(DangerAssessment {
                                command_str: parsed.raw_input.clone(),
                                level: RiskLevel::Level3Blocking,
                                title: "持久化改写配置文件 (CONFIG REWRITE)".to_string(),
                                reason: "该指令会扫描当前内存配置并直接重写物理 redis.conf 文件，可能覆盖注释或导致配置漂移不一致。".to_string(),
                                suggestion: "如需查看当前生效配置，请使用 'CONFIG GET <parameter>'；非必要请勿直接改写磁盘配置文件。".to_string(),
                            });
                        } else if sub_upper == "SET" {
                            if let Some(param) = args.get(1) {
                                let param_lower = param.to_lowercase();
                                if param_lower == "requirepass" || param_lower == "masterauth" || param_lower == "dir" || param_lower == "dbfilename" {
                                    return Some(DangerAssessment {
                                        command_str: parsed.raw_input.clone(),
                                        level: RiskLevel::Level2Warning,
                                        title: "动态修改核心运行时参数 (CONFIG SET)".to_string(),
                                        reason: format!("修改敏感参数 '{}' 可能导致客户端认证失效或影响持久化文件落盘路径。", param),
                                        suggestion: "请确保已在测试环境中充分验证，或改由基础设施运维配置流水线统一管理。".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }

                // 5. DEBUG SEGFAULT / DEBUG SLEEP
                if upper_cmd == "DEBUG" {
                    if let Some(sub) = args.first() {
                        let sub_upper = sub.to_uppercase();
                        if sub_upper == "SEGFAULT" || sub_upper == "CRASH-AND-RECOVER" || sub_upper == "RESTART-SERVER" {
                            return Some(DangerAssessment {
                                command_str: parsed.raw_input.clone(),
                                level: RiskLevel::Level3Blocking,
                                title: "调试崩溃与进程异常退出 (DEBUG)".to_string(),
                                reason: "DEBUG 子命令会直接使 Redis 进程发生段错误（Segmentation fault）或强制退出崩溃。".to_string(),
                                suggestion: "生产与测试环境严禁执行 DEBUG 崩溃指令！".to_string(),
                            });
                        } else if sub_upper == "SLEEP" {
                            return Some(DangerAssessment {
                                command_str: parsed.raw_input.clone(),
                                level: RiskLevel::Level3Blocking,
                                title: "主线程强制休眠 (DEBUG SLEEP)".to_string(),
                                reason: "该指令会导致 Redis 主线程完全挂起指定秒数，期间无法处理任何读写请求。".to_string(),
                                suggestion: "如需模拟网络延迟，请在客户端或代理层进行限流/延迟模拟。".to_string(),
                            });
                        }
                    }
                }

                // 6. BGSAVE / BGREWRITEAOF (Fork overhead)
                if upper_cmd == "BGSAVE" || upper_cmd == "BGREWRITEAOF" {
                    return Some(DangerAssessment {
                        command_str: parsed.raw_input.clone(),
                        level: RiskLevel::Level2Warning,
                        title: "手动触发后台持久化重写 (BGSAVE / BGREWRITEAOF)".to_string(),
                        reason: "Redis 在 fork 子进程生成快照时可能引起瞬时内存页复制（Copy-On-Write）翻倍及 CPU 飙升。".to_string(),
                        suggestion: "Redis 默认具备自动快照策略，非必要无需手动触发；请检查 INFO persistence 查看当前快照进度。".to_string(),
                    });
                }

                // 7. SLAVEOF NO ONE / REPLICAOF NO ONE
                if upper_cmd == "SLAVEOF" || upper_cmd == "REPLICAOF" {
                    if let Some(arg1) = args.first() {
                        if arg1.to_uppercase() == "NO" && args.get(1).map(|s| s.to_uppercase()) == Some("ONE".to_string()) {
                            return Some(DangerAssessment {
                                command_str: parsed.raw_input.clone(),
                                level: RiskLevel::Level2Warning,
                                title: "脱离主从复制集群 (REPLICAOF NO ONE)".to_string(),
                                reason: "该指令会使当前从节点立即停止复制并提升为独立 Master 节点，可能导致数据分叉与拓扑裂脑。".to_string(),
                                suggestion: "如需集群故障转移，请在对应节点使用 'CLUSTER FAILOVER' 进行平滑切换。".to_string(),
                            });
                        }
                    }
                }

                // 8. SWAPDB / MIGRATE
                if upper_cmd == "SWAPDB" {
                    return Some(DangerAssessment {
                        command_str: parsed.raw_input.clone(),
                        level: RiskLevel::Level2Warning,
                        title: "交换数据库编号映射 (SWAPDB)".to_string(),
                        reason: "原子交换两个数据库的 Key 空间，将瞬间变更所有客户端查询路由目标。".to_string(),
                        suggestion: "请确保所有业务客户端均已知晓数据库切换，防止读写错库。".to_string(),
                    });
                }

                if upper_cmd == "MIGRATE" {
                    return Some(DangerAssessment {
                        command_str: parsed.raw_input.clone(),
                        level: RiskLevel::Level2Warning,
                        title: "跨实例键迁移 (MIGRATE)".to_string(),
                        reason: "MIGRATE 在网络传输期间会锁定两端实例的 Key，若数据量较大可能引起阻塞。".to_string(),
                        suggestion: "建议指定合理的 TIMEOUT 参数，或在低峰期分批迁移。".to_string(),
                    });
                }

                // 9. SMEMBERS
                if upper_cmd == "SMEMBERS" {
                    let key_name = args.first().map(|s| s.as_str()).unwrap_or("key");
                    return Some(DangerAssessment {
                        command_str: parsed.raw_input.clone(),
                        level: RiskLevel::Level2Warning,
                        title: "全量集合读取 (SMEMBERS)".to_string(),
                        reason: format!("若集合 '{}' 包含数万以上成员，全量 SMEMBERS 会导致大网络包传输与单线程延迟上升。", key_name),
                        suggestion: "对于大集合，推荐使用 'SSCAN <key> 0 COUNT 50' 分批读取，或先用 'SCARD <key>' 查看元素总数。".to_string(),
                    });
                }

                None
            }
            CommandType::Macro { .. } => None,
        }
    }
}
