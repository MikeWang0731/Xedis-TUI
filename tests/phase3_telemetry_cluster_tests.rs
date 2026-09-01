use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use xedis_tui::app::{ActiveRightTab, App, FocusedPane};
use xedis_tui::backend::cluster_info::ClusterTopologyParser;
use xedis_tui::backend::formatter::FormattedValue;
use xedis_tui::config::AppConfig;
use xedis_tui::core::macro_engine::MacroEngine;
use xedis_tui::core::telemetry::{MetricsHistory, TelemetryMetrics, TelemetryParser};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn test_telemetry_info_parsing_and_metrics() {
    let info_str = r#"
# Server
redis_version:7.2.4
redis_mode:cluster
os:Linux 6.6.0-x86_64
uptime_in_seconds:259200

# Clients
connected_clients:128
blocked_clients:2

# Memory
used_memory:1073741824
used_memory_human:1.00G
used_memory_rss:1342177280
used_memory_rss_human:1.25G
used_memory_peak:1610612736
used_memory_peak_human:1.50G
maxmemory:2147483648
maxmemory_human:2.00G
mem_fragmentation_ratio:1.25

# Stats
total_commands_processed:15000000
instantaneous_ops_per_sec:15200
instantaneous_input_kbps:1200.50
instantaneous_output_kbps:2400.75
keyspace_hits:950000
keyspace_misses:50000

# CPU
used_cpu_sys:120.50
used_cpu_user:85.20

# Keyspace
db0:keys=500000,expires=100000,avg_ttl=3600
db1:keys=250000,expires=50000,avg_ttl=1800
"#;

    let metrics = TelemetryParser::parse_info(info_str, None, 1.0);

    assert_eq!(metrics.version, "7.2.4");
    assert_eq!(metrics.redis_mode, "cluster");
    assert_eq!(metrics.uptime_in_seconds, 259200);
    assert_eq!(metrics.uptime_human, "3d 00h 00m");
    assert_eq!(metrics.connected_clients, 128);
    assert_eq!(metrics.blocked_clients, 2);

    assert_eq!(metrics.used_memory_bytes, Some(1073741824));
    assert_eq!(metrics.used_memory_human, "1.00G");
    assert_eq!(metrics.max_memory_human, "2.00G");
    assert_eq!(metrics.mem_fragmentation_ratio, Some(1.25));

    assert_eq!(metrics.instantaneous_ops_per_sec, Some(15200));
    assert_eq!(metrics.total_keys, 750000);
    assert_eq!(metrics.expires_keys, 150000);
    assert_eq!(metrics.hit_rate_pct, Some(95.0));
}

#[test]
fn test_telemetry_delta_calculations() {
    let mut prev = TelemetryMetrics::default();
    prev.total_commands_processed = 100_000;
    prev.used_cpu_sys = Some(10.0);
    prev.used_cpu_user = Some(5.0);

    let info_str = r#"
# Server
redis_version:7.2.4

# Stats
total_commands_processed:115000
instantaneous_ops_per_sec:0

# CPU
used_cpu_sys:11.0
used_cpu_user:5.5
"#;

    let elapsed = 2.0; // 2 seconds elapsed
    let metrics = TelemetryParser::parse_info(info_str, Some(&prev), elapsed);

    // Delta commands: 15000 / 2s = 7500 ops/sec
    assert_eq!(metrics.instantaneous_ops_per_sec, Some(7500));

    // Delta CPU: (1.0 sys + 0.5 user) / 2.0s = 0.75 * 100 = 75.0%
    assert_eq!(metrics.cpu_usage_pct, 75.0);
}

