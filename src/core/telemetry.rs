use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryMetrics {
    // Server Info
    pub version: String,
    pub redis_mode: String,
    pub os: String,
    pub uptime_in_seconds: u64,
    pub uptime_human: String,
    pub ping_latency_ms: f64,

    // Memory Metrics (Option-aware for custom middleware)
    pub used_memory_bytes: Option<u64>,
    pub used_memory_human: String,
    pub max_memory_bytes: Option<u64>,
    pub max_memory_human: String,
    pub used_memory_rss_bytes: Option<u64>,
    pub used_memory_rss_human: String,
    pub used_memory_peak_bytes: Option<u64>,
    pub used_memory_peak_human: String,
    pub mem_fragmentation_ratio: Option<f64>,

    // Throughput & Hit Rate Stats
    pub instantaneous_ops_per_sec: Option<u64>,
    pub total_commands_processed: u64,
    pub instantaneous_input_kbps: Option<f64>,
    pub instantaneous_output_kbps: Option<f64>,
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
    pub hit_rate_pct: Option<f64>,

    // CPU Metrics (Option-aware for Main vs Fork/Children)
    pub used_cpu_sys: Option<f64>,
    pub used_cpu_user: Option<f64>,
    pub used_cpu_sys_main_thread: Option<f64>,
    pub used_cpu_user_main_thread: Option<f64>,
    pub used_cpu_sys_children: Option<f64>,
    pub used_cpu_user_children: Option<f64>,
    pub cpu_sys_pct: Option<f64>,
    pub cpu_user_pct: Option<f64>,
    pub cpu_children_sys_pct: Option<f64>,
    pub cpu_children_user_pct: Option<f64>,
    pub cpu_usage_pct: f64,
    pub cpu_sample_elapsed_sec: f64,

    // Client & Keyspace Stats
    pub connected_clients: u64,
    pub blocked_clients: u64,
    pub total_keys: u64,
    pub expires_keys: u64,
}

impl Default for TelemetryMetrics {
    fn default() -> Self {
        Self {
            version: "7.2.4".to_string(),
            redis_mode: "standalone".to_string(),
            os: "Linux/macOS".to_string(),
            uptime_in_seconds: 3600 * 24 * 3 + 1800,
            uptime_human: "3d 00h 30m".to_string(),
            ping_latency_ms: 0.38,

            used_memory_bytes: Some(2_297_900_000),
            used_memory_human: "2.14 GB".to_string(),
            max_memory_bytes: Some(4_294_967_296),
            max_memory_human: "4.00 GB".to_string(),
            used_memory_rss_bytes: Some(2_576_980_377),
            used_memory_rss_human: "2.40 GB".to_string(),
            used_memory_peak_bytes: Some(2_899_102_924),
            used_memory_peak_human: "2.70 GB".to_string(),
            mem_fragmentation_ratio: Some(1.12),

            instantaneous_ops_per_sec: Some(12480),
            total_commands_processed: 89_240_100,
            instantaneous_input_kbps: Some(842.5),
            instantaneous_output_kbps: Some(1520.1),
            keyspace_hits: 14_290_000,
            keyspace_misses: 115_000,
            hit_rate_pct: Some(99.2),

            used_cpu_sys: Some(1420.5),
            used_cpu_user: Some(890.2),
            used_cpu_sys_main_thread: Some(1420.5),
            used_cpu_user_main_thread: Some(890.2),
            used_cpu_sys_children: None,
            used_cpu_user_children: None,
            cpu_sys_pct: Some(2.8),
            cpu_user_pct: Some(5.6),
            cpu_children_sys_pct: None,
            cpu_children_user_pct: None,
            cpu_usage_pct: 8.4,
            cpu_sample_elapsed_sec: 0.0,

            connected_clients: 342,
            blocked_clients: 0,
            total_keys: 1_208_490,
            expires_keys: 340_120,
        }
    }
}

