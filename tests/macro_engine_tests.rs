use xedis_tui::backend::formatter::FormattedValue;
use xedis_tui::core::macro_engine::MacroEngine;

#[test]
fn test_macro_help_table() {
    let help_val = MacroEngine::get_help_value();
    match help_val {
        FormattedValue::Table { headers, rows } => {
            assert!(headers.contains(&"Macro / Command".to_string()) || headers.contains(&"快捷宏 / 命令".to_string()));
            assert!(rows.iter().any(|r| r[0] == "/scan"));
            assert!(rows.iter().any(|r| r[0] == "/bigkeys"));
            assert!(rows.iter().any(|r| r[0] == "/slowlog"));
            assert!(rows.iter().any(|r| r[0] == "/clients"));
        }
        _ => panic!("Expected Table value for help"),
    }
}

#[test]
fn test_macro_bigkeys_table() {
    let bigkeys_val = MacroEngine::get_mock_bigkeys(3);
    match bigkeys_val {
        FormattedValue::Table { headers, rows } => {
            assert_eq!(headers, vec!["Rank", "Key", "Type", "Est. Memory", "Elements"]);
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0][0], "1");
        }
        _ => panic!("Expected Table value for bigkeys"),
    }
}

#[test]
fn test_macro_clients_table() {
    let clients_val = MacroEngine::get_mock_clients();
    match clients_val {
        FormattedValue::Table { headers, rows } => {
            assert!(headers.contains(&"Address".to_string()));
            assert!(!rows.is_empty());
        }
        _ => panic!("Expected Table value for clients"),
    }
}

#[test]
fn test_slowlog_smart_suggestions() {
    let keys_tip = MacroEngine::suggest_for_slow_command("KEYS user:*");
    assert!(keys_tip.is_some());
    assert!(keys_tip.unwrap().contains("/scan"));

    let smembers_tip = MacroEngine::suggest_for_slow_command("SMEMBERS big_set");
    assert!(smembers_tip.is_some());
    assert!(smembers_tip.unwrap().contains("SSCAN"));

    let ping_tip = MacroEngine::suggest_for_slow_command("PING");
    assert!(ping_tip.is_none());
}
