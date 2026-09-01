use xedis_tui::core::router::{CommandRouter, CommandType};

#[test]
fn test_tokenize_quotes() {
    let input = r#"HSET user:1001 name "Alice Wonder" note 'developer and tester' count 42"#;
    let tokens = CommandRouter::tokenize(input);
    assert_eq!(
        tokens,
        vec![
            "HSET",
            "user:1001",
            "name",
            "Alice Wonder",
            "note",
            "developer and tester",
            "count",
            "42"
        ]
    );
}

#[test]
fn test_parse_node_prefix() {
    let input = "@node-1 HGETALL user:session:9801";
    let parsed = CommandRouter::parse(input).expect("Should parse");
    assert_eq!(parsed.target_node, Some("node-1".to_string()));
    match parsed.command_type {
        CommandType::Native { cmd, args } => {
            assert_eq!(cmd, "HGETALL");
            assert_eq!(args, vec!["user:session:9801"]);
        }
        _ => panic!("Expected Native command"),
    }
}

#[test]
fn test_parse_macro_command() {
    let input = "/scan order:* 50";
    let parsed = CommandRouter::parse(input).expect("Should parse");
    assert_eq!(parsed.target_node, None);
    match parsed.command_type {
        CommandType::Macro { name, args } => {
            assert_eq!(name, "/scan");
            assert_eq!(args, vec!["order:*", "50"]);
        }
        _ => panic!("Expected Macro command"),
    }
}

#[test]
fn test_parse_node_and_macro_combined() {
    let input = "@node-2 /bigkeys 10";
    let parsed = CommandRouter::parse(input).expect("Should parse");
    assert_eq!(parsed.target_node, Some("node-2".to_string()));
    match parsed.command_type {
        CommandType::Macro { name, args } => {
            assert_eq!(name, "/bigkeys");
            assert_eq!(args, vec!["10"]);
        }
        _ => panic!("Expected Macro command"),
    }
}

#[test]
fn test_parse_empty() {
    assert!(CommandRouter::parse("").is_none());
    assert!(CommandRouter::parse("   ").is_none());
}
