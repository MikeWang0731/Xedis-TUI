use xedis_tui::backend::client::ClusterNodeInfo;
use xedis_tui::core::autocomplete::{AutocompleteEngine, SuggestionKind};

fn mock_nodes() -> Vec<ClusterNodeInfo> {
    vec![
        ClusterNodeInfo {
            id: "node-1".to_string(),
            raw_id: "node1_full_id_12345678".to_string(),
            address: "127.0.0.1:6379".to_string(),
            cport: 16379,
            role: "Master".to_string(),
            master_id: None,
            slots_raw: "0-5460".to_string(),
            slot_ranges: vec![(0, 5460)],
            slot_count: 5461,
            key_count: 1000,
            is_healthy: true,
            ping_ms: 0.38,
        },
        ClusterNodeInfo {
            id: "node-2".to_string(),
            raw_id: "node2_full_id_12345678".to_string(),
            address: "127.0.0.1:6380".to_string(),
            cport: 16380,
            role: "Master".to_string(),
            master_id: None,
            slots_raw: "5461-10922".to_string(),
            slot_ranges: vec![(5461, 10922)],
            slot_count: 5462,
            key_count: 1000,
            is_healthy: true,
            ping_ms: 0.41,
        },
    ]
}

#[test]
fn test_macro_autocomplete() {
    let nodes = mock_nodes();
    let (suggestions, range) = AutocompleteEngine::get_suggestions("/s", 2, &nodes);
    assert_eq!(range, (0, 2));
    assert!(!suggestions.is_empty());
    for item in &suggestions {
        assert_eq!(item.kind, SuggestionKind::Macro);
    }
    let names: Vec<&str> = suggestions.iter().map(|s| s.completion_text.trim()).collect();
    assert!(names.contains(&"/scan"));
    assert!(names.contains(&"/slowlog"));
}

#[test]
fn test_node_autocomplete() {
    let nodes = mock_nodes();
    let (suggestions, range) = AutocompleteEngine::get_suggestions("@", 1, &nodes);
    assert_eq!(range, (0, 1));
    assert_eq!(suggestions.len(), 3); // node-1, node-2, @all
    assert_eq!(suggestions[0].kind, SuggestionKind::Node);
    assert_eq!(suggestions[0].completion_text, "@node-1 ");
    assert_eq!(suggestions[1].completion_text, "@node-2 ");
    assert_eq!(suggestions[2].completion_text, "@all ");
}

#[test]
fn test_command_prefix_autocomplete() {
    let nodes = mock_nodes();
    let (suggestions, range) = AutocompleteEngine::get_suggestions("HGET", 4, &nodes);
    assert_eq!(range, (0, 4));
    assert!(!suggestions.is_empty());
    let names: Vec<&str> = suggestions.iter().map(|s| s.completion_text.trim()).collect();
    assert!(names.contains(&"HGET"));
    assert!(names.contains(&"HGETALL"));
}

#[test]
fn test_info_persistence_subcommand_autocomplete() {
    let nodes = mock_nodes();

    // 1. Typing "INFO PERSIST"
    let input = "INFO PERSIST";
    let (suggestions, range) = AutocompleteEngine::get_suggestions(input, input.len(), &nodes);
    assert_eq!(range, (5, 12));
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].kind, SuggestionKind::Subcommand);
    assert_eq!(suggestions[0].completion_text, "persistence ");
    assert_eq!(suggestions[0].display_title, "INFO PERSISTENCE");
    assert!(suggestions[0].description.contains("持久化"));

    // 2. Typing "INFO " (lists all sections)
    let input2 = "INFO ";
    let (suggestions2, range2) = AutocompleteEngine::get_suggestions(input2, input2.len(), &nodes);
    assert_eq!(range2, (5, 5));
    assert!(suggestions2.len() >= 10);
    let sec_names: Vec<&str> = suggestions2.iter().map(|s| s.completion_text.trim()).collect();
    assert!(sec_names.contains(&"persistence"));
    assert!(sec_names.contains(&"memory"));
    assert!(sec_names.contains(&"server"));
    assert!(sec_names.contains(&"clients"));

    // 3. Routed command: "@node-1 INFO MEM"
    let input3 = "@node-1 INFO MEM";
    let (suggestions3, range3) = AutocompleteEngine::get_suggestions(input3, input3.len(), &nodes);
    assert_eq!(range3, (13, 16));
    assert_eq!(suggestions3.len(), 1);
    assert_eq!(suggestions3[0].completion_text, "memory ");

    // 4. "CLUSTER NOD"
    let input4 = "CLUSTER NOD";
    let (suggestions4, range4) = AutocompleteEngine::get_suggestions(input4, input4.len(), &nodes);
    assert_eq!(range4, (8, 11));
    assert_eq!(suggestions4.len(), 1);
    assert_eq!(suggestions4[0].completion_text, "nodes ");
}

#[test]
fn test_dangerous_and_extended_command_autocomplete() {
    let nodes = mock_nodes();

    // 1. "KEY" -> KEYS
    let (s_keys, _) = AutocompleteEngine::get_suggestions("KEY", 3, &nodes);
    let names: Vec<&str> = s_keys.iter().map(|s| s.completion_text.trim()).collect();
    assert!(names.contains(&"KEYS"));

    // 2. "FLUSH" -> FLUSHALL, FLUSHDB
    let (s_flush, _) = AutocompleteEngine::get_suggestions("FLUSH", 5, &nodes);
    let f_names: Vec<&str> = s_flush.iter().map(|s| s.completion_text.trim()).collect();
    assert!(f_names.contains(&"FLUSHALL"));
    assert!(f_names.contains(&"FLUSHDB"));

    // 3. "SHUT" -> SHUTDOWN
    let (s_shut, _) = AutocompleteEngine::get_suggestions("SHUT", 4, &nodes);
    let shut_names: Vec<&str> = s_shut.iter().map(|s| s.completion_text.trim()).collect();
    assert!(shut_names.contains(&"SHUTDOWN"));

    // 4. "FLUSHALL " -> ASYNC, SYNC subcommands
    let (s_flush_sub, _) = AutocompleteEngine::get_suggestions("FLUSHALL ", 9, &nodes);
    let sub_names: Vec<&str> = s_flush_sub.iter().map(|s| s.completion_text.trim()).collect();
    assert!(sub_names.contains(&"async"));
    assert!(sub_names.contains(&"sync"));
}