#[test]
fn test_telemetry_na_fallbacks_for_custom_middleware() {
    // Middleware that only provides basic minimal fields (no fork children, no maxmemory)
    let min_info = r#"
# Server
redis_version:xedis-engine-1.0
uptime_in_seconds:120

# Memory
used_memory:524288000
used_memory_human:500.00 MB

# CPU
used_cpu_sys:12.4
used_cpu_user:30.1
"#;

    let metrics = TelemetryParser::parse_info(min_info, None, 1.0);

    assert_eq!(metrics.used_memory_display(), "500.00 MB");
    assert_eq!(metrics.rss_display(), "N/A");
    assert_eq!(metrics.max_memory_display(), "N/A");
    assert_eq!(metrics.frag_display(), "N/A");
    assert_eq!(metrics.main_cpu_display(), "Sys 0.0% / Usr 0.0%");
    assert_eq!(metrics.children_cpu_display(), "N/A");
    assert_eq!(metrics.ops_display(), "N/A");
    assert_eq!(metrics.net_display(), "N/A");
    assert_eq!(metrics.hit_rate_display(), "N/A");

    // Second sample 1.0s later with CPU activity
    let min_info2 = r#"
# CPU
used_cpu_sys:12.6
used_cpu_user:30.5
"#;
    let metrics2 = TelemetryParser::parse_info(min_info2, Some(&metrics), 1.0);
    // delta sys: 0.2 / 1s = 20.0%, delta usr: 0.4 / 1s = 40.0%
    assert_eq!(metrics2.main_cpu_display(), "Sys 20.0% / Usr 40.0%");
    assert!((metrics2.cpu_usage_pct - 60.0).abs() < 1e-6);
    assert_eq!(metrics2.children_cpu_display(), "N/A");
}

#[test]
fn test_custom_middleware_docker_flexible_formats() {
    // Test uppercase keys, equals separator, hyphenated names, whitespace, and unit suffixes
    let raw_custom_info = r#"
# Server
Server_Version: custom-xedis-v2.0
Operating_System: Alpine Linux (Docker)
uptime_seconds: 3600

# Memory (using equals and units)
Used_Memory = 512MB
RSS = 600MB
Max_Memory = 2GB
Fragmentation_Ratio = 1.17

# CPU (using hyphens and 's' suffix)
used-cpu-sys: 45.2s
used-cpu-user: 120.8s

# Stats
Clients = 42
QPS = 8500
"#;

    let metrics = TelemetryParser::parse_info(raw_custom_info, None, 1.0);

    assert_eq!(metrics.version, "custom-xedis-v2.0");
    assert_eq!(metrics.os, "Alpine Linux (Docker)");
    assert_eq!(metrics.uptime_in_seconds, 3600);
    assert_eq!(metrics.used_memory_bytes, Some(512 * 1024 * 1024));
    assert_eq!(metrics.used_memory_rss_bytes, Some(600 * 1024 * 1024));
    assert_eq!(metrics.max_memory_bytes, Some(2 * 1024 * 1024 * 1024));
    assert_eq!(metrics.mem_fragmentation_ratio, Some(1.17));
    assert_eq!(metrics.used_cpu_sys, Some(45.2));
    assert_eq!(metrics.used_cpu_user, Some(120.8));
    assert_eq!(metrics.connected_clients, 42);
    assert_eq!(metrics.instantaneous_ops_per_sec, Some(8500));
}

#[test]
fn test_metrics_history_buffer() {
    let mut history = MetricsHistory::new(5);
    history.push(1000, 10.0);
    history.push(2000, 20.0);
    history.push(3000, 30.0);
    history.push(4000, 40.0);
    history.push(5000, 50.0);
    history.push(6000, 60.0); // Should pop oldest (1000)

    let qps_slice = history.qps_as_slice();
    assert_eq!(qps_slice.len(), 5);
    assert_eq!(qps_slice[0], 2000);
    assert_eq!(qps_slice[4], 6000);
    assert_eq!(history.max_qps(), 6000);
    assert_eq!(history.avg_qps(), (2000 + 3000 + 4000 + 5000 + 6000) / 5);
}

