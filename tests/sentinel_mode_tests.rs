use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::collections::HashMap;
use xedis_tui::backend::cluster_info::{
    ClusterTopology, ClusterTopologyParser, RedisTopologyMode,
};
use xedis_tui::backend::formatter::FormattedValue;
use xedis_tui::core::autocomplete::{AutocompleteEngine, SuggestionKind};
use xedis_tui::core::macro_engine::MacroEngine;
use xedis_tui::ui::cluster_view::ClusterView;
use xedis_tui::ui::theme::ThemePalette;

#[test]
fn test_parse_info_sentinel() {
    let raw_info = r#"
# Sentinel
sentinel_masters:2
sentinel_tilt:0
sentinel_running_scripts:1
sentinel_scripts_queue_length:0
sentinel_simulate_failure_flags:0
master0:name=mymaster,status=ok,address=127.0.0.1:6379,slaves=2,sentinels=3
master1:name=resque,status=ok,address=192.168.1.50:6379,slaves=1,sentinels=3
"#;

    let topo = ClusterTopologyParser::parse_info_sentinel(raw_info);

    assert_eq!(topo.masters.len(), 2);
    assert_eq!(topo.total_sentinels, 3);
    assert!(!topo.tilt_mode);
    assert_eq!(topo.running_scripts, 1);

    let m0 = &topo.masters[0];
    assert_eq!(m0.name, "mymaster");
    assert_eq!(m0.ip, "127.0.0.1");
    assert_eq!(m0.port, 6379);
    assert_eq!(m0.status, "ok");
    assert_eq!(m0.num_slaves, 2);
    assert_eq!(m0.num_other_sentinels, 2);

    let m1 = &topo.masters[1];
    assert_eq!(m1.name, "resque");
    assert_eq!(m1.ip, "192.168.1.50");
    assert_eq!(m1.port, 6379);
    assert_eq!(m1.status, "ok");
    assert_eq!(m1.num_slaves, 1);
}

#[test]
fn test_parse_sentinel_entries() {
    // 1. Masters
    let mut master_map = HashMap::new();
    master_map.insert("name".to_string(), "prod-db".to_string());
    master_map.insert("ip".to_string(), "10.0.1.10".to_string());
    master_map.insert("port".to_string(), "6379".to_string());
    master_map.insert("flags".to_string(), "master,sdown".to_string());
    master_map.insert("quorum".to_string(), "2".to_string());
    master_map.insert("num-slaves".to_string(), "3".to_string());
    master_map.insert("num-other-sentinels".to_string(), "2".to_string());
    master_map.insert("down-after-milliseconds".to_string(), "15000".to_string());
    master_map.insert("failover-timeout".to_string(), "120000".to_string());

    let masters = ClusterTopologyParser::parse_sentinel_masters_entries(&[master_map]);
    assert_eq!(masters.len(), 1);
    assert_eq!(masters[0].name, "prod-db");
    assert_eq!(masters[0].ip, "10.0.1.10");
    assert_eq!(masters[0].port, 6379);
    assert_eq!(masters[0].status, "sdown");
    assert_eq!(masters[0].quorum, 2);
    assert_eq!(masters[0].num_slaves, 3);
    assert_eq!(masters[0].down_after_ms, 15000);
    assert_eq!(masters[0].failover_timeout_ms, 120000);

    // 2. Slaves
    let mut slave_map = HashMap::new();
    slave_map.insert("ip".to_string(), "10.0.1.11".to_string());
    slave_map.insert("port".to_string(), "6380".to_string());
    slave_map.insert("flags".to_string(), "slave".to_string());
    slave_map.insert("master-link-status".to_string(), "ok".to_string());
    slave_map.insert("slave-repl-offset".to_string(), "2458900".to_string());
    slave_map.insert("master-link-down-time".to_string(), "0".to_string());

    let slaves = ClusterTopologyParser::parse_sentinel_slaves_entries(&[slave_map]);
    assert_eq!(slaves.len(), 1);
    assert_eq!(slaves[0].ip, "10.0.1.11");
    assert_eq!(slaves[0].port, 6380);
    assert_eq!(slaves[0].link_status, "ok");
    assert_eq!(slaves[0].repl_offset, 2458900);
    assert_eq!(slaves[0].lag_sec, 0);

    // 3. Peers
    let mut peer_map = HashMap::new();
    peer_map.insert("name".to_string(), "sentinel-peer-01".to_string());
    peer_map.insert("ip".to_string(), "10.0.1.20".to_string());
    peer_map.insert("port".to_string(), "26379".to_string());
    peer_map.insert("flags".to_string(), "sentinel".to_string());
    peer_map.insert("last-ok-ping-reply".to_string(), "140".to_string());

    let peers = ClusterTopologyParser::parse_sentinel_peers_entries(&[peer_map]);
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].id, "sentinel-peer-01");
    assert_eq!(peers[0].ip, "10.0.1.20");
    assert_eq!(peers[0].port, 26379);
    assert!(peers[0].is_healthy);
    assert_eq!(peers[0].last_ok_ping_ms, 140);
}

