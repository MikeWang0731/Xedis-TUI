use crate::backend::cluster_info::{ClusterNode, ClusterTopology, ClusterTopologyParser};
use crate::backend::formatter::FormattedValue;
use crate::core::macro_engine::MacroEngine;
use crate::core::telemetry::{MetricsHistory, TelemetryMetrics, TelemetryParser};
use crate::ui::slowlog_view::SlowlogEntry;
use chrono::Local;
use std::time::{Duration, Instant};

pub type ClusterNodeInfo = ClusterNode;

#[derive(Debug, Clone)]
pub struct TelemetryData {
    pub connected: bool,
    pub server_desc: String,
    pub is_cluster: bool,
    pub metrics: TelemetryMetrics,
    pub history: MetricsHistory,
    pub topology: ClusterTopology,
    pub slowlogs: Vec<SlowlogEntry>,
    pub last_poll_time: Option<Instant>,
}

impl Default for TelemetryData {
    fn default() -> Self {
        let metrics = TelemetryMetrics::default();
        let history = MetricsHistory::new(60);
        let topology = ClusterTopology::mock_cluster_topology();
        let slowlogs = vec![
            SlowlogEntry {
                id: 1,
                timestamp: "14:18:22".to_string(),
                latency_ms: 82.4,
                command: "KEYS report:user:daily:*".to_string(),
                node: "node-2".to_string(),
                suggestion: MacroEngine::suggest_for_slow_command("KEYS"),
            },
            SlowlogEntry {
                id: 2,
                timestamp: "14:15:09".to_string(),
                latency_ms: 28.1,
                command: "SMEMBERS group:channel:all_members".to_string(),
                node: "node-1".to_string(),
                suggestion: MacroEngine::suggest_for_slow_command("SMEMBERS"),
            },
            SlowlogEntry {
                id: 3,
                timestamp: "14:11:45".to_string(),
                latency_ms: 16.8,
                command: "HGETALL cache:hot:products_catalog".to_string(),
                node: "node-3".to_string(),
                suggestion: MacroEngine::suggest_for_slow_command("HGETALL"),
            },
        ];

        Self {
            connected: false,
            server_desc: "Connecting...".to_string(),
            is_cluster: true,
            metrics,
            history,
            topology,
            slowlogs,
            last_poll_time: None,
        }
    }
}

#[allow(dead_code)]
impl TelemetryData {
    // Accessors for convenience and backward compatibility
    pub fn version(&self) -> &str {
        &self.metrics.version
    }

    pub fn ping_latency_ms(&self) -> f64 {
        self.metrics.ping_latency_ms
    }

    pub fn used_memory_human(&self) -> &str {
        &self.metrics.used_memory_human
    }

    pub fn used_memory_bytes(&self) -> u64 {
        self.metrics.used_memory_bytes.unwrap_or(0)
    }

    pub fn max_memory_bytes(&self) -> u64 {
        self.metrics.max_memory_bytes.unwrap_or(0)
    }

    pub fn max_memory_human(&self) -> &str {
        &self.metrics.max_memory_human
    }

    pub fn mem_fragmentation_ratio(&self) -> f64 {
        self.metrics.mem_fragmentation_ratio.unwrap_or(1.0)
    }

    pub fn instantaneous_ops_per_sec(&self) -> u64 {
        self.metrics.instantaneous_ops_per_sec.unwrap_or(0)
    }

    pub fn cpu_usage_pct(&self) -> f64 {
        self.metrics.cpu_usage_pct
    }

    pub fn connected_clients(&self) -> u64 {
        self.metrics.connected_clients
    }

    pub fn total_keys(&self) -> u64 {
        self.metrics.total_keys
    }

    pub fn hit_rate_pct(&self) -> f64 {
        self.metrics.hit_rate_pct.unwrap_or(0.0)
    }

    pub fn nodes(&self) -> Vec<ClusterNode> {
        let mut list = Vec::new();
        for shard in &self.topology.shards {
            list.push(shard.master.clone());
            list.extend(shard.replicas.clone());
        }
        if list.is_empty() {
            list.extend(self.topology.standalone_nodes.clone());
        }
        list
    }
}

pub enum XedisBackend {
    Live(redis::aio::MultiplexedConnection),
    LiveCluster(redis::cluster_async::ClusterConnection),
    DemoMock,
}

pub struct XedisClient {
    #[allow(dead_code)]
    pub target_url: String,
    pub backend: XedisBackend,
    pub telemetry: TelemetryData,
}

