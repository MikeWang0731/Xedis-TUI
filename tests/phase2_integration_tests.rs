use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use xedis_tui::app::App;
use xedis_tui::config::AppConfig;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[tokio::test]
async fn test_app_autocomplete_interaction() {
    let config = AppConfig::default();
    let mut app = App::new(config).await;

    // Type '/'
    app.handle_key(key(KeyCode::Char('/'))).await;
    assert_eq!(app.input_buffer, "/");
    assert!(app.autocomplete_active);
    assert!(!app.autocomplete_items.is_empty());
    assert_eq!(app.autocomplete_items[0].completion_text, "/scan ");

    // Press Tab to accept completion
    app.handle_key(key(KeyCode::Tab)).await;
    assert_eq!(app.input_buffer, "/scan ");
    assert!(!app.autocomplete_active);

    // Type 'user:* 10'
    for c in "user:* 10".chars() {
        app.handle_key(key(KeyCode::Char(c))).await;
    }
    assert_eq!(app.input_buffer, "/scan user:* 10");

    // Submit command
    let init_record_count = app.records.len();
    app.handle_key(key(KeyCode::Enter)).await;
    assert_eq!(app.records.len(), init_record_count + 1);
    let last = app.records.last().unwrap();
    assert_eq!(last.command, "/scan user:* 10");
    assert!(matches!(last.result, xedis_tui::backend::formatter::FormattedValue::Json(_)));
}

#[tokio::test]
async fn test_app_node_routing_and_macro_execution() {
    let config = AppConfig::default();
    let mut app = App::new(config).await;

    // Type '@'
    app.handle_key(key(KeyCode::Char('@'))).await;
    assert!(app.autocomplete_active);
    assert_eq!(app.autocomplete_items[0].completion_text, "@node-1 ");

    // Press Enter to accept autocomplete
    app.handle_key(key(KeyCode::Enter)).await;
    assert_eq!(app.input_buffer, "@node-1 ");

    // Add command 'HGETALL user:profile'
    for c in "HGETALL user:profile".chars() {
        app.handle_key(key(KeyCode::Char(c))).await;
    }

    let init_record_count = app.records.len();
    app.handle_key(key(KeyCode::Enter)).await;
    assert_eq!(app.records.len(), init_record_count + 1);

    let last = app.records.last().unwrap();
    assert_eq!(last.target_node, Some("node-1".to_string()));
    assert!(matches!(last.result, xedis_tui::backend::formatter::FormattedValue::Table { .. }));
}

#[tokio::test]
async fn test_app_scrolling_and_history() {
    let config = AppConfig::default();
    let mut app = App::new(config).await;

    // Test PageUp & PageDown scrolling
    assert_eq!(app.scroll_offset, 0);
    app.handle_key(key(KeyCode::PageUp)).await;
    assert_eq!(app.scroll_offset, 6);
    app.handle_key(key(KeyCode::PageDown)).await;
    assert_eq!(app.scroll_offset, 0);

    // Test Up arrow history after submitting
    app.input_buffer = "PING".to_string();
    app.cursor_pos = 4;
    app.submit_command().await;

    app.handle_key(key(KeyCode::Up)).await;
    assert_eq!(app.input_buffer, "PING");
}
