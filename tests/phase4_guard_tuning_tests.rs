use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use xedis_tui::app::App;
use xedis_tui::config::AppConfig;
use xedis_tui::core::guard::RiskLevel;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[tokio::test]
async fn test_safety_guard_interception_and_cancel() {
    let mut config = AppConfig::default();
    config.safety_guard_enabled = true;
    let mut app = App::new(config).await;

    // 1. Enter FLUSHALL
    app.input_buffer = "FLUSHALL".to_string();
    app.cursor_pos = app.input_buffer.len();
    app.submit_command().await;

    // Guard should intercept
    assert!(app.pending_guard.is_some(), "FLUSHALL should trigger pending guard modal");
    let (_, assess) = app.pending_guard.as_ref().unwrap();
    assert_eq!(assess.level, RiskLevel::Level3Blocking);
    assert_eq!(app.records.len(), 0, "Command should not execute immediately");

    // 2. User presses 'n' to cancel
    app.handle_key(make_key(KeyCode::Char('n'))).await;
    assert!(app.pending_guard.is_none(), "Pending guard should be dismissed");
    assert_eq!(app.records.len(), 0, "Command was canceled, records should remain empty");

    // 3. Enter KEYS * and cancel via Esc
    app.input_buffer = "KEYS *".to_string();
    app.submit_command().await;
    assert!(app.pending_guard.is_some());

    app.handle_key(make_key(KeyCode::Esc)).await;
    assert!(app.pending_guard.is_none());
    assert_eq!(app.records.len(), 0);
}

#[tokio::test]
async fn test_safety_guard_confirm_and_execute() {
    let mut config = AppConfig::default();
    config.safety_guard_enabled = true;
    let mut app = App::new(config).await;

    // 1. Enter KEYS *
    app.input_buffer = "KEYS *".to_string();
    app.submit_command().await;
    assert!(app.pending_guard.is_some());

    // 2. User presses Enter or 'y' to confirm execution
    app.handle_key(make_key(KeyCode::Enter)).await;
    assert!(app.pending_guard.is_none(), "Guard should clear on confirmation");
    assert_eq!(app.records.len(), 1, "Command should be executed and recorded");
    assert_eq!(app.records[0].command, "KEYS *");
}

#[tokio::test]
async fn test_help_modal_workflow() {
    let config = AppConfig::default();
    let mut app = App::new(config).await;

    assert!(!app.help_active);

    // 1. Open help via F1
    app.handle_key(make_key(KeyCode::F(1))).await;
    assert!(app.help_active);
    assert_eq!(app.help_tab, 0);

    // 2. Switch tab via Tab
    app.handle_key(make_key(KeyCode::Tab)).await;
    assert_eq!(app.help_tab, 1);

    // 3. Switch tab via number key '3' (Troubleshoot)
    app.handle_key(make_key(KeyCode::Char('3'))).await;
    assert_eq!(app.help_tab, 2);

    // 4. Scroll down in help
    app.handle_key(make_key(KeyCode::Down)).await;
    assert_eq!(app.help_scroll_offset, 1);

    app.handle_key(make_key(KeyCode::PageDown)).await;
    assert_eq!(app.help_scroll_offset, 6);

    // 5. Close help with Esc
    app.handle_key(make_key(KeyCode::Esc)).await;
    assert!(!app.help_active);

    // 6. Trigger via /help macro in prompt
    app.input_buffer = "/help".to_string();
    app.submit_command().await;
    assert!(app.help_active);

    // Close via 'q'
    app.handle_key(make_key(KeyCode::Char('q'))).await;
    assert!(!app.help_active);
}

#[tokio::test]
async fn test_history_trimming_under_high_load() {
    let mut config = AppConfig::default();
    config.max_stream_records = 20; // Set small limit for testing
    let mut app = App::new(config).await;

    for _ in 0..50 {
        app.input_buffer = format!("PING");

        app.submit_command().await;
    }

    assert_eq!(app.records.len(), 20, "Records must be capped to max_stream_records (20)");
}