#[test]
fn test_mock_sentinel_topology_integrity() {
    let topology = ClusterTopology::mock_sentinel_topology();

    assert_eq!(topology.mode, RedisTopologyMode::Sentinel);
    assert!(!topology.is_cluster);
    assert!(topology.sentinel.is_some());

    let sent = topology.sentinel.as_ref().unwrap();
    assert_eq!(sent.masters.len(), 1);
    assert_eq!(sent.total_sentinels, 3);
    assert!(!sent.tilt_mode);

    let master = &sent.masters[0];
    assert_eq!(master.name, "mymaster");
    assert_eq!(master.status, "ok");
    assert_eq!(master.quorum, 2);
    assert_eq!(master.slaves.len(), 2);
    assert_eq!(master.sentinels.len(), 2);
}

#[test]
fn test_sentinel_autocomplete() {
    // 1. Typing 'SENTINEL ' should suggest subcommands like MASTERS, SLAVES, etc.
    let (items, range) = AutocompleteEngine::get_suggestions("SENTINEL ", 9, &[]);
    assert!(!items.is_empty());
    assert!(items.iter().all(|it| it.kind == SuggestionKind::Subcommand));
    let names: Vec<&str> = items.iter().map(|it| it.completion_text.trim()).collect();
    assert!(names.contains(&"masters"));
    assert!(names.contains(&"slaves"));
    assert!(names.contains(&"sentinels"));
    assert!(names.contains(&"ckquorum"));
    assert!(names.contains(&"failover"));
    assert_eq!(range, (9, 9));

    // 2. Typing 'INFO ' should include SENTINEL section
    let (items, _) = AutocompleteEngine::get_suggestions("INFO ", 5, &[]);
    let info_sections: Vec<&str> = items.iter().map(|it| it.completion_text.trim()).collect();
    assert!(info_sections.contains(&"sentinel"));
}

#[test]
fn test_sentinel_settings_macro() {
    let settings = MacroEngine::format_settings_with_mode(
        "127.0.0.1",
        26379,
        "Sentinel Mode",
        "Balanced",
        "Dark",
        1000,
        false,
    );

    match settings {
        FormattedValue::Table { headers, rows } => {
            assert_eq!(headers, vec!["Configuration Item", "Current Value", "Description"]);
            assert!(rows.iter().any(|r| r[0] == "Protocol Mode" && r[1] == "Sentinel Mode"));
        }
        _ => panic!("Expected FormattedValue::Table"),
    }
}