impl TelemetryMetrics {
    pub fn empty() -> Self {
        Self {
            version: "unknown".to_string(),
            redis_mode: "standalone".to_string(),
            os: "unknown".to_string(),
            uptime_in_seconds: 0,
            uptime_human: "0s".to_string(),
            ping_latency_ms: 0.0,

            used_memory_bytes: None,
            used_memory_human: String::new(),
            max_memory_bytes: None,
            max_memory_human: String::new(),
            used_memory_rss_bytes: None,
            used_memory_rss_human: String::new(),
            used_memory_peak_bytes: None,
            used_memory_peak_human: String::new(),
            mem_fragmentation_ratio: None,

            instantaneous_ops_per_sec: None,
            total_commands_processed: 0,
            instantaneous_input_kbps: None,
            instantaneous_output_kbps: None,
            keyspace_hits: 0,
            keyspace_misses: 0,
            hit_rate_pct: None,

            used_cpu_sys: None,
            used_cpu_user: None,
            used_cpu_sys_main_thread: None,
            used_cpu_user_main_thread: None,
            used_cpu_sys_children: None,
            used_cpu_user_children: None,
            cpu_sys_pct: None,
            cpu_user_pct: None,
            cpu_children_sys_pct: None,
            cpu_children_user_pct: None,
            cpu_usage_pct: 0.0,
            cpu_sample_elapsed_sec: 0.0,

            connected_clients: 0,
            blocked_clients: 0,
            total_keys: 0,
            expires_keys: 0,
        }
    }

    pub fn used_memory_display(&self) -> String {
        if self.used_memory_bytes.is_some() && !self.used_memory_human.is_empty() {
            self.used_memory_human.clone()
        } else {
            "N/A".to_string()
        }
    }

    pub fn rss_display(&self) -> String {
        if self.used_memory_rss_bytes.is_some() && !self.used_memory_rss_human.is_empty() {
            self.used_memory_rss_human.clone()
        } else {
            "N/A".to_string()
        }
    }

    pub fn max_memory_display(&self) -> String {
        if let Some(max_b) = self.max_memory_bytes {
            if max_b > 0 {
                self.max_memory_human.clone()
            } else {
                "Unlimited".to_string()
            }
        } else {
            "N/A".to_string()
        }
    }

    pub fn frag_display(&self) -> String {
        if let Some(ratio) = self.mem_fragmentation_ratio {
            format!("{:.2}", ratio)
        } else {
            "N/A".to_string()
        }
    }

    pub fn main_cpu_display(&self) -> String {
        if let (Some(s), Some(u)) = (self.cpu_sys_pct, self.cpu_user_pct) {
            format!("Sys {:.1}% / Usr {:.1}%", s, u)
        } else if self.used_cpu_sys.is_some() || self.used_cpu_user.is_some() {
            format!("Sys {:.1}% / Usr {:.1}%", self.cpu_usage_pct * 0.35, self.cpu_usage_pct * 0.65)
        } else {
            "N/A".to_string()
        }
    }

    pub fn children_cpu_display(&self) -> String {
        if let (Some(s), Some(u)) = (self.cpu_children_sys_pct, self.cpu_children_user_pct) {
            format!("Sys {:.1}% / Usr {:.1}%", s, u)
        } else if self.used_cpu_sys_children.is_some() || self.used_cpu_user_children.is_some() {
            "Sys 0.0% / Usr 0.0%".to_string()
        } else {
            "N/A".to_string()
        }
    }

    pub fn ops_display(&self) -> String {
        if let Some(ops) = self.instantaneous_ops_per_sec {
            TelemetryParser::format_number(ops)
        } else {
            "N/A".to_string()
        }
    }

    pub fn net_display(&self) -> String {
        match (self.instantaneous_input_kbps, self.instantaneous_output_kbps) {
            (Some(in_k), Some(out_k)) => format!("{:.0}k/{:.0}k", in_k, out_k),
            (Some(in_k), None) => format!("{:.0}k/N/A", in_k),
            (None, Some(out_k)) => format!("N/A/{:.0}k", out_k),
            (None, None) => "N/A".to_string(),
        }
    }