impl XedisClient {
    pub async fn connect(url: &str, is_cluster: bool) -> Self {
        if is_cluster {
            if let Ok(client) = redis::cluster::ClusterClient::new(vec![url]) {
                if let Ok(conn) = client.get_async_connection().await {
                    let mut client_obj = Self {
                        target_url: url.to_string(),
                        backend: XedisBackend::LiveCluster(conn),
                        telemetry: TelemetryData {
                            connected: true,
                            is_cluster: true,
                            server_desc: url.to_string(),
                            metrics: TelemetryMetrics::empty(),
                            history: MetricsHistory::new_empty(60),
                            topology: ClusterTopology::default(),
                            slowlogs: Vec::new(),
                            last_poll_time: None,
                        },
                    };
                    client_obj.poll_telemetry().await;
                    return client_obj;
                }
            }
        } else if let Ok(client) = redis::Client::open(url) {
            if let Ok(conn) = client.get_multiplexed_tokio_connection().await {
                let mut client_obj = Self {
                    target_url: url.to_string(),
                    backend: XedisBackend::Live(conn),
                    telemetry: TelemetryData {
                        connected: true,
                        is_cluster: false,
                        server_desc: url.to_string(),
                        metrics: TelemetryMetrics::empty(),
                        history: MetricsHistory::new_empty(60),
                        topology: ClusterTopology::default(),
                        slowlogs: Vec::new(),
                        last_poll_time: None,
                    },
                };
                client_obj.poll_telemetry().await;
                return client_obj;
            }
        }

        // Graceful fallback to Demo / Offline mock mode for immediate UI preview when no server is online
        let mut telemetry = TelemetryData::default();
        telemetry.connected = false;
        telemetry.server_desc = format!("{} [OFFLINE / DEMO PREVIEW]", url);
        Self {
            target_url: url.to_string(),
            backend: XedisBackend::DemoMock,
            telemetry,
        }
    }

    pub async fn execute_command(&mut self, _target_node: Option<&str>, cmd: &str, args: &[String]) -> (FormattedValue, Duration) {
        let start = Instant::now();

        match &mut self.backend {
            XedisBackend::Live(conn) => {
                let mut command = redis::cmd(cmd);
                for arg in args {
                    command.arg(arg);
                }
                match command.query_async::<redis::Value>(conn).await {
                    Ok(val) => (FormattedValue::from_redis_value(val), start.elapsed()),
                    Err(e) => (FormattedValue::Error(format!("ERR {}", e)), start.elapsed()),
                }
            }
            XedisBackend::LiveCluster(conn) => {
                let mut command = redis::cmd(cmd);
                for arg in args {
                    command.arg(arg);
                }
                match command.query_async::<redis::Value>(conn).await {
                    Ok(val) => (FormattedValue::from_redis_value(val), start.elapsed()),
                    Err(e) => (FormattedValue::Error(format!("ERR Cluster: {}", e)), start.elapsed()),
                }
            }
            XedisBackend::DemoMock => {
                tokio::time::sleep(Duration::from_millis(1)).await;
                let elapsed = start.elapsed();
                let upper_cmd = cmd.to_uppercase();

                let res = match upper_cmd.as_str() {
                    "PING" => FormattedValue::Status("PONG".to_string()),
                    "GET" => {
                        let key = args.first().map(|s| s.as_str()).unwrap_or("key");
                        FormattedValue::String(format!("val_for_{}", key))
                    }
                    "SET" => FormattedValue::Status("OK".to_string()),
                    "DEL" => FormattedValue::Integer(1),
                    "EXISTS" => FormattedValue::Integer(1),
                    "TTL" => FormattedValue::Integer(3600),
                    "TYPE" => FormattedValue::Status("string".to_string()),
                    "HGETALL" => FormattedValue::Table {
                        headers: vec!["Field".to_string(), "Value".to_string()],
                        rows: vec![
                            vec!["user_id".to_string(), "\"usr_88234\"".to_string()],
                            vec!["role".to_string(), "\"admin\"".to_string()],
                            vec!["login_ip".to_string(), "\"192.168.1.104\"".to_string()],
                            vec!["ttl".to_string(), "3600 (1h)".to_string()],
                        ],
                    },
                    "LRANGE" => FormattedValue::List(vec![
                        "job:task:001".to_string(),
                        "job:task:002".to_string(),
                        "job:task:003".to_string(),
                    ]),
                    "SMEMBERS" => FormattedValue::List(vec![
                        "admin".to_string(),
                        "developer".to_string(),
                        "tester".to_string(),
                    ]),
                    "SCAN" => FormattedValue::Json(
                        serde_json::to_string_pretty(&serde_json::json!([
                            "order:20260831001",
                            "order:20260831002",
                            "order:20260831003"
                        ]))
                        .unwrap_or_default(),
                    ),
                    "INFO" => FormattedValue::String(
                        "# Server\nredis_version:7.2.4\nredis_mode:cluster\nos:Darwin\n\n# Memory\nused_memory_human:2.14G\nmaxmemory_human:4.00G".to_string(),
                    ),
                    "CLUSTER" => {
                        if args.first().map(|s| s.to_uppercase()).as_deref() == Some("NODES") {
                            FormattedValue::String("node-1 127.0.0.1:6379@16379 master - 0 1788157290000 1 connected 0-5460\nnode-2 127.0.0.1:6380@16380 master - 0 1788157290000 2 connected 5461-10922\nnode-3 127.0.0.1:6381@16381 master - 0 1788157290000 3 connected 10923-16383".to_string())
                        } else {
                            FormattedValue::Status("OK".to_string())
                        }
                    }
                    _ => FormattedValue::Status(format!("OK (Simulated response for {})", cmd)),
                };
                (res, elapsed)
            }
        }
    }

