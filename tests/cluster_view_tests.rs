use ratatui::backend::TestBackend;
use ratatui::Terminal;
use xedis_tui::backend::cluster_info::{
    ClusterHealthStatus, ClusterNode, ClusterShard, ClusterTopology, ClusterTopologyParser,
    NodeHealthState, RedisTopologyMode, SlotMigration, SlotMigrationType,
};
use xedis_tui::ui::cluster_view::ClusterView;
use xedis_tui::ui::theme::ThemePalette;

fn create_test_cluster_topology() -> ClusterTopology {
    ClusterTopology {
        mode: RedisTopologyMode::Cluster,
        is_cluster: true,
        total_nodes: 6,
        healthy_nodes: 5,
        covered_slots: 16384,
        is_fully_covered: true,
        cluster_state_ok: true,
        health_status: ClusterHealthStatus::Degraded,
        health_summary: "1 PFAIL".to_string(),
        shards: vec![
            ClusterShard {
                shard_index: 1,
                master: ClusterNode {
                    id: "8f5851b8".to_string(),
                    raw_id: "8f5851b81234567890abcdef".to_string(),
                    address: "127.0.0.1:22001".to_string(),
                    cport: 32001,
                    role: "Master".to_string(),
                    master_id: None,
                    is_healthy: true,
                    health_state: NodeHealthState::Healthy,
                    ping_ms: 3.0,
                    slots_raw: "0-5460".to_string(),
                    slot_ranges: vec![(0, 5460)],
                    slot_count: 5461,
                    key_count: 100,
                    migrations: vec![SlotMigration {
                        slot: 5460,
                        migration_type: SlotMigrationType::Migrating,
                        remote_node_id: "a1b2c3d4".to_string(),
                        remote_node_repr: "@a1b2c3d4 (127.0.0.1:22003)".to_string(),
                    }],
                    repl_offset: Some(145_210),
                },
                replicas: vec![
                    ClusterNode {
                        id: "ecbf04b1".to_string(),
                        raw_id: "ecbf04b11234567890abcdef".to_string(),
                        address: "127.0.0.1:22002".to_string(),
                        cport: 32002,
                        role: "Replica".to_string(),
                        master_id: Some("8f5851b8".to_string()),
                        is_healthy: true,
                        health_state: NodeHealthState::Healthy,
                        ping_ms: 3.0,
                        slots_raw: "".to_string(),
                        slot_ranges: vec![],
                        slot_count: 0,
                        key_count: 0,
                        migrations: vec![],
                        repl_offset: Some(145_210),
                    },
                ],
                slot_ranges: vec![(0, 5460)],
                total_slots: 5461,
                key_count: 100,
            },
            ClusterShard {
                shard_index: 2,
                master: ClusterNode {
                    id: "a1b2c3d4".to_string(),
                    raw_id: "a1b2c3d41234567890abcdef".to_string(),
                    address: "127.0.0.1:22003".to_string(),
                    cport: 32003,
                    role: "Master".to_string(),
                    master_id: None,
                    is_healthy: true,
                    health_state: NodeHealthState::Healthy,
                    ping_ms: 2.5,
                    slots_raw: "5461-10922".to_string(),
                    slot_ranges: vec![(5461, 10922)],
                    slot_count: 5462,
                    key_count: 120,
                    migrations: vec![SlotMigration {
                        slot: 5460,
                        migration_type: SlotMigrationType::Importing,
                        remote_node_id: "8f5851b8".to_string(),
                        remote_node_repr: "@8f5851b8 (127.0.0.1:22001)".to_string(),
                    }],
                    repl_offset: Some(139_400),
                },
                replicas: vec![
                    ClusterNode {
                        id: "rep2a".to_string(),
                        raw_id: "rep2a1234567890abcdef".to_string(),
                        address: "127.0.0.1:22004".to_string(),
                        cport: 32004,
                        role: "Replica".to_string(),
                        master_id: Some("a1b2c3d4".to_string()),
                        is_healthy: true,
                        health_state: NodeHealthState::Healthy,
                        ping_ms: 2.8,
                        slots_raw: "".to_string(),
                        slot_ranges: vec![],
                        slot_count: 0,
                        key_count: 0,
                        migrations: vec![],
                        repl_offset: Some(139_400),
                    },
                    ClusterNode {
                        id: "rep2b".to_string(),
                        raw_id: "rep2b1234567890abcdef".to_string(),
                        address: "127.0.0.1:22005".to_string(),
                        cport: 32005,
                        role: "Replica".to_string(),
                        master_id: Some("a1b2c3d4".to_string()),
                        is_healthy: false,
                        health_state: NodeHealthState::Pfail,
                        ping_ms: 120.0,
                        slots_raw: "".to_string(),
                        slot_ranges: vec![],
                        slot_count: 0,
                        key_count: 0,
                        migrations: vec![],
                        repl_offset: Some(137_170),
                    },
                ],
                slot_ranges: vec![(5461, 10922)],
                total_slots: 5462,
                key_count: 120,
            },
        ],
        standalone_nodes: vec![],
        replication: None,
        sentinel: None,
    }
}

