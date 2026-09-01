use xedis_tui::core::guard::{RiskLevel, SafetyGuard};
use xedis_tui::core::router::CommandRouter;

#[test]
fn test_level3_flushall_flushdb_shutdown() {
    // FLUSHALL
    let p1 = CommandRouter::parse("FLUSHALL").unwrap();
    let assess1 = SafetyGuard::inspect(&p1).expect("FLUSHALL should be intercepted");
    assert_eq!(assess1.level, RiskLevel::Level3Blocking);
    assert!(assess1.title.contains("FLUSHALL"));
    assert!(assess1.suggestion.contains("FLUSHALL ASYNC"));

    // flushdb with node
    let p2 = CommandRouter::parse("@node-1 flushdb").unwrap();
    let assess2 = SafetyGuard::inspect(&p2).expect("flushdb should be intercepted");
    assert_eq!(assess2.level, RiskLevel::Level3Blocking);

    // SHUTDOWN
    let p3 = CommandRouter::parse("SHUTDOWN").unwrap();
    let assess3 = SafetyGuard::inspect(&p3).expect("SHUTDOWN should be intercepted");
    assert_eq!(assess3.level, RiskLevel::Level3Blocking);
    assert!(assess3.reason.contains("终止"));
}

#[test]
fn test_level3_keys_wildcard() {
    let p1 = CommandRouter::parse("KEYS *").unwrap();
    let assess1 = SafetyGuard::inspect(&p1).expect("KEYS * should be intercepted");
    assert_eq!(assess1.level, RiskLevel::Level3Blocking);
    assert!(assess1.suggestion.contains("/scan"));

    let p2 = CommandRouter::parse("keys user:*").unwrap();
    let assess2 = SafetyGuard::inspect(&p2).expect("keys user:* should be intercepted");
    assert_eq!(assess2.level, RiskLevel::Level3Blocking);
    assert!(assess2.reason.contains("O(N)"));
}

#[test]
fn test_level3_config_rewrite_and_debug() {
    let p1 = CommandRouter::parse("CONFIG REWRITE").unwrap();
    let assess1 = SafetyGuard::inspect(&p1).expect("CONFIG REWRITE should be intercepted");
    assert_eq!(assess1.level, RiskLevel::Level3Blocking);

    let p2 = CommandRouter::parse("DEBUG SEGFAULT").unwrap();
    let assess2 = SafetyGuard::inspect(&p2).expect("DEBUG SEGFAULT should be intercepted");
    assert_eq!(assess2.level, RiskLevel::Level3Blocking);

    let p3 = CommandRouter::parse("DEBUG SLEEP 10").unwrap();
    let assess3 = SafetyGuard::inspect(&p3).expect("DEBUG SLEEP should be intercepted");
    assert_eq!(assess3.level, RiskLevel::Level3Blocking);
}

#[test]
fn test_level2_warning_commands() {
    // CONFIG SET requirepass
    let p1 = CommandRouter::parse("CONFIG SET requirepass \"secret123\"").unwrap();
    let assess1 = SafetyGuard::inspect(&p1).expect("CONFIG SET requirepass should be warned");
    assert_eq!(assess1.level, RiskLevel::Level2Warning);

    // BGSAVE
    let p2 = CommandRouter::parse("BGSAVE").unwrap();
    let assess2 = SafetyGuard::inspect(&p2).expect("BGSAVE should be warned");
    assert_eq!(assess2.level, RiskLevel::Level2Warning);

    // SLAVEOF NO ONE / REPLICAOF NO ONE
    let p3 = CommandRouter::parse("REPLICAOF NO ONE").unwrap();
    let assess3 = SafetyGuard::inspect(&p3).expect("REPLICAOF NO ONE should be warned");
    assert_eq!(assess3.level, RiskLevel::Level2Warning);

    // SWAPDB
    let p4 = CommandRouter::parse("SWAPDB 0 1").unwrap();
    let assess4 = SafetyGuard::inspect(&p4).expect("SWAPDB should be warned");
    assert_eq!(assess4.level, RiskLevel::Level2Warning);

    // MIGRATE
    let p5 = CommandRouter::parse("MIGRATE 127.0.0.1 6380 mykey 0 5000").unwrap();
    let assess5 = SafetyGuard::inspect(&p5).expect("MIGRATE should be warned");
    assert_eq!(assess5.level, RiskLevel::Level2Warning);

    // SMEMBERS
    let p6 = CommandRouter::parse("SMEMBERS big_set").unwrap();
    let assess6 = SafetyGuard::inspect(&p6).expect("SMEMBERS should be warned");
    assert_eq!(assess6.level, RiskLevel::Level2Warning);
    assert!(assess6.suggestion.contains("SSCAN"));
}

#[test]
fn test_safe_commands_pass_freely() {
    let safe_list = vec![
        "GET user:1001",
        "SET key value EX 3600",
        "HGET user:1001 name",
        "HSET user:1001 age 25",
        "LRANGE queue:jobs 0 10",
        "PING",
        "INFO",
        "INFO memory",
        "CLUSTER NODES",
        "CONFIG GET maxmemory",
        "/scan user:* 20",
        "/bigkeys 5",
        "/slowlog 10",
        "/interval 1s",
        "/settings",
        "/clients",
        "/help",
        "/clear",
    ];

    for raw in safe_list {
        let parsed = CommandRouter::parse(raw).expect(&format!("Should parse '{}'", raw));
        let assess = SafetyGuard::inspect(&parsed);
        assert!(assess.is_none(), "Safe command '{}' should not trigger guard", raw);
    }
}