#[test]
fn test_cluster_nodes_parsing_and_shards() {
    let nodes_raw = r#"
e01a1b2c3d4e5f60718293a4b5c6d7e8f9012345 127.0.0.1:6379@16379 master - 0 1788157290000 1 connected 0-5460
e04a1b2c3d4e5f60718293a4b5c6d7e8f9012345 127.0.0.1:6382@16382 slave e01a1b2c3d4e5f60718293a4b5c6d7e8f9012345 0 1788157290000 1 connected
e02a1b2c3d4e5f60718293a4b5c6d7e8f9012345 127.0.0.1:6380@16380 master - 0 1788157290000 2 connected 5461-10922
e05a1b2c3d4e5f60718293a4b5c6d7e8f9012345 127.0.0.1:6383@16383 slave e02a1b2c3d4e5f60718293a4b5c6d7e8f9012345 0 1788157290000 2 connected
e03a1b2c3d4e5f60718293a4b5c6d7e8f9012345 127.0.0.1:6381@16381 master - 0 1788157290000 3 connected 10923-16383
e06a1b2c3d4e5f60718293a4b5c6d7e8f9012345 127.0.0.1:6384@16384 slave e03a1b2c3d4e5f60718293a4b5c6d7e8f9012345 0 1788157290000 3 connected
"#;

    let topology = ClusterTopologyParser::parse_cluster_nodes(nodes_raw, 0.42);

    assert_eq!(topology.total_nodes, 6);
    assert_eq!(topology.healthy_nodes, 6);
    assert_eq!(topology.shards.len(), 3);
    assert_eq!(topology.covered_slots, 16384);
    assert!(topology.is_fully_covered);

    // Verify shard 1
    let shard1 = &topology.shards[0];
    assert_eq!(shard1.master.address, "127.0.0.1:6379");
    assert_eq!(shard1.master.cport, 16379);
    assert_eq!(shard1.total_slots, 5461);
    assert_eq!(shard1.replicas.len(), 1);
    assert_eq!(shard1.replicas[0].address, "127.0.0.1:6382");
}

#[test]
fn test_custom_fragmented_cluster_slots_adaptive() {
    // 5 Masters with arbitrary, irregular, multi-range and discrete slot distributions
    // Shard A: 0-1999, 8000-8999 (3000 slots)
    // Shard B: 2000-3999 (2000 slots)
    // Shard C: 4000-5999 (2000 slots)
    // Shard D: 6000-7999, 9000-9999 (3000 slots)
    // Shard E: 10000-16383 (6384 slots)
    // Total = 3000 + 2000 + 2000 + 3000 + 6384 = 16384 (100% full coverage)
    let nodes_raw = r#"
n05 10.0.0.5:6379@16379 master - 0 1788157290000 5 connected 10000-16383
n02 10.0.0.2:6379@16379 master - 0 1788157290000 2 connected 2000-3999
n04 10.0.0.4:6379@16379 master - 0 1788157290000 4 connected 6000-7999 9000-9999
n01 10.0.0.1:6379@16379 master - 0 1788157290000 1 connected 0-1999 8000-8999
n03 10.0.0.3:6379@16379 master - 0 1788157290000 3 connected 4000-5999
r01 10.0.0.11:6379@16379 slave n01 0 1788157290000 1 connected
"#;

    let topology = ClusterTopologyParser::parse_cluster_nodes(nodes_raw, 0.5);

    assert_eq!(topology.total_nodes, 6);
    assert_eq!(topology.shards.len(), 5);
    assert_eq!(topology.covered_slots, 16384);
    assert!(topology.is_fully_covered);

    // Sorted by first slot range start:
    // Shard 1: n01 (starts at 0)
    assert_eq!(topology.shards[0].master.id, "n01");
    assert_eq!(topology.shards[0].total_slots, 3000);
    assert_eq!(topology.shards[0].slot_ranges, vec![(0, 1999), (8000, 8999)]);
    assert_eq!(topology.shards[0].replicas.len(), 1);
    assert_eq!(topology.shards[0].replicas[0].id, "r01");

    // Shard 2: n02 (starts at 2000)
    assert_eq!(topology.shards[1].master.id, "n02");
    assert_eq!(topology.shards[1].total_slots, 2000);

    // Shard 3: n03 (starts at 4000)
    assert_eq!(topology.shards[2].master.id, "n03");
    assert_eq!(topology.shards[2].total_slots, 2000);

    // Shard 4: n04 (starts at 6000)
    assert_eq!(topology.shards[3].master.id, "n04");
    assert_eq!(topology.shards[3].total_slots, 3000);

    // Shard 5: n05 (starts at 10000)
    assert_eq!(topology.shards[4].master.id, "n05");
    assert_eq!(topology.shards[4].total_slots, 6384);
}