#[test]
fn test_cluster_view_borders_alignment() {
    let topology = create_test_cluster_topology();
    let theme = ThemePalette::dark();

    for width in [100, 80, 65, 50, 35] {
        let backend = TestBackend::new(width, 40);
        let mut term = Terminal::new(backend).unwrap();

        term.draw(|f| {
            ClusterView::render(f, f.area(), &topology, 0, &theme);
        })
        .unwrap();

        let buffer = term.backend().buffer();

        // Check each shard card in the buffer:
        // Find rows with shard header ' ╭── Shard #' and shard bottom ' ╰'
        let mut shard_top_rights: Vec<(u16, u16)> = Vec::new();
        let mut shard_bot_rights: Vec<(u16, u16)> = Vec::new();

        for y in 0..buffer.area.height {
            let row_chars: Vec<(u16, String)> = (0..buffer.area.width)
                .map(|x| (x, buffer[(x, y)].symbol().to_string()))
                .collect();
            let row_str: String = row_chars.iter().map(|(_, s)| s.clone()).collect();

            // Shard top border
            if row_str.contains("╭── Shard #") {
                if let Some((x, _)) = row_chars.iter().rfind(|(_, s)| s == "╮") {
                    shard_top_rights.push((*x, y));
                }
            }

            // Shard bottom border: row_chars[1] is ' ' and row_chars[2] is '╰' (inner boxes have '│' at row_chars[2])
            if row_chars.len() > 2 && row_chars[1].1 == " " && row_chars[2].1 == "╰" {
                if let Some((x, _)) = row_chars.iter().rfind(|(_, s)| s == "╯") {
                    shard_bot_rights.push((*x, y));
                }
            }
        }

        assert!(!shard_top_rights.is_empty(), "Expected shard top borders at width {}", width);
        assert_eq!(
            shard_top_rights.len(),
            shard_bot_rights.len(),
            "Expected equal number of top and bottom borders at width {}",
            width
        );

        for (i, (top_x, _)) in shard_top_rights.iter().enumerate() {
            let (bot_x, _) = shard_bot_rights[i];
            assert_eq!(
                *top_x, bot_x,
                "Shard #{} top-right corner (x={}) did not align with bottom-right corner (x={}) at width {}",
                i + 1,
                top_x,
                bot_x,
                width
            );
        }
    }
}

