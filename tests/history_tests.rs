use xedis_tui::core::history::HistoryManager;

#[test]
fn test_history_push_and_dedup() {
    let mut history = HistoryManager::new_in_memory(10);
    history.push("GET key1");
    history.push("GET key1"); // Consecutive duplicate should be ignored
    history.push("SET key2 val");

    assert_eq!(history.len(), 2);
}

#[test]
fn test_history_navigation() {
    let mut history = HistoryManager::new_in_memory(10);
    history.push("PING");
    history.push("INFO");
    history.push("DBSIZE");

    // Start navigating from a draft
    let draft = "custom input";
    let p1 = history.navigate_prev(draft);
    assert_eq!(p1, Some("DBSIZE".to_string()));

    let p2 = history.navigate_prev("ignored");
    assert_eq!(p2, Some("INFO".to_string()));

    let p3 = history.navigate_prev("ignored");
    assert_eq!(p3, Some("PING".to_string()));

    // At top of history
    let p4 = history.navigate_prev("ignored");
    assert_eq!(p4, Some("PING".to_string()));

    // Navigate back down
    let n1 = history.navigate_next();
    assert_eq!(n1, Some("INFO".to_string()));

    let n2 = history.navigate_next();
    assert_eq!(n2, Some("DBSIZE".to_string()));

    // Restores initial draft!
    let n3 = history.navigate_next();
    assert_eq!(n3, Some("custom input".to_string()));
}