    pub async fn execute_macro(
        &mut self,
        _target_node: Option<&str>,
        macro_name: &str,
        args: &[String],
    ) -> (FormattedValue, Duration) {
        let start = Instant::now();
        let name = macro_name.to_lowercase();

        match name.as_str() {
            "/scan" => {
                let pattern = args.first().cloned().unwrap_or_else(|| "*".to_string());
                let count: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);

                if let XedisBackend::Live(conn) = &mut self.backend {
                    let mut cursor: u64 = 0;
                    let mut all_keys = Vec::new();
                    loop {
                        let mut cmd = redis::cmd("SCAN");
                        cmd.arg(cursor).arg("MATCH").arg(&pattern).arg("COUNT").arg(count.min(100));
                        if let Ok((next_cursor, keys)) = cmd.query_async::<(u64, Vec<String>)>(conn).await {
                            all_keys.extend(keys);
                            cursor = next_cursor;
                            if cursor == 0 || all_keys.len() >= count {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    let json_val = serde_json::to_string_pretty(&all_keys).unwrap_or_default();
                    (FormattedValue::Json(json_val), start.elapsed())
                } else {
                    // Demo Mock response
                    tokio::time::sleep(Duration::from_millis(2)).await;
                    let mock_keys = vec![
                        format!("{}:001", pattern.trim_end_matches('*')),
                        format!("{}:002", pattern.trim_end_matches('*')),
                        format!("{}:003", pattern.trim_end_matches('*')),
                        format!("{}:004", pattern.trim_end_matches('*')),
                        format!("{}:005", pattern.trim_end_matches('*')),
                    ];
                    let json_val = serde_json::to_string_pretty(&mock_keys).unwrap_or_default();
                    (FormattedValue::Json(json_val), start.elapsed())
                }
            }
            "/bigkeys" => {
                let count: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(5);
                tokio::time::sleep(Duration::from_millis(3)).await;
                (MacroEngine::get_mock_bigkeys(count), start.elapsed())
            }
            "/slowlog" => {
                let count: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(5);
                let headers = vec![
                    "ID".to_string(),
                    "Latency (ms)".to_string(),
                    "Slow Command".to_string(),
                    "Node".to_string(),
                    "Optimization Guidance".to_string(),
                ];
                let mut rows = Vec::new();
                for entry in self.telemetry.slowlogs.iter().take(count.max(1)) {
                    rows.push(vec![
                        format!("#{}", entry.id),
                        format!("{:.1} ms", entry.latency_ms),
                        entry.command.clone(),
                        entry.node.clone(),
                        entry.suggestion.clone().unwrap_or_else(|| "Consider caching or query pagination".to_string()),
                    ]);
                }
                if rows.is_empty() {
                    rows.push(vec![
                        "-".to_string(),
                        "0.0 ms".to_string(),
                        "No slow queries detected (>10ms)".to_string(),
                        "-".to_string(),
                        "Instance is running smoothly".to_string(),
                    ]);
                }
                (FormattedValue::Table { headers, rows }, start.elapsed())
            }
            "/clients" => {
                tokio::time::sleep(Duration::from_millis(2)).await;
                (MacroEngine::get_mock_clients(), start.elapsed())
            }
            "/help" | "/macros" => {
                (MacroEngine::get_help_value(), start.elapsed())
            }
            "/clear" => {
                (FormattedValue::Status("Command stream cleared".to_string()), start.elapsed())
            }
            _ => {
                (
                    FormattedValue::Error(format!("Unknown macro: {}. Enter /help to see all available macros.", macro_name)),
                    start.elapsed(),
                )
            }
        }
    }

    pub async fn poll_telemetry(&mut self) {
        let now = Instant::now();
        let elapsed_sec = self
            .telemetry
            .last_poll_time
            .map(|t| now.duration_since(t).as_secs_f64())
            .unwrap_or(1.0)
            .max(0.1);
        self.telemetry.last_poll_time = Some(now);

        match &mut self.backend {
            XedisBackend::Live(conn) => {
                let ping_start = Instant::now();
                if let Ok(pong) = redis::cmd("PING").query_async::<String>(conn).await {
                    if pong == "PONG" {
                        self.telemetry.metrics.ping_latency_ms = ping_start.elapsed().as_micros() as f64 / 1000.0;
                    }
                }

                // 1. Robust multi-stage INFO collection (supports BulkString, Array, Status, and sub-sections)
                let mut collected_info = String::new();
                if let Ok(val) = redis::cmd("INFO").query_async::<redis::Value>(conn).await {
                    collected_info = Self::redis_value_to_string(&val);
                }
                if collected_info.trim().is_empty() {
                    if let Ok(val) = redis::cmd("INFO").arg("ALL").query_async::<redis::Value>(conn).await {
                        collected_info = Self::redis_value_to_string(&val);
                    }
                }
                if collected_info.trim().is_empty() {
                    for sec in &["server", "memory", "cpu", "stats", "clients", "keyspace"] {
                        if let Ok(val) = redis::cmd("INFO").arg(sec).query_async::<redis::Value>(conn).await {
                            let s = Self::redis_value_to_string(&val);
                            if !s.is_empty() {
                                collected_info.push_str(&s);
                                collected_info.push('\n');
                            }
                        }
                    }
                }

                if !collected_info.is_empty() {
                    self.telemetry.metrics = TelemetryParser::parse_info(
                        &collected_info,
                        Some(&self.telemetry.metrics),
                        elapsed_sec,
                    );
                }

                // Try DBSIZE if keys wasn't in INFO
                if self.telemetry.metrics.total_keys == 0 {
                    if let Ok(cnt) = redis::cmd("DBSIZE").query_async::<u64>(conn).await {
                        self.telemetry.metrics.total_keys = cnt;
                    }
                }

                if let Ok(repl_str) = redis::cmd("INFO").arg("replication").query_async::<String>(conn).await {
                    self.telemetry.topology.replication = Some(ClusterTopologyParser::parse_info_replication(&repl_str));
                }
                if let Ok(slowlog_val) = redis::cmd("SLOWLOG").arg("GET").arg(20).query_async::<redis::Value>(conn).await {
                    let parsed = Self::parse_slowlog_redis_value(&slowlog_val, "standalone");
                    if !parsed.is_empty() {
                        self.telemetry.slowlogs = parsed;
                    }
                }

                // Push into rolling history
                self.telemetry.history.push(
                    self.telemetry.metrics.instantaneous_ops_per_sec.unwrap_or(0),
                    self.telemetry.metrics.cpu_usage_pct,
                );
            }
            XedisBackend::LiveCluster(conn) => {
                let ping_start = Instant::now();
                if let Ok(pong) = redis::cmd("PING").query_async::<String>(conn).await {
                    if pong == "PONG" {
                        self.telemetry.metrics.ping_latency_ms = ping_start.elapsed().as_micros() as f64 / 1000.0;
                    }
                }

                let mut collected_info = String::new();
                if let Ok(val) = redis::cmd("INFO").query_async::<redis::Value>(conn).await {
                    collected_info = Self::redis_value_to_string(&val);
                }
                if collected_info.trim().is_empty() {
                    if let Ok(val) = redis::cmd("INFO").arg("ALL").query_async::<redis::Value>(conn).await {
                        collected_info = Self::redis_value_to_string(&val);
                    }
                }
                if collected_info.trim().is_empty() {
                    for sec in &["server", "memory", "cpu", "stats", "clients", "keyspace"] {
                        if let Ok(val) = redis::cmd("INFO").arg(sec).query_async::<redis::Value>(conn).await {
                            let s = Self::redis_value_to_string(&val);
                            if !s.is_empty() {
                                collected_info.push_str(&s);
                                collected_info.push('\n');
                            }
                        }
                    }
                }

                if !collected_info.is_empty() {
                    self.telemetry.metrics = TelemetryParser::parse_info(
                        &collected_info,
                        Some(&self.telemetry.metrics),
                        elapsed_sec,
                    );
                }

                if let Ok(nodes_str) = redis::cmd("CLUSTER").arg("NODES").query_async::<String>(conn).await {
                    self.telemetry.topology = ClusterTopologyParser::parse_cluster_nodes(
                        &nodes_str,
                        self.telemetry.metrics.ping_latency_ms,
                    );
                }
                if let Ok(slowlog_val) = redis::cmd("SLOWLOG").arg("GET").arg(20).query_async::<redis::Value>(conn).await {
                    let parsed = Self::parse_slowlog_redis_value(&slowlog_val, "cluster");
                    if !parsed.is_empty() {
                        self.telemetry.slowlogs = parsed;
                    }
                }

                // Push into rolling history
                self.telemetry.history.push(
                    self.telemetry.metrics.instantaneous_ops_per_sec.unwrap_or(0),
                    self.telemetry.metrics.cpu_usage_pct,
                );
            }
            XedisBackend::DemoMock => {
                // Simulated live fluctuation for demo
                let jitter = ((Instant::now().elapsed().as_millis() % 500) as f64) / 10.0;
                let qps_val = (12400.0 + jitter * 20.0) as u64;
                let cpu_val = (8.0 + jitter * 0.1).clamp(1.0, 99.0);

                self.telemetry.metrics.instantaneous_ops_per_sec = Some(qps_val);
                self.telemetry.metrics.cpu_usage_pct = (cpu_val * 10.0).round() / 10.0;
                self.telemetry.history.push(qps_val, cpu_val);
            }
        }
    }

    fn redis_value_to_string(val: &redis::Value) -> String {
        match val {
            redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
            redis::Value::SimpleString(s) => s.clone(),
            redis::Value::VerbatimString { text, .. } => text.clone(),
            redis::Value::Okay => "OK".to_string(),
            redis::Value::Int(i) => i.to_string(),
            redis::Value::Array(items) => {
                let mut buf = String::new();
                for item in items {
                    buf.push_str(&Self::redis_value_to_string(item));
                    buf.push('\n');
                }
                buf
            }
            redis::Value::Map(pairs) => {
                let mut buf = String::new();
                for (k, v) in pairs {
                    buf.push_str(&format!("{}: {}\n", Self::redis_value_to_string(k), Self::redis_value_to_string(v)));
                }
                buf
            }
            _ => String::new(),
        }
    }

    fn parse_slowlog_redis_value(val: &redis::Value, default_node: &str) -> Vec<SlowlogEntry> {
        let mut result = Vec::new();
        if let redis::Value::Array(entries) = val {
            for entry_val in entries {
                if let redis::Value::Array(fields) = entry_val {
                    if fields.len() >= 4 {
                        let id = match &fields[0] {
                            redis::Value::Int(i) => *i as u64,
                            _ => 0,
                        };
                        let timestamp_sec = match &fields[1] {
                            redis::Value::Int(ts) => *ts,
                            _ => 0,
                        };
                        let time_str = if timestamp_sec > 0 {
                            chrono::DateTime::from_timestamp(timestamp_sec, 0)
                                .map(|dt| dt.format("%H:%M:%S").to_string())
                                .unwrap_or_else(|| Local::now().format("%H:%M:%S").to_string())
                        } else {
                            Local::now().format("%H:%M:%S").to_string()
                        };

                        let duration_us = match &fields[2] {
                            redis::Value::Int(us) => *us as f64,
                            _ => 0.0,
                        };
                        let latency_ms = (duration_us / 1000.0 * 10.0).round() / 10.0;

                        let command_str = match &fields[3] {
                            redis::Value::Array(cmd_tokens) => {
                                let tokens: Vec<String> = cmd_tokens
                                    .iter()
                                    .map(|v| match v {
                                        redis::Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
                                        _ => format!("{:?}", v),
                                    })
                                    .collect();
                                tokens.join(" ")
                            }
                            redis::Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
                            _ => "UNKNOWN".to_string(),
                        };

                        let node_str = if fields.len() >= 5 {
                            match &fields[4] {
                                redis::Value::BulkString(b) => String::from_utf8_lossy(b).to_string(),
                                _ => default_node.to_string(),
                            }
                        } else {
                            default_node.to_string()
                        };

                        let suggestion = MacroEngine::suggest_for_slow_command(&command_str);

                        result.push(SlowlogEntry {
                            id,
                            timestamp: time_str,
                            latency_ms,
                            command: command_str,
                            node: node_str,
                            suggestion,
                        });
                    }
                }
            }
        }
        result
    }
}
