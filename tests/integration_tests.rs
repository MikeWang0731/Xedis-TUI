use xedis_tui::backend::formatter::FormattedValue;
use xedis_tui::config::LayoutPreset;
use redis::Value;

#[test]
fn test_layout_presets_cycle() {
    let preset = LayoutPreset::Balanced;
    assert_eq!(preset.split_ratio(), 58);

    let next1 = preset.next();
    assert_eq!(next1, LayoutPreset::Focus);
    assert_eq!(next1.split_ratio(), 75);

    let next2 = next1.next();
    assert_eq!(next2, LayoutPreset::Monitor);
    assert_eq!(next2.split_ratio(), 35);

    let next3 = next2.next();
    assert_eq!(next3, LayoutPreset::Zen);
    assert_eq!(next3.split_ratio(), 100);

    let next4 = next3.next();
    assert_eq!(next4, LayoutPreset::Balanced);
}

#[test]
fn test_formatter_simple_and_int() {
    let status_val = Value::SimpleString("PONG".to_string());
    let formatted = FormattedValue::from_redis_value(status_val);
    match formatted {
        FormattedValue::Status(s) => assert_eq!(s, "PONG"),
        _ => panic!("Expected Status"),
    }

    let int_val = Value::Int(42);
    let formatted_int = FormattedValue::from_redis_value(int_val);
    match formatted_int {
        FormattedValue::Integer(i) => assert_eq!(i, 42),
        _ => panic!("Expected Integer"),
    }
}

#[test]
fn test_formatter_table_pairs() {
    let hash_array = Value::Array(vec![
        Value::BulkString(b"user_id".to_vec()),
        Value::BulkString(b"1001".to_vec()),
        Value::BulkString(b"role".to_vec()),
        Value::BulkString(b"admin".to_vec()),
    ]);
    let formatted = FormattedValue::from_redis_value(hash_array);
    match formatted {
        FormattedValue::Table { headers, rows } => {
            assert_eq!(headers, vec!["Field", "Value"]);
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0], vec!["user_id", "1001"]);
            assert_eq!(rows[1], vec!["role", "admin"]);
        }
        _ => panic!("Expected Table"),
    }
}

#[test]
fn test_formatter_json_detection() {
    let json_bytes = b"{\"name\":\"alice\",\"age\":30}".to_vec();
    let json_val = Value::BulkString(json_bytes);
    let formatted = FormattedValue::from_redis_value(json_val);
    match formatted {
        FormattedValue::Json(pretty) => {
            assert!(pretty.contains("\"name\": \"alice\""));
        }
        _ => panic!("Expected JSON formatting"),
    }
}

#[test]
fn test_formatter_cluster_map_response() {
    let mut map = Vec::new();
    map.push((
        Value::BulkString(b"127.0.0.1:22000".to_vec()),
        Value::BulkString(b"# Persistence\r\nloading:0\r\nrdb_changes:0\r\n".to_vec()),
    ));
    map.push((
        Value::BulkString(b"127.0.0.1:22001".to_vec()),
        Value::BulkString(b"# Persistence\r\nloading:0\r\nrdb_changes:0\r\n".to_vec()),
    ));

    let map_val = Value::Map(map);
    let formatted = FormattedValue::from_redis_value(map_val);

    match formatted {
        FormattedValue::String(s) => {
            assert!(!s.contains('\r'), "Must not contain carriage return \\r");
            assert!(s.contains("--- Node: @127.0.0.1:22000 ---"));
            assert!(s.contains("# Persistence"));
            assert!(s.contains("loading:0"));
        }
        _ => panic!("Expected String formatting for cluster multi-node response"),
    }
}