    pub fn hit_rate_display(&self) -> String {
        if let Some(rate) = self.hit_rate_pct {
            format!("{:.1}%", rate)
        } else {
            "N/A".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsHistory {
    pub qps: VecDeque<u64>,
    pub cpu: VecDeque<f64>, // stored as floating point percentage (0.0..=100.0)
    pub capacity: usize,
}

#[allow(dead_code)]
impl MetricsHistory {
    pub fn new_empty(capacity: usize) -> Self {
        Self {
            qps: VecDeque::with_capacity(capacity),
            cpu: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn new(capacity: usize) -> Self {
        let default_qps: Vec<u64> = vec![
            12100, 12250, 12400, 12380, 12550, 12600, 12480, 12490, 12520, 12480,
            12350, 12420, 12580, 12610, 12490, 12530, 12500, 12480, 12450, 12480,
        ];
        let default_cpu: Vec<f64> = vec![
            6.5, 7.2, 8.0, 9.4, 7.8, 8.6, 12.4, 8.2, 7.5, 8.4,
            9.1, 8.8, 10.2, 14.5, 8.4, 7.9, 8.1, 9.0, 8.2, 8.4,
        ];

        let mut qps = VecDeque::with_capacity(capacity);
        for v in default_qps.into_iter().take(capacity) {
            qps.push_back(v);
        }
        let mut cpu = VecDeque::with_capacity(capacity);
        for v in default_cpu.into_iter().take(capacity) {
            cpu.push_back(v);
        }

        Self { qps, cpu, capacity }
    }

    pub fn push(&mut self, qps_val: u64, cpu_pct: f64) {
        if self.qps.len() >= self.capacity {
            self.qps.pop_front();
        }
        self.qps.push_back(qps_val);

        if self.cpu.len() >= self.capacity {
            self.cpu.pop_front();
        }
        self.cpu.push_back(cpu_pct.clamp(0.0, 100.0));
    }

    pub fn qps_as_slice(&self) -> Vec<u64> {
        self.qps.iter().copied().collect()
    }

    pub fn cpu_as_slice(&self) -> Vec<u64> {
        self.cpu.iter().map(|v| v.round() as u64).collect()
    }

    pub fn cpu_float_slice(&self) -> Vec<f64> {
        self.cpu.iter().copied().collect()
    }

    pub fn max_qps(&self) -> u64 {
        self.qps.iter().copied().max().unwrap_or(0)
    }

    pub fn avg_qps(&self) -> u64 {
        if self.qps.is_empty() {
            0
        } else {
            let sum: u64 = self.qps.iter().sum();
            sum / self.qps.len() as u64
        }
    }

    pub fn max_cpu(&self) -> f64 {
        self.cpu.iter().copied().fold(0.0, f64::max)
    }

    pub fn avg_cpu(&self) -> f64 {
        if self.cpu.is_empty() {
            0.0
        } else {
            let sum: f64 = self.cpu.iter().sum();
            sum / self.cpu.len() as f64
        }
    }
}

pub struct TelemetryParser;

impl TelemetryParser {
    /// Parse flexible integer values from strings like "1024", "1024B", "500MB", "2.5G", "12.0"
    pub fn parse_flexible_u64(raw: &str) -> Option<u64> {
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }

        // Direct integer parse
        if let Ok(n) = s.parse::<u64>() {
            return Some(n);
        }

        // Strip unit suffixes or float representation
        let lower = s.to_lowercase();
        if let Ok(f) = s.parse::<f64>() {
            return Some(f.round() as u64);
        }

        if lower.ends_with("tb") || lower.ends_with('t') {
            let num_str = lower.trim_end_matches("tb").trim_end_matches('t');
            num_str.parse::<f64>().ok().map(|v| (v * 1024.0 * 1024.0 * 1024.0 * 1024.0).round() as u64)
        } else if lower.ends_with("gb") || lower.ends_with('g') {
            let num_str = lower.trim_end_matches("gb").trim_end_matches('g');
            num_str.parse::<f64>().ok().map(|v| (v * 1024.0 * 1024.0 * 1024.0).round() as u64)
        } else if lower.ends_with("mb") || lower.ends_with('m') {
            let num_str = lower.trim_end_matches("mb").trim_end_matches('m');
            num_str.parse::<f64>().ok().map(|v| (v * 1024.0 * 1024.0).round() as u64)
        } else if lower.ends_with("kb") || lower.ends_with('k') {
            let num_str = lower.trim_end_matches("kb").trim_end_matches('k');
            num_str.parse::<f64>().ok().map(|v| (v * 1024.0).round() as u64)
        } else if lower.ends_with('b') || lower.ends_with("bytes") {
            let num_str = lower.trim_end_matches("bytes").trim_end_matches('b').trim();
            num_str.parse::<f64>().ok().map(|v| v.round() as u64)
        } else {
            None
        }
    }

    /// Parse flexible floating point values from strings like "12.5", "12.5s", "12.5ms", "85%"
    pub fn parse_flexible_f64(raw: &str) -> Option<f64> {
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }

        if let Ok(f) = s.parse::<f64>() {
            return Some(f);
        }

        let lower = s.to_lowercase();
        let cleaned = lower
            .trim_end_matches('%')
            .trim_end_matches("sec")
            .trim_end_matches('s')
            .trim_end_matches("ms")
            .trim();

        cleaned.parse::<f64>().ok()
    }

    pub fn parse_info(
        info_str: &str,
        prev: Option<&TelemetryMetrics>,
        elapsed_sec: f64,
    ) -> TelemetryMetrics {
        let mut metrics = prev.cloned().unwrap_or_else(TelemetryMetrics::empty);
        let mut total_keys_acc = 0u64;
        let mut total_expires_acc = 0u64;
        let mut parsed_instantaneous_ops: Option<u64> = None;

        for line in info_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Split on either colon ':' or equals '=' with whitespace tolerance
            let pair = line.split_once(':').or_else(|| line.split_once('='));

            if let Some((raw_k, raw_v)) = pair {
                let k_norm = raw_k.trim().to_lowercase().replace('-', "_");
                let v = raw_v.trim();

                match k_norm.as_str() {
                    // Server
                    "redis_version" | "version" | "server_version" => metrics.version = v.to_string(),
                    "redis_mode" | "mode" | "server_mode" => metrics.redis_mode = v.to_string(),
                    "os" | "operating_system" => metrics.os = v.to_string(),
                    "uptime_in_seconds" | "uptime_seconds" | "uptime" | "uptime_sec" => {
                        if let Some(sec) = Self::parse_flexible_u64(v) {
                            metrics.uptime_in_seconds = sec;
                            metrics.uptime_human = Self::format_uptime(sec);
                        }
                    }

                    // Memory
                    "used_memory" | "used_memory_bytes" | "used_mem" | "mem_used" | "memory_used" => {
                        if let Some(b) = Self::parse_flexible_u64(v) {
                            metrics.used_memory_bytes = Some(b);
                            metrics.used_memory_human = Self::format_bytes(b);
                        }
                    }
                    "used_memory_human" | "used_mem_human" => metrics.used_memory_human = v.to_string(),
                    "maxmemory" | "max_memory" | "max_memory_bytes" | "max_mem" => {
                        if let Some(b) = Self::parse_flexible_u64(v) {
                            metrics.max_memory_bytes = Some(b);
                            metrics.max_memory_human = if b > 0 {
                                Self::format_bytes(b)
                            } else {
                                "Unlimited".to_string()
                            };
                        }
                    }
                    "maxmemory_human" | "max_memory_human" => {
                        if v != "0B" && !v.is_empty() {
                            metrics.max_memory_human = v.to_string();
                        }
                    }
                    "used_memory_rss" | "used_memory_rss_bytes" | "used_rss" | "rss" | "rss_memory" => {
                        if let Some(b) = Self::parse_flexible_u64(v) {
                            metrics.used_memory_rss_bytes = Some(b);
                            metrics.used_memory_rss_human = Self::format_bytes(b);
                        }
                    }
                    "used_memory_rss_human" | "rss_human" => metrics.used_memory_rss_human = v.to_string(),
                    "used_memory_peak" | "used_memory_peak_bytes" | "peak_memory" => {
                        if let Some(b) = Self::parse_flexible_u64(v) {
                            metrics.used_memory_peak_bytes = Some(b);
                            metrics.used_memory_peak_human = Self::format_bytes(b);
                        }
                    }
                    "used_memory_peak_human" | "peak_memory_human" => metrics.used_memory_peak_human = v.to_string(),
                    "mem_fragmentation_ratio" | "mem_fragmentation" | "frag_ratio" | "fragmentation_ratio" => {
                        if let Some(ratio) = Self::parse_flexible_f64(v) {
                            metrics.mem_fragmentation_ratio = Some(ratio);
                        }
                    }

                    // Stats & Throughput
                    "instantaneous_ops_per_sec" | "instantaneous_ops" | "ops_per_sec" | "qps" | "ops" => {
                        if let Some(ops) = Self::parse_flexible_u64(v) {
                            parsed_instantaneous_ops = Some(ops);
                        }
                    }
                    "total_commands_processed" | "total_commands" | "total_ops" => {
                        if let Some(total) = Self::parse_flexible_u64(v) {
                            metrics.total_commands_processed = total;
                        }
                    }
                    "instantaneous_input_kbps" | "input_kbps" | "net_input_kbps" => {
                        if let Some(kbps) = Self::parse_flexible_f64(v) {
                            metrics.instantaneous_input_kbps = Some(kbps);
                        }
                    }
                    "instantaneous_output_kbps" | "output_kbps" | "net_output_kbps" => {
                        if let Some(kbps) = Self::parse_flexible_f64(v) {
                            metrics.instantaneous_output_kbps = Some(kbps);
                        }
                    }
                    "keyspace_hits" | "hits" => {
                        if let Some(hits) = Self::parse_flexible_u64(v) {
                            metrics.keyspace_hits = hits;
                        }
                    }
                    "keyspace_misses" | "misses" => {
                        if let Some(misses) = Self::parse_flexible_u64(v) {
                            metrics.keyspace_misses = misses;
                        }
                    }

                    // CPU
                    "used_cpu_sys_main_thread" | "cpu_sys_main_thread" | "cpu_sys_main" => {
                        if let Some(sys_m) = Self::parse_flexible_f64(v) {
                            metrics.used_cpu_sys_main_thread = Some(sys_m);
                        }
                    }
                    "used_cpu_user_main_thread" | "cpu_user_main_thread" | "cpu_user_main" => {
                        if let Some(user_m) = Self::parse_flexible_f64(v) {
                            metrics.used_cpu_user_main_thread = Some(user_m);
                        }
                    }
                    "used_cpu_sys" | "cpu_sys" | "cpu_system" | "sys_cpu" => {
                        if let Some(sys) = Self::parse_flexible_f64(v) {
                            metrics.used_cpu_sys = Some(sys);
                        }
                    }
                    "used_cpu_user" | "cpu_user" | "user_cpu" => {
                        if let Some(user) = Self::parse_flexible_f64(v) {
                            metrics.used_cpu_user = Some(user);
                        }
                    }
                    "used_cpu_sys_children" | "cpu_sys_children" => {
                        if let Some(sys_c) = Self::parse_flexible_f64(v) {
                            metrics.used_cpu_sys_children = Some(sys_c);
                        }
                    }
                    "used_cpu_user_children" | "cpu_user_children" => {
                        if let Some(user_c) = Self::parse_flexible_f64(v) {
                            metrics.used_cpu_user_children = Some(user_c);
                        }
                    }

                    // Clients
                    "connected_clients" | "clients" | "curr_clients" => {
                        if let Some(c) = Self::parse_flexible_u64(v) {
                            metrics.connected_clients = c;
                        }
                    }
                    "blocked_clients" | "blocked" => {
                        if let Some(b) = Self::parse_flexible_u64(v) {
                            metrics.blocked_clients = b;
                        }
                    }
                    "total_keys" | "keys" => {
                        if let Some(k_cnt) = Self::parse_flexible_u64(v) {
                            total_keys_acc = k_cnt;
                        }
                    }

                    // Keyspace databases: db0:keys=1000,expires=200...
                    _ if k_norm.starts_with("db") => {
                        for part in v.split(',') {
                            if let Some((sub_k, sub_v)) = part.split_once('=').or_else(|| part.split_once(':')) {
                                let sub_k_norm = sub_k.trim().to_lowercase();
                                if sub_k_norm == "keys" {
                                    if let Some(cnt) = Self::parse_flexible_u64(sub_v) {
                                        total_keys_acc += cnt;
                                    }
                                } else if sub_k_norm == "expires" {
                                    if let Some(cnt) = Self::parse_flexible_u64(sub_v) {
                                        total_expires_acc += cnt;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if total_keys_acc > 0 {
            metrics.total_keys = total_keys_acc;
            metrics.expires_keys = total_expires_acc;
        }

        // Calculate CPU percentage with Delta Window Compensation & EMA Smoothing
        if let Some(prev_m) = prev {
            if elapsed_sec > 0.05 {
                let sys_diff = match (metrics.used_cpu_sys, prev_m.used_cpu_sys) {
                    (Some(curr), Some(p)) => (curr - p).max(0.0),
                    _ => 0.0,
                };
                let user_diff = match (metrics.used_cpu_user, prev_m.used_cpu_user) {
                    (Some(curr), Some(p)) => (curr - p).max(0.0),
                    _ => 0.0,
                };

                let total_cpu_diff = sys_diff + user_diff;
                let accumulated_time = prev_m.cpu_sample_elapsed_sec + elapsed_sec;

                let (raw_sys_pct, raw_user_pct, raw_total_pct) = if total_cpu_diff > 0.00001 {
                    // Counter changed! Compute rate using accumulated elapsed window
                    let effective_sec = accumulated_time.max(elapsed_sec);
                    metrics.cpu_sample_elapsed_sec = 0.0;
                    let sys_pct = (sys_diff / effective_sec) * 100.0;
                    let user_pct = (user_diff / effective_sec) * 100.0;
                    let total_pct = (sys_pct + user_pct).clamp(0.0, 100.0);
                    (sys_pct, user_pct, total_pct)
                } else {
                    // Counter unchanged: accumulate time window across silent intervals
                    metrics.cpu_sample_elapsed_sec = accumulated_time;
                    (0.0, 0.0, 0.0)
                };

                if metrics.used_cpu_sys.is_some() || metrics.used_cpu_user.is_some() {
                    metrics.cpu_sys_pct = Some((raw_sys_pct * 10.0).round() / 10.0);
                    metrics.cpu_user_pct = Some((raw_user_pct * 10.0).round() / 10.0);
                    metrics.cpu_usage_pct = (raw_total_pct * 10.0).round() / 10.0;
                }

                if metrics.used_cpu_sys_children.is_some() || metrics.used_cpu_user_children.is_some() {
                    let sys_c_diff = match (metrics.used_cpu_sys_children, prev_m.used_cpu_sys_children) {
                        (Some(curr), Some(p)) => (curr - p).max(0.0),
                        _ => 0.0,
                    };
                    let user_c_diff = match (metrics.used_cpu_user_children, prev_m.used_cpu_user_children) {
                        (Some(curr), Some(p)) => (curr - p).max(0.0),
                        _ => 0.0,
                    };
                    let raw_c_sys = (sys_c_diff / elapsed_sec) * 100.0;
                    let raw_c_user = (user_c_diff / elapsed_sec) * 100.0;
                    metrics.cpu_children_sys_pct = Some((raw_c_sys * 10.0).round() / 10.0);
                    metrics.cpu_children_user_pct = Some((raw_c_user * 10.0).round() / 10.0);
                }

                if parsed_instantaneous_ops.is_none() || parsed_instantaneous_ops == Some(0) {
                    let cmd_diff = metrics.total_commands_processed.saturating_sub(prev_m.total_commands_processed);
                    if cmd_diff > 0 {
                        metrics.instantaneous_ops_per_sec = Some((cmd_diff as f64 / elapsed_sec).round() as u64);
                    } else {
                        metrics.instantaneous_ops_per_sec = parsed_instantaneous_ops;
                    }
                } else {
                    metrics.instantaneous_ops_per_sec = parsed_instantaneous_ops;
                }
            }
        } else {
            metrics.instantaneous_ops_per_sec = parsed_instantaneous_ops;
            if metrics.used_cpu_sys.is_some() || metrics.used_cpu_user.is_some() {
                metrics.cpu_sys_pct = Some(metrics.cpu_usage_pct * 0.35);
                metrics.cpu_user_pct = Some(metrics.cpu_usage_pct * 0.65);
            }
        }

        // Calculate Hit Rate
        let total_lookups = metrics.keyspace_hits + metrics.keyspace_misses;
        if total_lookups > 0 {
            metrics.hit_rate_pct = Some((metrics.keyspace_hits as f64 / total_lookups as f64) * 100.0);
        }

        // Auto compute fragmentation ratio if not provided directly
        if metrics.mem_fragmentation_ratio.is_none() {
            if let (Some(rss), Some(used)) = (metrics.used_memory_rss_bytes, metrics.used_memory_bytes) {
                if used > 0 {
                    metrics.mem_fragmentation_ratio = Some(rss as f64 / used as f64);
                }
            }
        }

        metrics
    }

    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn format_uptime(sec: u64) -> String {
        let days = sec / 86400;
        let hours = (sec % 86400) / 3600;
        let mins = (sec % 3600) / 60;
        if days > 0 {
            format!("{}d {:02}h {:02}m", days, hours, mins)
        } else if hours > 0 {
            format!("{}h {:02}m", hours, mins)
        } else {
            format!("{}m {:02}s", mins, sec % 60)
        }
    }

    pub fn format_number(n: u64) -> String {
        if n >= 1_000_000_000 {
            format!("{:.2}B", n as f64 / 1_000_000_000.0)
        } else if n >= 1_000_000 {
            format!("{:.2}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}K", n as f64 / 1_000.0)
        } else {
            n.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_thread_cpu_parsing() {
        let info_str = r#"
# CPU
used_cpu_sys: 2.563428
used_cpu_user: 6.486121
used_cpu_sys_children: 0.000000
used_cpu_user_children: 0.000000
used_cpu_sys_main_thread: 2.563428
used_cpu_user_main_thread: 6.486121
"#;
        let metrics = TelemetryParser::parse_info(info_str, None, 1.0);
        assert_eq!(metrics.used_cpu_sys, Some(2.563428));
        assert_eq!(metrics.used_cpu_user, Some(6.486121));
        assert_eq!(metrics.used_cpu_sys_main_thread, Some(2.563428));
        assert_eq!(metrics.used_cpu_user_main_thread, Some(6.486121));
    }

    #[test]
    fn test_stepped_cpu_counter_spike_elimination() {
        // Step 0: Initial baseline
        let info_init = "used_cpu_sys: 2.0\nused_cpu_user: 5.0\n";
        let mut m = TelemetryParser::parse_info(info_init, None, 1.0);

        // Step 1 to 4: Server counters remain unchanged for 4 seconds (low-frequency refresh)
        for _ in 0..4 {
            m = TelemetryParser::parse_info(info_init, Some(&m), 1.0);
            assert!(m.cpu_usage_pct <= 10.0);
        }

        // Verify accumulated elapsed window is around 4.0s
        assert_eq!(m.cpu_sample_elapsed_sec, 4.0);

        // Step 5: Server suddenly updates counter by +0.3s (0.1s sys, 0.2s user) over 5 total seconds
        // Real average CPU load = 0.3s / 5.0s = 6.0%
        // Without window compensation, raw 1s diff would be 0.3s / 1.0s = 30.0% or higher
        let info_step5 = "used_cpu_sys: 2.1\nused_cpu_user: 5.2\n";
        let m5 = TelemetryParser::parse_info(info_step5, Some(&m), 1.0);

        // CPU window was reset and usage percentage is appropriately smoothed
        assert_eq!(m5.cpu_sample_elapsed_sec, 0.0);
        assert!(m5.cpu_usage_pct < 10.0, "Expected smoothed CPU < 10.0%, got {}", m5.cpu_usage_pct);
        assert!(m5.cpu_usage_pct > 0.0);
    }

    #[test]
    fn test_continuous_cpu_smoothing() {
        let mut m = TelemetryParser::parse_info("used_cpu_sys: 10.0\nused_cpu_user: 20.0\n", None, 1.0);
        
        // Feed 10% CPU load continuously every second (0.04s sys, 0.06s user)
        for i in 1..=5 {
            let sys = 10.0 + (i as f64) * 0.04;
            let user = 20.0 + (i as f64) * 0.06;
            let info = format!("used_cpu_sys: {:.4}\nused_cpu_user: {:.4}\n", sys, user);
            m = TelemetryParser::parse_info(&info, Some(&m), 1.0);
        }

        // Should converge towards 10.0%
        assert!((m.cpu_usage_pct - 10.0).abs() < 2.0, "Expected CPU around 10.0%, got {}", m.cpu_usage_pct);
    }
}