#[test]
fn test_standalone_replication_parsing() {
    let repl_raw = r#"
# Replication
role:master
connected_slaves:2
slave0:ip=127.0.0.1,port=6380,state=online,offset=142980,lag=0
slave1:ip=127.0.0.1,port=6381,state=online,offset=142980,lag=1
master_repl_offset:142980
"#;

    let repl_info = ClusterTopologyParser::parse_info_replication(repl_raw);

    assert_eq!(repl_info.role, "master");
    assert_eq!(repl_info.connected_slaves, 2);
    assert_eq!(repl_info.slaves.len(), 2);
    assert_eq!(repl_info.slaves[0].port, 6380);
    assert_eq!(repl_info.slaves[1].lag, 1);
    assert_eq!(repl_info.master_repl_offset, 142980);
}

#[tokio::test]
async fn test_dynamic_polling_interval_macro() {
    let config = AppConfig::default();
    let mut app = App::new(config).await;

    assert_eq!(app.poll_interval, Duration::from_millis(1000));
    assert!(!app.is_poll_paused);

    // Change to 500ms
    app.input_buffer = "/interval 500ms".to_string();
    app.submit_command().await;
    assert_eq!(app.poll_interval, Duration::from_millis(500));
    assert!(!app.is_poll_paused);

    // Change to 2s
    app.input_buffer = "/interval 2s".to_string();
    app.submit_command().await;
    assert_eq!(app.poll_interval, Duration::from_secs(2));
    assert!(!app.is_poll_paused);

    // Pause polling
    app.input_buffer = "/interval pause".to_string();
    app.submit_command().await;
    assert!(app.is_poll_paused);

    // Resume polling
    app.input_buffer = "/interval resume".to_string();
    app.submit_command().await;
    assert!(!app.is_poll_paused);
}

#[tokio::test]
async fn test_settings_macro() {
    let config = AppConfig::default();
    let mut app = App::new(config).await;

    app.input_buffer = "/settings".to_string();
    app.submit_command().await;

    let last_record = app.records.last().expect("Record should exist");
    assert_eq!(last_record.command, "/settings");
    match &last_record.result {
        FormattedValue::Table { headers, rows } => {
            assert!(headers.contains(&"Configuration Item".to_string()));
            assert!(rows.iter().any(|r| r[0] == "Server Address"));
            assert!(rows.iter().any(|r| r[0] == "Telemetry Polling"));
        }
        _ => panic!("Expected Table result for /settings"),
    }
}

#[test]
fn test_slowlog_smart_suggestions() {
    let keys_sug = MacroEngine::suggest_for_slow_command("KEYS user:*");
    assert!(keys_sug.is_some());
    assert!(keys_sug.unwrap().contains("/scan"));

    let eval_sug = MacroEngine::suggest_for_slow_command("EVAL sha1 0");
    assert!(eval_sug.is_some());
    assert!(eval_sug.unwrap().contains("Lua"));

    let zrange_sug = MacroEngine::suggest_for_slow_command("ZRANGEBYSCORE leaderboard 0 10000");
    assert!(zrange_sug.is_some());
    assert!(zrange_sug.unwrap().contains("LIMIT"));

    let sort_sug = MacroEngine::suggest_for_slow_command("SORT mylist ALPHA");
    assert!(sort_sug.is_some());
    assert!(sort_sug.unwrap().contains("SORT"));
}