#[test]
fn test_cluster_view_node_boxes_structure() {
    let topology = create_test_cluster_topology();
    let theme = ThemePalette::dark();

    let backend = TestBackend::new(90, 45);
    let mut term = Terminal::new(backend).unwrap();

    term.draw(|f| {
        ClusterView::render(f, f.area(), &topology, 0, &theme);
    })
    .unwrap();

    let buffer = term.backend().buffer();
    let full_content: String = (0..buffer.area.height)
        .map(|y| {
            let row: String = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            row
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Verify outer shard cards
    assert!(full_content.contains("Shard #1"));
    assert!(full_content.contains("Shard #2"));

    // Verify dedicated node boxes for Master
    assert!(full_content.contains("Master #1"));
    assert!(full_content.contains("Master #2"));

    // Verify dedicated node boxes for Replicas
    assert!(full_content.contains("Replica #1"));
    assert!(full_content.contains("Replica #2"));

    // Verify node information is present
    assert!(full_content.contains("@8f5851b8"));
    assert!(full_content.contains("127.0.0.1:22001"));
    assert!(full_content.contains("[HEALTHY]"));
    assert!(full_content.contains("Slots: 0-5460 (5461 slots)"));

    // Verify tree connectors
    assert!(full_content.contains("└── ╭── Replica"));
    assert!(full_content.contains("├── ╭── Replica"));
}

#[test]
fn test_cluster_view_master_and_replica_right_borders_align() {
    let topology = create_test_cluster_topology();
    let theme = ThemePalette::dark();

    let backend = TestBackend::new(85, 45);
    let mut term = Terminal::new(backend).unwrap();

    term.draw(|f| {
        ClusterView::render(f, f.area(), &topology, 0, &theme);
    })
    .unwrap();

    let buffer = term.backend().buffer();

    let mut master1_right_x = None;
    let mut replica1_right_x = None;

    for y in 0..buffer.area.height {
        let row_chars: Vec<(u16, String)> = (0..buffer.area.width)
            .map(|x| (x, buffer[(x, y)].symbol().to_string()))
            .collect();
        let row_str: String = row_chars.iter().map(|(_, s)| s.clone()).collect();

        if row_str.contains("Master #1") {
            // Find the ╮ for Master #1 inner box
            if let Some((x, _)) = row_chars.iter().find(|(_, s)| s == "╮") {
                master1_right_x = Some(*x);
            }
        }

        if row_str.contains("Replica #1") && !row_str.contains("Shard #1") {
            // Find the ╮ for Replica #1 inner box
            if let Some((x, _)) = row_chars.iter().find(|(_, s)| s == "╮") {
                replica1_right_x = Some(*x);
            }
        }
    }

    assert!(master1_right_x.is_some(), "Master #1 inner box corner not found");
    assert!(replica1_right_x.is_some(), "Replica #1 inner box corner not found");
    assert_eq!(
        master1_right_x.unwrap(),
        replica1_right_x.unwrap(),
        "Master #1 inner box right border and Replica #1 inner box right border should align vertically"
    );
}

#[test]
fn test_cluster_view_narrow_widths_no_panic() {
    let topology = create_test_cluster_topology();
    let theme = ThemePalette::dark();

    for w in [55, 42, 32, 28] {
        let backend = TestBackend::new(w, 40);
        let mut term = Terminal::new(backend).unwrap();

        let res = term.draw(|f| {
            ClusterView::render(f, f.area(), &topology, 0, &theme);
        });
        assert!(res.is_ok(), "Rendering failed at narrow width {}", w);

        let buffer = term.backend().buffer();
        // Ensure every character position is within bounds
        assert_eq!(buffer.area.width, w);
    }
}

#[test]
fn test_standalone_and_sentinel_border_alignment() {
    let sentinel_topo = ClusterTopology::mock_sentinel_topology();
    let theme = ThemePalette::dark();

    for width in [90, 65, 45] {
        let backend = TestBackend::new(width, 30);
        let mut term = Terminal::new(backend).unwrap();

        term.draw(|f| {
            ClusterView::render(f, f.area(), &sentinel_topo, 0, &theme);
        })
        .unwrap();

        let buffer = term.backend().buffer();

        let mut top_x = None;
        let mut bot_x = None;

        for y in 0..buffer.area.height {
            let row_chars: Vec<(u16, String)> = (0..buffer.area.width)
                .map(|x| (x, buffer[(x, y)].symbol().to_string()))
                .collect();
            let row_str: String = row_chars.iter().map(|(_, s)| s.clone()).collect();

            if row_str.contains("Master: ") {
                if let Some((x, _)) = row_chars.iter().rfind(|(_, s)| s == "╮") {
                    top_x = Some(*x);
                }
            }
            if row_str.contains(" ╰─") {
                if let Some((x, _)) = row_chars.iter().rfind(|(_, s)| s == "╯") {
                    bot_x = Some(*x);
                }
            }
        }

        assert!(top_x.is_some(), "Sentinel top border not found at width {}", width);
        assert!(bot_x.is_some(), "Sentinel bottom border not found at width {}", width);
        assert_eq!(top_x.unwrap(), bot_x.unwrap(), "Sentinel borders did not align at width {}", width);
    }
}

#[test]
fn test_parse_pfail_and_fail_nodes() {
    let raw = r#"
node-1 127.0.0.1:6379@16379 master - 0 1700000000 1 connected 0-5460
node-2 127.0.0.1:6380@16380 master,fail? - 0 1700000000 2 connected 5461-10922
node-3 127.0.0.1:6381@16381 master - 0 1700000000 3 connected 10923-16383
node-4 127.0.0.1:6382@16382 slave node-1 0 1700000000 1 connected
node-5 127.0.0.1:6383@16383 slave,fail node-3 0 1700000000 3 disconnected
"#;

    let topo = ClusterTopologyParser::parse_cluster_nodes(raw, 0.5);
    assert_eq!(topo.shards.len(), 3);

    // node-1 is healthy
    assert_eq!(topo.shards[0].master.health_state, NodeHealthState::Healthy);
    assert!(topo.shards[0].master.is_healthy);

    // node-2 has fail? -> Pfail
    assert_eq!(topo.shards[1].master.health_state, NodeHealthState::Pfail);
    assert!(!topo.shards[1].master.is_healthy);

    // node-5 has fail -> Fail
    assert_eq!(topo.shards[2].replicas[0].health_state, NodeHealthState::Fail);
    assert!(!topo.shards[2].replicas[0].is_healthy);

    // Overall cluster health: node-2 is PFAIL, node-5 is FAIL -> Degraded
    assert_eq!(topo.health_status, ClusterHealthStatus::Degraded);
}

#[test]
fn test_parse_slot_migration_syntax() {
    let raw = r#"
e01a1b2c3d4e5f60718293a4b5c6d7e8f9012345 127.0.0.1:6379@16379 master - 0 1700000000 1 connected 0-5460 [5461->-e02a1b2c3d4e5f60718293a4b5c6d7e8f9012345]
e02a1b2c3d4e5f60718293a4b5c6d7e8f9012345 127.0.0.1:6380@16380 master - 0 1700000000 2 connected 5462-10922 [5461-<-e01a1b2c3d4e5f60718293a4b5c6d7e8f9012345]
node-3 127.0.0.1:6381@16381 master - 0 1700000000 3 connected 10923-16383
"#;

    let topo = ClusterTopologyParser::parse_cluster_nodes(raw, 0.4);
    assert_eq!(topo.shards.len(), 3);

    // Master 1 has migrating slot 5461 to Master 2
    let m1 = &topo.shards[0].master;
    assert_eq!(m1.migrations.len(), 1);
    assert_eq!(m1.migrations[0].slot, 5461);
    assert_eq!(m1.migrations[0].migration_type, SlotMigrationType::Migrating);
    assert!(m1.migrations[0].remote_node_repr.contains("@e02a1b2c"));
    assert!(m1.migrations[0].remote_node_repr.contains("127.0.0.1:6380"));

    // Master 2 has importing slot 5461 from Master 1
    let m2 = &topo.shards[1].master;
    assert_eq!(m2.migrations.len(), 1);
    assert_eq!(m2.migrations[0].slot, 5461);
    assert_eq!(m2.migrations[0].migration_type, SlotMigrationType::Importing);
    assert!(m2.migrations[0].remote_node_repr.contains("@e01a1b2c"));
    assert!(m2.migrations[0].remote_node_repr.contains("127.0.0.1:6379"));

    // Active migrations cause Degraded state
    assert_eq!(topo.health_status, ClusterHealthStatus::Degraded);
}

#[test]
fn test_cluster_health_status_evaluation() {
    // 1. Healthy cluster
    let healthy_raw = r#"
node-1 127.0.0.1:6379@16379 master - 0 1700000000 1 connected 0-5460
node-4 127.0.0.1:6382@16382 slave node-1 0 1700000000 1 connected
node-2 127.0.0.1:6380@16380 master - 0 1700000000 2 connected 5461-10922
node-5 127.0.0.1:6383@16383 slave node-2 0 1700000000 2 connected
node-3 127.0.0.1:6381@16381 master - 0 1700000000 3 connected 10923-16383
node-6 127.0.0.1:6384@16384 slave node-3 0 1700000000 3 connected
"#;
    let healthy_topo = ClusterTopologyParser::parse_cluster_nodes(healthy_raw, 0.4);
    assert_eq!(healthy_topo.health_status, ClusterHealthStatus::Healthy);

    // 2. Failed cluster due to cluster_state: fail
    let failed_topo = ClusterTopologyParser::parse_cluster_nodes_with_info(healthy_raw, 0.4, Some(false));
    assert_eq!(failed_topo.health_status, ClusterHealthStatus::Failed);

    // 3. Failed cluster due to missing slots
    let partial_raw = r#"
node-1 127.0.0.1:6379@16379 master - 0 1700000000 1 connected 0-5460
node-2 127.0.0.1:6380@16380 master - 0 1700000000 2 connected 5461-10922
"#;
    let partial_topo = ClusterTopologyParser::parse_cluster_nodes(partial_raw, 0.4);
    assert_eq!(partial_topo.health_status, ClusterHealthStatus::Failed);
}

#[test]
fn test_render_all_health_states_and_borders() {
    let theme = ThemePalette::dark();
    let healthy_topo = ClusterTopology::mock_healthy_cluster_topology();
    let degraded_topo = ClusterTopology::mock_cluster_topology();
    let failed_topo = ClusterTopology::mock_failed_cluster_topology();

    for topo in [&healthy_topo, &degraded_topo, &failed_topo] {
        for width in [100, 75, 55, 45] {
            let backend = TestBackend::new(width, 45);
            let mut term = Terminal::new(backend).unwrap();

            let res = term.draw(|f| {
                ClusterView::render(f, f.area(), topo, 0, &theme);
            });
            assert!(res.is_ok(), "Failed to render topology {:?} at width {}", topo.health_status, width);

            let buffer = term.backend().buffer();
            let mut top_x = None;
            let mut bot_x = None;

            for y in 0..buffer.area.height {
                let row_chars: Vec<(u16, String)> = (0..buffer.area.width)
                    .map(|x| (x, buffer[(x, y)].symbol().to_string()))
                    .collect();
                let row_str: String = row_chars.iter().map(|(_, s)| s.clone()).collect();

                if row_str.contains("Shard #1") {
                    if let Some((x, _)) = row_chars.iter().rfind(|(_, s)| s == "╮") {
                        top_x = Some(*x);
                    }
                }
                if row_str.contains(" ╰─") {
                    if let Some((x, _)) = row_chars.iter().rfind(|(_, s)| s == "╯") {
                        bot_x = Some(*x);
                    }
                }
            }

            assert!(top_x.is_some(), "Shard top border not found at width {}", width);
            assert!(bot_x.is_some(), "Shard bottom border not found at width {}", width);
            assert_eq!(top_x.unwrap(), bot_x.unwrap(), "Borders did not align at width {}", width);
        }
    }
}

#[test]
fn test_replication_offset_and_lag_formatting() {
    let topology = create_test_cluster_topology();
    let theme = ThemePalette::dark();

    let backend = TestBackend::new(95, 45);
    let mut term = Terminal::new(backend).unwrap();

    term.draw(|f| {
        ClusterView::render(f, f.area(), &topology, 0, &theme);
    })
    .unwrap();

    let buffer = term.backend().buffer();
    let full_content: String = (0..buffer.area.height)
        .map(|y| {
            let row: String = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            row
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Master 1 repl offset
    assert!(full_content.contains("Repl Offset: 145,210 (Master)"));

    // Replica 1 has matching offset -> In Sync
    assert!(full_content.contains("Repl Offset: 145,210"));
    assert!(full_content.contains("Lag: 0 B (In Sync)"));

    // Shard 2 replica 2 has lag (139,400 - 137,170 = 2230 bytes = 2.2 KB)
    assert!(full_content.contains("Repl Offset: 137,170"));
    assert!(full_content.contains("Lag: 2.2 KB (behind)"));

    // Check slot migration rendering
    assert!(full_content.contains("⇄ MIGRATING Slot 5460 -> @a1b2c3d4 (127.0.0.1:22003)"));
    assert!(full_content.contains("⇆ IMPORTING Slot 5460 <- @8f5851b8 (127.0.0.1:22001)"));
}

#[test]
fn test_summary_header_never_overflows_and_wraps_cleanly() {
    let topology = create_test_cluster_topology();
    let theme = ThemePalette::dark();

    // Specifically test width 82 (user's terminal width from the screenshot)
    let backend = TestBackend::new(82, 40);
    let mut term = Terminal::new(backend).unwrap();

    term.draw(|f| {
        ClusterView::render(f, f.area(), &topology, 0, &theme);
    })
    .unwrap();

    let buffer = term.backend().buffer();
    let full_content: String = (0..buffer.area.height)
        .map(|y| {
            let row: String = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            row
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Must NOT be truncated like "Active M"
    assert!(!full_content.contains("Active M\n"));
    assert!(!full_content.contains("Active M "));

    // Must have full metrics on line 1 and alerts on line 2
    assert!(full_content.contains("[Cluster: DEGRADED]"));
    assert!(full_content.contains("Nodes: 5/6"));
    assert!(full_content.contains("Shards: 2"));
    assert!(full_content.contains("Slots: 16384/16384 (100%)"));
    assert!(full_content.contains("↳ Alerts:"));
    assert!(full_content.contains("Active Migration: 1 slot"));

    // Ensure right borders are intact and no character overwrote the outer border
    for y in 0..buffer.area.height {
        let right_sym = buffer[(81, y)].symbol();
        assert!(
            right_sym == "│" || right_sym == "╮" || right_sym == "╯",
            "Right border at y={} was overwritten by '{}'",
            y,
            right_sym
        );
    }

    // Verify all typical widths: 140, 100, 85, 82, 80, 75, 68, 55, 45, 35, 28
    for width in [140, 100, 85, 82, 80, 75, 68, 55, 45, 35, 28] {
        let backend = TestBackend::new(width, 40);
        let mut term = Terminal::new(backend).unwrap();

        let res = term.draw(|f| {
            ClusterView::render(f, f.area(), &topology, 0, &theme);
        });
        assert!(res.is_ok(), "Failed to render at width {}", width);

        let buf = term.backend().buffer();
        // Top right corner and inner right borders must be preserved
        assert_eq!(buf[(width - 1, 0)].symbol(), "╮");
        assert_eq!(buf[(width - 1, 1)].symbol(), "│");
        assert_eq!(buf[(width - 1, 2)].symbol(), "│");
    }
}

#[test]
fn test_cluster_view_scrollbar_rendering_when_overflowing() {
    let mut topology = create_test_cluster_topology();
    // Add a 3rd shard so content clearly overflows height 25
    let mut shard3 = topology.shards[0].clone();
    shard3.shard_index = 3;
    topology.shards.push(shard3);

    let theme = ThemePalette::dark();
    let backend = TestBackend::new(80, 25);
    let mut term = Terminal::new(backend).unwrap();

    term.draw(|f| {
        ClusterView::render(f, f.area(), &topology, 0, &theme);
    })
    .unwrap();

    let buffer = term.backend().buffer();
    let scrollbar_col = 78; // inner.width = 78, x = 1 + 78 - 1 = 78. Col 79 is outer border '│'

    let col_chars: Vec<String> = (1..24)
        .map(|y| buffer[(scrollbar_col, y)].symbol().to_string())
        .collect();

    // Must have up arrow at top of scrollbar
    assert_eq!(col_chars.first().unwrap(), "↑", "Expected ↑ arrow at top of scrollbar");
    // Must have down arrow at bottom of scrollbar
    assert_eq!(col_chars.last().unwrap(), "↓", "Expected ↓ arrow at bottom of scrollbar");

    // Must contain both track '│' and thumb '█'
    assert!(col_chars.iter().any(|s| s == "█"), "Expected thumb '█' in scrollbar");
    assert!(col_chars.iter().any(|s| s == "│"), "Expected track '│' in scrollbar");

    // Verify low-brightness styling
    let top_cell = &buffer[(scrollbar_col, 1)];
    assert_eq!(top_cell.fg, theme.scrollbar_arrow);

    // Verify outer border at col 79 is preserved
    for y in 0..25 {
        let sym = buffer[(79, y)].symbol();
        assert!(sym == "│" || sym == "╮" || sym == "╯");
    }
}

#[test]
fn test_cluster_view_scrollbar_movement_across_pages() {
    let mut topology = create_test_cluster_topology();
    let mut shard3 = topology.shards[0].clone();
    shard3.shard_index = 3;
    topology.shards.push(shard3);

    let theme = ThemePalette::dark();
    let scrollbar_col = 78;

    // Test offset 0 (Top), offset 1 (Middle), offset 2 (Bottom)
    let mut thumb_positions = Vec::new();

    for offset in [0, 1, 2] {
        let backend = TestBackend::new(80, 26);
        let mut term = Terminal::new(backend).unwrap();

        term.draw(|f| {
            ClusterView::render(f, f.area(), &topology, offset, &theme);
        })
        .unwrap();

        let buffer = term.backend().buffer();
        let thumb_ys: Vec<u16> = (2..24)
            .filter(|y| buffer[(scrollbar_col, *y)].symbol() == "█")
            .collect();

        assert!(!thumb_ys.is_empty(), "Thumb not found for offset {}", offset);
        let avg_y = thumb_ys.iter().sum::<u16>() as f32 / thumb_ys.len() as f32;
        thumb_positions.push(avg_y);
    }

    // Verify thumb moves progressively downwards as scroll_offset increases
    assert!(
        thumb_positions[0] < thumb_positions[1],
        "Thumb should move down from offset 0 to 1 (got {} vs {})",
        thumb_positions[0],
        thumb_positions[1]
    );
    assert!(
        thumb_positions[1] < thumb_positions[2],
        "Thumb should move down from offset 1 to 2 (got {} vs {})",
        thumb_positions[1],
        thumb_positions[2]
    );
}

#[test]
fn test_cluster_view_no_scrollbar_when_content_fits() {
    // Single standalone topology easily fits in height 40
    let topology = ClusterTopology::mock_standalone_topology();
    let theme = ThemePalette::dark();

    let backend = TestBackend::new(80, 40);
    let mut term = Terminal::new(backend).unwrap();

    term.draw(|f| {
        ClusterView::render(f, f.area(), &topology, 0, &theme);
    })
    .unwrap();

    let buffer = term.backend().buffer();
    let scrollbar_col = 78;

    let col_chars: Vec<String> = (1..39)
        .map(|y| buffer[(scrollbar_col, y)].symbol().to_string())
        .collect();

    // No scrollbar arrow should be present
    assert!(!col_chars.contains(&"↑".to_string()), "Should NOT show ↑ when content fits");
    assert!(!col_chars.contains(&"↓".to_string()), "Should NOT show ↓ when content fits");
}

#[test]
fn test_cluster_view_scrollbar_in_light_theme() {
    let mut topology = create_test_cluster_topology();
    let mut shard3 = topology.shards[0].clone();
    shard3.shard_index = 3;
    topology.shards.push(shard3);

    let theme = ThemePalette::light();
    let backend = TestBackend::new(80, 25);
    let mut term = Terminal::new(backend).unwrap();

    term.draw(|f| {
        ClusterView::render(f, f.area(), &topology, 0, &theme);
    })
    .unwrap();

    let buffer = term.backend().buffer();
    let scrollbar_col = 78;

    let top_arrow = &buffer[(scrollbar_col, 1)];
    assert_eq!(top_arrow.symbol(), "↑");
    assert_eq!(top_arrow.fg, theme.scrollbar_arrow);
}



