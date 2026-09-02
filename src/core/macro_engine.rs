use crate::backend::formatter::FormattedValue;

pub struct MacroEngine;

#[derive(Debug, Clone)]
pub struct MacroDescriptor {
    pub name: &'static str,
    pub signature: &'static str,
    pub description: &'static str,
    #[allow(dead_code)]
    pub example: &'static str,
}

impl MacroEngine {
    pub const ALL_MACROS: &'static [MacroDescriptor] = &[
        MacroDescriptor {
            name: "/scan",
            signature: "/scan [pattern] [count]",
            description: "非阻塞安全游标扫描 (替代高危 KEYS *)",
            example: "/scan order:* 50",
        },
        MacroDescriptor {
            name: "/bigkeys",
            signature: "/bigkeys [count]",
            description: "大 Key 探测与内存占用排行分析",
            example: "/bigkeys 10",
        },
        MacroDescriptor {
            name: "/slowlog",
            signature: "/slowlog [count]",
            description: "拉取慢查询日志并附带排障与优化建议",
            example: "/slowlog 10",
        },
        MacroDescriptor {
            name: "/interval",
            signature: "/interval [ms|s|pause]",
            description: "动态调整后台遥测指标采样频率 (如 500ms, 1s, 2s, pause)",
            example: "/interval 500ms",
        },
        MacroDescriptor {
            name: "/settings",
            signature: "/settings",
            description: "查看与配置当前运行时参数与连接信息",
            example: "/settings",
        },
        MacroDescriptor {
            name: "/clients",
            signature: "/clients",
            description: "查看活跃客户端连接状态与空闲耗时",
            example: "/clients",
        },
        MacroDescriptor {
            name: "/theme",
            signature: "/theme [dark|light|toggle]",
            description: "切换界面深色 / 浅色配色主题 (Dark / Light Theme)",
            example: "/theme light",
        },
        MacroDescriptor {
            name: "/help",
            signature: "/help",
            description: "查看全部快捷宏与键盘快捷键指南",
            example: "/help",
        },
        MacroDescriptor {
            name: "/clear",
            signature: "/clear",
            description: "清空当前交互流窗口 (等同于 Ctrl+L)",
            example: "/clear",
        },
    ];

    pub fn get_help_value() -> FormattedValue {
        let headers = vec![
            "Macro / Command".to_string(),
            "Syntax".to_string(),
            "Description".to_string(),
        ];
        let mut rows = Vec::new();
        for m in Self::ALL_MACROS {
            rows.push(vec![
                m.name.to_string(),
                m.signature.to_string(),
                m.description.to_string(),
            ]);
        }
        rows.push(vec![
            "@<node_id>".to_string(),
            "@node-1 <cmd>".to_string(),
            "定向派发指令至指定集群节点".to_string(),
        ]);
        rows.push(vec![
            "Tab".to_string(),
            "Press Tab".to_string(),
            "在左侧交互流与右侧多维看板间切换焦点".to_string(),
        ]);
        rows.push(vec![
            "F5 / Ctrl+B".to_string(),
            "Press F5 or Ctrl+B".to_string(),
            "循环切换 4 种分栏布局预设 (Balanced/Focus/Monitor/Zen)".to_string(),
        ]);
        rows.push(vec![
            "F2 / F3 / F4".to_string(),
            "Press F2/F3/F4".to_string(),
            "快速切换右侧看板: 监控概览 / 集群拓扑 / 慢查排障".to_string(),
        ]);

        FormattedValue::Table { headers, rows }
    }

    pub fn get_mock_bigkeys(count: usize) -> FormattedValue {
        let headers = vec![
            "Rank".to_string(),
            "Key".to_string(),
            "Type".to_string(),
            "Est. Memory".to_string(),
            "Elements".to_string(),
        ];
        let mut rows = vec![
            vec![
                "1".to_string(),
                "group:channel:all_members".to_string(),
                "SET".to_string(),
                "14.82 MB".to_string(),
                "142,500 items".to_string(),
            ],
            vec![
                "2".to_string(),
                "cache:hot:products_catalog".to_string(),
                "HASH".to_string(),
                "8.40 MB".to_string(),
                "35,000 fields".to_string(),
            ],
            vec![
                "3".to_string(),
                "stream:events:user_activity".to_string(),
                "STREAM".to_string(),
                "6.15 MB".to_string(),
                "50,000 entries".to_string(),
            ],
            vec![
                "4".to_string(),
                "leaderboard:global:2026".to_string(),
                "ZSET".to_string(),
                "3.90 MB".to_string(),
                "88,000 members".to_string(),
            ],
            vec![
                "5".to_string(),
                "session:payload:admin_bundle".to_string(),
                "STRING".to_string(),
                "1.25 MB".to_string(),
                "1,310,720 bytes".to_string(),
            ],
        ];

        rows.truncate(count.max(1));
        FormattedValue::Table { headers, rows }
    }

    pub fn get_mock_clients() -> FormattedValue {
        let headers = vec![
            "ID".to_string(),
            "Address".to_string(),
            "Age (s)".to_string(),
            "Idle (s)".to_string(),
            "Flags".to_string(),
            "Last Command".to_string(),
        ];
        let rows = vec![
            vec![
                "1042".to_string(),
                "127.0.0.1:52180".to_string(),
                "142".to_string(),
                "0".to_string(),
                "N".to_string(),
                "HGETALL".to_string(),
            ],
            vec![
                "1043".to_string(),
                "192.168.1.104:49152".to_string(),
                "3600".to_string(),
                "1".to_string(),
                "N".to_string(),
                "GET".to_string(),
            ],
            vec![
                "1044".to_string(),
                "192.168.1.108:58231".to_string(),
                "820".to_string(),
                "45".to_string(),
                "N".to_string(),
                "PING".to_string(),
            ],
            vec![
                "1045".to_string(),
                "10.0.4.15:33211".to_string(),
                "7200".to_string(),
                "120".to_string(),
                "P (PubSub)".to_string(),
                "PSUBSCRIBE".to_string(),
            ],
        ];
        FormattedValue::Table { headers, rows }
    }

    pub fn format_interval_result(new_interval_ms: u64, is_paused: bool) -> FormattedValue {
        let headers = vec![
            "Setting".to_string(),
            "Current Value".to_string(),
            "Status".to_string(),
            "Presets".to_string(),
        ];

        let (val_str, status_str) = if is_paused || new_interval_ms == 0 {
            ("0 ms (Paused)".to_string(), "[PAUSED] Background poller suspended".to_string())
        } else if new_interval_ms >= 1000 && new_interval_ms % 1000 == 0 {
            (format!("{}s ({}ms)", new_interval_ms / 1000, new_interval_ms), "[ACTIVE] Live sampling active".to_string())
        } else {
            (format!("{}ms", new_interval_ms), "[ACTIVE] Live sampling active".to_string())
        };

        let rows = vec![
            vec![
                "Telemetry Polling Interval".to_string(),
                val_str,
                status_str,
                "250ms | 500ms | 1s | 2s | 5s | pause".to_string(),
            ],
        ];

        FormattedValue::Table { headers, rows }
    }

    pub fn format_theme_result(old_theme: &str, new_theme: &str) -> FormattedValue {
        let headers = vec![
            "Setting".to_string(),
            "Current Theme".to_string(),
            "Status".to_string(),
            "Available Themes".to_string(),
        ];

        let status_str = if old_theme.eq_ignore_ascii_case(new_theme) {
            format!("[ACTIVE] Theme already set to {}", new_theme)
        } else {
            format!("[SWITCHED] Switched from {} to {}", old_theme, new_theme)
        };

        let rows = vec![
            vec![
                "UI Color Theme".to_string(),
                new_theme.to_string(),
                status_str,
                "dark | light | toggle (/theme [mode])".to_string(),
            ],
        ];

        FormattedValue::Table { headers, rows }
    }

    pub fn format_settings(
        host: &str,
        port: u16,
        is_cluster: bool,
        layout_name: &str,
        theme_name: &str,
        interval_ms: u64,
        is_paused: bool,
    ) -> FormattedValue {
        let headers = vec![
            "Configuration Item".to_string(),
            "Current Value".to_string(),
            "Description".to_string(),
        ];

        let mode_str = if is_cluster { "Cluster Mode" } else { "Standalone / Sentinel" };
        let poll_str = if is_paused || interval_ms == 0 {
            "Paused (0ms)".to_string()
        } else if interval_ms >= 1000 && interval_ms % 1000 == 0 {
            format!("{}s ({}ms)", interval_ms / 1000, interval_ms)
        } else {
            format!("{}ms", interval_ms)
        };

        let rows = vec![
            vec!["Server Address".to_string(), format!("{}:{}", host, port), "Target Redis connection endpoint".to_string()],
            vec!["Protocol Mode".to_string(), mode_str.to_string(), "Standalone single-node or distributed Cluster topology".to_string()],
            vec!["Layout Preset".to_string(), layout_name.to_string(), "Current viewport split ratio (toggle via F5 / Ctrl+B)".to_string()],
            vec!["UI Color Theme".to_string(), theme_name.to_string(), "Active color palette: Dark / Light (/theme [mode])".to_string()],
            vec!["Telemetry Polling".to_string(), poll_str, "Sampling interval for QPS, CPU, Memory, Slowlog (/interval [val])".to_string()],
            vec!["Config Path".to_string(), "~/.config/xedis/config.toml".to_string(), "Persistent configuration file on local machine".to_string()],
        ];

        FormattedValue::Table { headers, rows }
    }

    pub fn suggest_for_slow_command(cmd: &str) -> Option<String> {
        let upper = cmd.to_uppercase();
        if upper.starts_with("KEYS") {
            Some("生产环境严禁使用 KEYS 命令，会导致 Redis 线程全局阻塞，建议改用 /scan 或 SCAN 游标分批拉取。".to_string())
        } else if upper.starts_with("SMEMBERS") || upper.starts_with("HGETALL") {
            Some("全量读取大集合/哈希极易引发单次高耗时，建议改用 SSCAN / HSCAN 分批读取或进行分片拆分。".to_string())
        } else if upper.starts_with("FLUSHALL") || upper.starts_with("FLUSHDB") {
            Some("全库清空为高危耗时指令，若必须清理建议异步执行 FLUSHALL ASYNC。".to_string())
        } else if upper.starts_with("MGET") || upper.starts_with("MSET") {
            Some("批量 Key 过多时网络传输与解析耗时大，建议单次批处理控制在 50~100 个以内。".to_string())
        } else if upper.starts_with("EVAL") || upper.starts_with("EVALSHA") {
            Some("Lua 脚本执行时间过长会阻塞整个实例，请检查脚本内循环与大范围遍历逻辑。".to_string())
        } else if upper.starts_with("ZRANGEBYSCORE") || upper.starts_with("ZRANGE") {
            Some("ZSET 范围查询返回过多元素，建议增加 LIMIT offset count 分页截断。".to_string())
        } else if upper.starts_with("SORT") {
            Some("SORT 命令计算复杂度高（O(N+M*log(M))），建议在应用层排序或改用 ZSET。".to_string())
        } else {
            None
        }
    }
}
