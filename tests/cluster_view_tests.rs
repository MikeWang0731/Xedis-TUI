use ratatui::backend::TestBackend;
use ratatui::Terminal;
use xedis_tui::backend::cluster_info::{ClusterNode, ClusterShard, ClusterTopology, RedisTopologyMode};
use xedis_tui::ui::cluster_view::ClusterView;
use xedis_tui::ui::theme::ThemePalette;

fn create_test_cluster_topology() -> ClusterTopology {
    ClusterTopology {
        mode: RedisTopologyMode::Cluster,
        is_cluster: true,
        total_nodes: 6,
        healthy_nodes: 6,
        covered_slots: 16384,
        is_fully_covered: true,
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
                    ping_ms: 3.0,
                    slots_raw: "0-5460".to_string(),
                    slot_ranges: vec![(0, 5460)],
                    slot_count: 5461,
                    key_count: 100,
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
                        ping_ms: 3.0,
                        slots_raw: "".to_string(),
                        slot_ranges: vec![],
                        slot_count: 0,
                        key_count: 0,
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
                    ping_ms: 2.5,
                    slots_raw: "5461-10922".to_string(),
                    slot_ranges: vec![(5461, 10922)],
                    slot_count: 5462,
                    key_count: 120,
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
                        ping_ms: 2.8,
                        slots_raw: "".to_string(),
                        slot_ranges: vec![],
                        slot_count: 0,
                        key_count: 0,
                    },
                    ClusterNode {
                        id: "rep2b".to_string(),
                        raw_id: "rep2b1234567890abcdef".to_string(),
                        address: "127.0.0.1:22005".to_string(),
                        cport: 32005,
                        role: "Replica".to_string(),
                        master_id: Some("a1b2c3d4".to_string()),
                        is_healthy: false,
                        ping_ms: 120.0,
                        slots_raw: "".to_string(),
                        slot_ranges: vec![],
                        slot_count: 0,
                        key_count: 0,
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
