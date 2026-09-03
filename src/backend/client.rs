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

    pub fn is_broadcast_target(target: &str) -> bool {
        let t = target.to_lowercase();
        t == "all" || t == "cluster" || t == "all-masters" || t == "all-replicas"
    }

    pub fn filter_nodes_for_target(&self, target: &str) -> Vec<ClusterNode> {
        let all_nodes = self.telemetry.nodes();
        let target_lower = target.to_lowercase();
        if target_lower == "all" || target_lower == "cluster" {
            if all_nodes.is_empty() {
                return ClusterTopology::mock_cluster_topology()
                    .shards
                    .iter()
                    .map(|s| s.master.clone())
                    .collect();
            }
            all_nodes
        } else if target_lower == "all-masters" {
            let masters: Vec<_> = all_nodes
                .iter()
                .filter(|n| n.role.eq_ignore_ascii_case("Master"))
                .cloned()
                .collect();
            if masters.is_empty() {
                ClusterTopology::mock_cluster_topology()
                    .shards
                    .into_iter()
                    .map(|s| s.master)
                    .collect()
            } else {
                masters
            }
        } else if target_lower == "all-replicas" {
            all_nodes
                .into_iter()
                .filter(|n| n.role.eq_ignore_ascii_case("Replica"))
                .collect()
        } else {
            all_nodes
                .into_iter()
                .filter(|n| {
                    n.id.eq_ignore_ascii_case(target)
                        || n.address.eq_ignore_ascii_case(target)
                        || n.raw_id.starts_with(target)
                        || n.address.split(':').next() == Some(target)
                })
                .collect()
        }
    }

    pub fn build_node_url(base_url: &str, node_addr: &str) -> String {
        let clean_addr = if let Some((addr, _cport)) = node_addr.split_once('@') {
            addr
        } else {
            node_addr
        };

        if let Some(rest) = base_url.strip_prefix("redis://") {
            if let Some((auth, _)) = rest.split_once('@') {
                return format!("redis://{}@{}", auth, clean_addr);
            }
            return format!("redis://{}", clean_addr);
        }
        if let Some(rest) = base_url.strip_prefix("rediss://") {
            if let Some((auth, _)) = rest.split_once('@') {
                return format!("rediss://{}@{}", auth, clean_addr);
            }
            return format!("rediss://{}", clean_addr);
        }
        format!("redis://{}", clean_addr)
    }

    pub fn format_node_section_header(node: &ClusterNode, duration: Duration) -> String {
        let latency_str = if duration.as_micros() < 1000 {
            format!("{:.2}ms", duration.as_micros() as f64 / 1000.0)
        } else {
            format!("{}ms", duration.as_millis())
        };
        format!(
            "--- Node: @{} ({} · {}) [{}] ---",
            node.id, node.address, node.role, latency_str
        )
    }

    fn generate_mock_response_for_node(node: &ClusterNode, cmd: &str, args: &[String]) -> FormattedValue {
        let upper_cmd = cmd.to_uppercase();
        match upper_cmd.as_str() {
            "PING" => FormattedValue::Status("PONG".to_string()),
            "GET" => {
                let key = args.first().map(|s| s.as_str()).unwrap_or("key");
                FormattedValue::String(format!("val_for_{} (from {})", key, node.id))
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
                    vec!["node".to_string(), format!("\"{}\"", node.id)],
                    vec!["ttl".to_string(), "3600 (1h)".to_string()],
                ],
            },
            "LRANGE" => FormattedValue::List(vec![
                format!("{}:task:001", node.id),
                format!("{}:task:002", node.id),
                format!("{}:task:003", node.id),
            ]),
            "SMEMBERS" => FormattedValue::List(vec![
                "admin".to_string(),
                "developer".to_string(),
                "tester".to_string(),
            ]),
            "SCAN" => {
                let prefix = args.get(1).map(|s| s.trim_end_matches('*')).unwrap_or("key");
                FormattedValue::Json(
                    serde_json::to_string_pretty(&serde_json::json!([
                        format!("{}:{}:001", prefix, node.id),
                        format!("{}:{}:002", prefix, node.id),
                    ]))
                    .unwrap_or_default(),
                )
            }
            "INFO" => {
                let port = node.address.split(':').nth(1).unwrap_or("6379");
                FormattedValue::String(format!(
                    "# Server\nredis_version:7.2.4\nredis_mode:cluster\ntcp_port:{}\nrun_id:{}\nrole:{}\n\n# Memory\nused_memory_human:2.14G\nmaxmemory_human:4.00G",
                    port, node.raw_id, node.role.to_lowercase()
                ))
            }
            "CLUSTER" => {
                if args.first().map(|s| s.to_uppercase()).as_deref() == Some("NODES") {
                    FormattedValue::String("node-1 127.0.0.1:6379@16379 master - 0 1788157290000 1 connected 0-5460\nnode-2 127.0.0.1:6380@16380 master - 0 1788157290000 2 connected 5461-10922\nnode-3 127.0.0.1:6381@16381 master - 0 1788157290000 3 connected 10923-16383".to_string())
                } else if args.first().map(|s| s.to_uppercase()).as_deref() == Some("INFO") {
                    FormattedValue::String(
                        "cluster_state: ok\ncluster_slots_assigned: 16384\ncluster_slots_ok: 16384\ncluster_slots_pfail: 0\ncluster_slots_fail: 0\ncluster_known_nodes: 6\ncluster_size: 3\ncluster_current_epoch: 6\ncluster_my_epoch: 1\ncluster_stats_messages_ping_sent: 520\ncluster_stats_messages_pong_sent: 520\ncluster_stats_messages_sent: 1040\ncluster_stats_messages_ping_received: 520\ncluster_stats_messages_pong_received: 520\ncluster_stats_messages_received: 1040\ntotal_cluster_links_buffer_limit_exceeded: 0".to_string()
                    )
                } else {
                    FormattedValue::Status("OK".to_string())
                }
            }
            _ => FormattedValue::Status(format!("OK (Simulated response for {} on @{})", cmd, node.id)),
        }
    }

    async fn execute_broadcast_command(&self, target: &str, cmd: &str, args: &[String]) -> (FormattedValue, Duration) {
        let start = Instant::now();
        let target_nodes = self.filter_nodes_for_target(target);

        if target_nodes.is_empty() {
            return (
                FormattedValue::Error("No reachable cluster nodes found for broadcast".to_string()),
                start.elapsed(),
            );
        }

        match &self.backend {
            XedisBackend::Live(_) | XedisBackend::LiveCluster(_) => {
                let mut tasks = Vec::new();
                for node in target_nodes {
                    let node_url = Self::build_node_url(&self.target_url, &node.address);
                    let cmd_str = cmd.to_string();
                    let args_vec = args.to_vec();

                    tasks.push(async move {
                        let node_start = Instant::now();
                        match redis::Client::open(node_url.as_str()) {
                            Ok(client) => match client.get_multiplexed_tokio_connection().await {
                                Ok(mut conn) => {
                                    let mut command = redis::cmd(&cmd_str);
                                    for arg in &args_vec {
                                        command.arg(arg);
                                    }
                                    match command.query_async::<redis::Value>(&mut conn).await {
                                        Ok(val) => {
                                            let formatted = FormattedValue::from_redis_value(val);
                                            (node, formatted, node_start.elapsed())
                                        }
                                        Err(e) => (
                                            node,
                                            FormattedValue::Error(format!("ERR {}", e)),
                                            node_start.elapsed(),
                                        ),
                                    }
                                }
                                Err(e) => (
                                    node,
                                    FormattedValue::Error(format!("Connection error: {}", e)),
                                    node_start.elapsed(),
                                ),
                            },
                            Err(e) => (
                                node,
                                FormattedValue::Error(format!("Client error: {}", e)),
                                node_start.elapsed(),
                            ),
                        }
                    });
                }

                let results = futures_util::future::join_all(tasks).await;
                let mut sections = Vec::new();
                for (node, formatted, node_duration) in results {
                    let header = Self::format_node_section_header(&node, node_duration);
                    let body = formatted.to_display_text();
                    sections.push(format!("{}\n{}", header, body.trim()));
                }

                (FormattedValue::String(sections.join("\n\n")), start.elapsed())
            }
            XedisBackend::DemoMock => {
                tokio::time::sleep(Duration::from_millis(2)).await;
                let mut sections = Vec::new();
                for node in target_nodes {
                    let simulated_duration = Duration::from_micros((node.ping_ms * 1000.0) as u64).max(Duration::from_micros(200));
                    let header = Self::format_node_section_header(&node, simulated_duration);
                    let val = Self::generate_mock_response_for_node(&node, cmd, args);
                    let body = val.to_display_text();
                    sections.push(format!("{}\n{}", header, body.trim()));
                }
                (FormattedValue::String(sections.join("\n\n")), start.elapsed())
            }
        }
    }

    pub async fn execute_command(&mut self, target_node: Option<&str>, cmd: &str, args: &[String]) -> (FormattedValue, Duration) {
        let start = Instant::now();

        // 1. Check if broadcast mode is requested (@all, @cluster, @all-masters, @all-replicas)
        if let Some(target) = target_node {
            if Self::is_broadcast_target(target) {
                return self.execute_broadcast_command(target, cmd, args).await;
            }
        }

        // 2. Check if a specific single node is requested (@node-1, @127.0.0.1:6379, etc.)
        if let Some(target) = target_node {
            let matched_nodes = self.filter_nodes_for_target(target);
            if let Some(node) = matched_nodes.first() {
                match &self.backend {
                    XedisBackend::Live(_) | XedisBackend::LiveCluster(_) => {
                        let node_url = Self::build_node_url(&self.target_url, &node.address);
                        if let Ok(client) = redis::Client::open(node_url.as_str()) {
                            if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
                                let mut command = redis::cmd(cmd);
                                for arg in args {
                                    command.arg(arg);
                                }
                                match command.query_async::<redis::Value>(&mut conn).await {
                                    Ok(val) => return (FormattedValue::from_redis_value(val), start.elapsed()),
                                    Err(e) => return (FormattedValue::Error(format!("ERR {}", e)), start.elapsed()),
                                }
                            }
                        }
                    }
                    XedisBackend::DemoMock => {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                        let res = Self::generate_mock_response_for_node(node, cmd, args);
                        return (res, start.elapsed());
                    }
                }
            }
        }

        // 3. Fallback / Default execution on primary backend connection
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
                let default_node = ClusterNode {
                    id: "node-1".to_string(),
                    raw_id: "e01a1b2c3d4e5f60718293a4b5c6d7e8f9012345".to_string(),
                    address: "127.0.0.1:6379".to_string(),
                    cport: 16379,
                    role: "Master".to_string(),
                    master_id: None,
                    is_healthy: true,
                    ping_ms: 0.38,
                    slots_raw: "0-5460".to_string(),
                    slot_ranges: vec![(0, 5460)],
                    slot_count: 5461,
                    key_count: 402_830,
                };
                let res = Self::generate_mock_response_for_node(&default_node, cmd, args);
                (res, start.elapsed())
            }
        }
    }

    async fn execute_broadcast_scan(&self, target: &str, args: &[String]) -> (FormattedValue, Duration) {
        let start = Instant::now();
        let target_nodes = self.filter_nodes_for_target(target);
        let pattern = args.first().cloned().unwrap_or_else(|| "*".to_string());
        let count: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);

        match &self.backend {
            XedisBackend::Live(_) | XedisBackend::LiveCluster(_) => {
                let mut tasks = Vec::new();
                for node in target_nodes {
                    let node_url = Self::build_node_url(&self.target_url, &node.address);
                    let pat = pattern.clone();
                    tasks.push(async move {
                        let node_start = Instant::now();
                        match redis::Client::open(node_url.as_str()) {
                            Ok(client) => match client.get_multiplexed_tokio_connection().await {
                                Ok(mut conn) => {
                                    let mut cursor: u64 = 0;
                                    let mut all_keys = Vec::new();
                                    loop {
                                        let mut cmd = redis::cmd("SCAN");
                                        cmd.arg(cursor).arg("MATCH").arg(&pat).arg("COUNT").arg(count.min(100));
                                        if let Ok((next_cursor, keys)) = cmd.query_async::<(u64, Vec<String>)>(&mut conn).await {
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
                                    (node, FormattedValue::Json(json_val), node_start.elapsed())
                                }
                                Err(e) => (node, FormattedValue::Error(format!("Connection error: {}", e)), node_start.elapsed()),
                            },
                            Err(e) => (node, FormattedValue::Error(format!("Client error: {}", e)), node_start.elapsed()),
                        }
                    });
                }

                let results = futures_util::future::join_all(tasks).await;
                let mut sections = Vec::new();
                for (node, formatted, node_dur) in results {
                    let header = Self::format_node_section_header(&node, node_dur);
                    let body = formatted.to_display_text();
                    sections.push(format!("{}\n{}", header, body.trim()));
                }
                (FormattedValue::String(sections.join("\n\n")), start.elapsed())
            }
            XedisBackend::DemoMock => {
                tokio::time::sleep(Duration::from_millis(2)).await;
                let mut sections = Vec::new();
                for (idx, node) in target_nodes.iter().enumerate() {
                    let simulated_duration = Duration::from_micros((node.ping_ms * 1000.0) as u64).max(Duration::from_micros(200));
                    let header = Self::format_node_section_header(node, simulated_duration);
                    let pat_clean = pattern.trim_end_matches('*').trim_end_matches(':');
                    let mock_keys = vec![
                        format!("{}:{}:{:03}", pat_clean, node.id, idx * 2 + 1),
                        format!("{}:{}:{:03}", pat_clean, node.id, idx * 2 + 2),
                    ];
                    let json_val = serde_json::to_string_pretty(&mock_keys).unwrap_or_default();
                    sections.push(format!("{}\n{}", header, json_val));
                }
                (FormattedValue::String(sections.join("\n\n")), start.elapsed())
            }
        }
    }

    pub async fn execute_macro(
        &mut self,
        target_node: Option<&str>,
        macro_name: &str,
        args: &[String],
    ) -> (FormattedValue, Duration) {
        let start = Instant::now();
        let name = macro_name.to_lowercase();

        // 1. Check if broadcast scan is requested
        if let Some(target) = target_node {
            if Self::is_broadcast_target(target) && name == "/scan" {
                return self.execute_broadcast_scan(target, args).await;
            }
        }

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
                match redis::cmd("PING").query_async::<String>(conn).await {
                    Ok(pong) if pong == "PONG" => {
                        self.telemetry.connected = true;
                        self.telemetry.metrics.ping_latency_ms = ping_start.elapsed().as_micros() as f64 / 1000.0;
                    }
                    _ => {
                        self.telemetry.connected = false;
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
                match redis::cmd("PING").query_async::<String>(conn).await {
                    Ok(pong) if pong == "PONG" => {
                        self.telemetry.connected = true;
                        self.telemetry.metrics.ping_latency_ms = ping_start.elapsed().as_micros() as f64 / 1000.0;
                    }
                    _ => {
                        self.telemetry.connected = false;
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