#[tokio::test]
async fn test_right_dashboard_focus_and_scrolling() {
    let config = AppConfig::default();
    let mut app = App::new(config).await;

    assert_eq!(app.focused_pane, FocusedPane::LeftStream);
    assert_eq!(app.active_tab, ActiveRightTab::Telemetry);

    // Switch focus with Tab
    app.handle_key(key(KeyCode::Tab)).await;
    assert_eq!(app.focused_pane, FocusedPane::RightDashboard);

    // Switch tab with F3 (Cluster)
    app.handle_key(key(KeyCode::F(3))).await;
    assert_eq!(app.active_tab, ActiveRightTab::Cluster);

    // Scroll Down & Up in Cluster tab (3 shards -> max offset 2)
    assert_eq!(app.cluster_scroll_offset, 0);
    app.handle_key(key(KeyCode::Down)).await;
    assert_eq!(app.cluster_scroll_offset, 1);
    app.handle_key(key(KeyCode::PageDown)).await;
    assert_eq!(app.cluster_scroll_offset, 2); // Clamped to max shard index 2
    app.handle_key(key(KeyCode::Up)).await;
    assert_eq!(app.cluster_scroll_offset, 1);
    app.handle_key(key(KeyCode::Home)).await;
    assert_eq!(app.cluster_scroll_offset, 0);

    // Switch tab with F4 (Slowlog)
    app.handle_key(key(KeyCode::F(4))).await;
    assert_eq!(app.active_tab, ActiveRightTab::Slowlog);

    // Scroll Down & Up in Slowlog tab (3 entries -> max offset 2)
    assert_eq!(app.slowlog_scroll_offset, 0);
    app.handle_key(key(KeyCode::Down)).await;
    assert_eq!(app.slowlog_scroll_offset, 1);
    app.handle_key(key(KeyCode::PageDown)).await;
    assert_eq!(app.slowlog_scroll_offset, 2); // Clamped to max entry index 2
    app.handle_key(key(KeyCode::PageUp)).await;
    assert_eq!(app.slowlog_scroll_offset, 0);
    app.handle_key(key(KeyCode::Home)).await;
    assert_eq!(app.slowlog_scroll_offset, 0);

    // Test Tab switching with Left/Right and Number keys
    app.handle_key(key(KeyCode::Char('1'))).await;
    assert_eq!(app.active_tab, ActiveRightTab::Telemetry);

    app.handle_key(key(KeyCode::Right)).await;
    assert_eq!(app.active_tab, ActiveRightTab::Cluster);

    app.handle_key(key(KeyCode::Right)).await;
    assert_eq!(app.active_tab, ActiveRightTab::Slowlog);

    app.handle_key(key(KeyCode::Left)).await;
    assert_eq!(app.active_tab, ActiveRightTab::Cluster);

    app.handle_key(key(KeyCode::Char('3'))).await;
    assert_eq!(app.active_tab, ActiveRightTab::Slowlog);

    // Return to stream focus with Esc
    app.handle_key(key(KeyCode::Esc)).await;
    assert_eq!(app.focused_pane, FocusedPane::LeftStream);
}

#[tokio::test]
async fn test_stream_scroll_clamping_no_phantom_distance() {
    let config = AppConfig::default();
    let mut app = App::new(config).await;

    // Initially at bottom
    assert_eq!(app.scroll_offset, 0);

    // Press PageUp multiple times (e.g. 10 times)
    for _ in 0..10 {
        app.handle_key(key(KeyCode::PageUp)).await;
    }

    // Scroll offset must be clamped strictly to total_lines - 1, without accumulating invisible phantom distance
    let total_lines = xedis_tui::ui::stream_view::StreamView::total_lines_count(&app.records);
    let max_scroll = total_lines.saturating_sub(1);
    assert_eq!(app.scroll_offset, max_scroll);

    // Immediately on the very next PageDown, it should decrease immediately without delay
    app.handle_key(key(KeyCode::PageDown)).await;
    assert_eq!(app.scroll_offset, max_scroll.saturating_sub(6));
}