#[test]
fn test_sentinel_cluster_view_rendering_multiple_widths() {
    let topology = ClusterTopology::mock_sentinel_topology();
    let theme = ThemePalette::dark();

    // 1. Wide terminal (100 cols x 30 rows)
    let backend_wide = TestBackend::new(100, 30);
    let mut term_wide = Terminal::new(backend_wide).unwrap();
    term_wide
        .draw(|f| {
            ClusterView::render(f, f.area(), &topology, 0, &theme);
        })
        .unwrap();

    let buffer_wide = term_wide.backend().buffer();
    let content_wide = buffer_wide
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    assert!(content_wide.contains("Sentinel Topology"));
    assert!(content_wide.contains("mymaster"));
    assert!(content_wide.contains("127.0.0.1:6379"));
    assert!(content_wide.contains("Quorum: 2 (3 in total)"));
    assert!(content_wide.contains("[Myself] 127.0.0.1:26379"));
    assert!(content_wide.contains("[Peer #1] 127.0.0.1:26380"));

    // 2. Medium terminal (65 cols x 30 rows)
    let backend_med = TestBackend::new(65, 30);
    let mut term_med = Terminal::new(backend_med).unwrap();
    term_med
        .draw(|f| {
            ClusterView::render(f, f.area(), &topology, 0, &theme);
        })
        .unwrap();

    let buffer_med = term_med.backend().buffer();
    let content_med = buffer_med
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    assert!(content_med.contains("mymaster"));
    assert!(content_med.contains("[Myself]"));
    assert!(content_med.contains("2 (3 in total)"));

    // 3. Compact terminal (45 cols x 30 rows)
    let backend_compact = TestBackend::new(45, 30);
    let mut term_compact = Terminal::new(backend_compact).unwrap();
    term_compact
        .draw(|f| {
            ClusterView::render(f, f.area(), &topology, 0, &theme);
        })
        .unwrap();

    let buffer_compact = term_compact.backend().buffer();
    let content_compact = buffer_compact
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    assert!(content_compact.contains("mymaster"));
    assert!(content_compact.contains("[Myself]"));
}

#[test]
fn test_sentinel_border_color_consistency() {
    let topology = ClusterTopology::mock_sentinel_topology();
    let theme = ThemePalette::dark();

    let backend = TestBackend::new(100, 30);
    let mut term = Terminal::new(backend).unwrap();
    term.draw(|f| {
        ClusterView::render(f, f.area(), &topology, 0, &theme);
    })
    .unwrap();

    let buffer = term.backend().buffer();

    // Inspect each row in the buffer
    let mut vertical_bar_found = 0;
    let mut myself_cell_found = false;
    let mut peer_cell_found = false;

    for y in 0..buffer.area.height {
        // Collect characters on this row
        let row_chars: Vec<(u16, &ratatui::buffer::Cell)> = (0..buffer.area.width)
            .map(|x| (x, &buffer[(x, y)]))
            .collect();

        let row_str: String = row_chars.iter().map(|(_, c)| c.symbol()).collect();

        // Check if this row contains the master card outer border '│'
        if row_str.contains('│') {
            // Find the first '│' on this row which is the outer left border
            if let Some((_x, cell)) = row_chars.iter().find(|(_, c)| c.symbol() == "│") {
                vertical_bar_found += 1;
                // Verify that outer border strictly uses sentinel_border color,
                // and is NOT tainted by shard_replica_title or sentinel_peer_title or text_muted
                assert_eq!(
                    cell.fg, theme.sentinel_border,
                    "Outer border at line {} (row: {:?}) had color {:?}, expected sentinel_border {:?}",
                    y, row_str.trim(), cell.fg, theme.sentinel_border
                );
            }
        }

        // Verify [Myself] text uses sentinel_myself_title
        if row_str.contains("[Myself]") {
            if let Some(pos) = row_str.find("[Myself]") {
                let cell = &buffer[(pos as u16, y)];
                assert_eq!(cell.fg, theme.sentinel_myself_title, "Expected [Myself] to use sentinel_myself_title");
                myself_cell_found = true;
            }
        }

        // Verify [Peer #1] text uses sentinel_peer_title
        if row_str.contains("[Peer #1]") {
            if let Some(pos) = row_str.find("[Peer #1]") {
                let cell = &buffer[(pos as u16, y)];
                assert_eq!(cell.fg, theme.sentinel_peer_title, "Expected [Peer #1] to use sentinel_peer_title");
                peer_cell_found = true;
            }
        }
    }

    // Ensure we actually checked multiple rows of the master card
    assert!(vertical_bar_found >= 6, "Expected at least 6 rows with outer border '│', found {}", vertical_bar_found);
    assert!(myself_cell_found, "Expected [Myself] to be rendered and verified");
    assert!(peer_cell_found, "Expected [Peer #1] to be rendered and verified");
}
