use xedis_tui::app::App;
use xedis_tui::backend::client::XedisClient;
use xedis_tui::backend::formatter::FormattedValue;
use xedis_tui::config::AppConfig;
use xedis_tui::core::router::CommandRouter;

#[tokio::test]
async fn test_broadcast_all_cluster_info() {
    let mut client = XedisClient::connect("redis://127.0.0.1:6379", true).await;
    let (val, duration) = client.execute_command(Some("all"), "CLUSTER", &["info".to_string()]).await;

    assert!(duration.as_millis() < 500);
    match val {
        FormattedValue::String(s) => {
            assert!(s.contains("--- Node: @node-1"));
            assert!(s.contains("--- Node: @node-2"));
            assert!(s.contains("--- Node: @node-3"));
            assert!(s.contains("cluster_state: ok"));
            assert!(s.contains("cluster_slots_assigned: 16384"));
        }
        other => panic!("Expected FormattedValue::String with node sections, got {:?}", other),
    }
}

#[tokio::test]
async fn test_broadcast_all_ping() {
    let mut client = XedisClient::connect("redis://127.0.0.1:6379", true).await;
    let (val, _) = client.execute_command(Some("all"), "PING", &[]).await;

    match val {
        FormattedValue::String(s) => {
            assert!(s.contains("--- Node: @node-1"));
            assert!(s.contains("PONG"));
            assert!(s.contains("--- Node: @node-2"));
            assert!(s.contains("--- Node: @node-3"));
        }
        other => panic!("Expected FormattedValue::String, got {:?}", other),
    }
}

#[tokio::test]
async fn test_broadcast_scan_macro() {
    let mut client = XedisClient::connect("redis://127.0.0.1:6379", true).await;
    let (val, _) = client.execute_macro(Some("all"), "/scan", &["order:*".to_string(), "10".to_string()]).await;

    match val {
        FormattedValue::String(s) => {
            assert!(s.contains("--- Node: @node-1"));
            assert!(s.contains("order:node-1"));
            assert!(s.contains("--- Node: @node-2"));
            assert!(s.contains("order:node-2"));
        }
        other => panic!("Expected FormattedValue::String for broadcast scan, got {:?}", other),
    }
}

#[tokio::test]
async fn test_broadcast_all_masters_filter() {
    let mut client = XedisClient::connect("redis://127.0.0.1:6379", true).await;
    let (val, _) = client.execute_command(Some("all-masters"), "PING", &[]).await;

    match val {
        FormattedValue::String(s) => {
            assert!(s.contains("--- Node: @node-1"));
            assert!(s.contains("Master"));
            // Shouldn't contain replicas
            assert!(!s.contains("Replica"));
        }
        other => panic!("Expected FormattedValue::String, got {:?}", other),
    }
}

#[tokio::test]
async fn test_single_node_direct_table_response() {
    let mut client = XedisClient::connect("redis://127.0.0.1:6379", true).await;
    let (val, _) = client.execute_command(Some("node-1"), "HGETALL", &["user:profile".to_string()]).await;

    match val {
        FormattedValue::Table { headers, rows } => {
            assert_eq!(headers, vec!["Field", "Value"]);
            assert!(!rows.is_empty());
            assert!(rows.iter().any(|r| r[0] == "user_id"));
        }
        other => panic!("Expected FormattedValue::Table for single node query, got {:?}", other),
    }
}

#[tokio::test]
async fn test_app_end_to_end_broadcast_workflow() {
    let config = AppConfig::default();
    let mut app = App::new(config).await;

    let parsed = CommandRouter::parse("@all CLUSTER info").expect("Should parse @all command");
    assert_eq!(parsed.target_node, Some("all".to_string()));

    app.execute_parsed_command(parsed).await;

    let last = app.records.last().expect("Record should exist");
    assert_eq!(last.target_node, Some("all".to_string()));
    assert_eq!(last.command, "@all CLUSTER info");

    match &last.result {
        FormattedValue::String(s) => {
            assert!(s.contains("--- Node: @node-1"));
            assert!(s.contains("--- Node: @node-2"));
            assert!(s.contains("--- Node: @node-3"));
            assert!(s.contains("cluster_state: ok"));
        }
        other => panic!("Expected multi-node String output, got {:?}", other),
    }
}
