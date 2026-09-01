use std::time::Duration;
use xedis_tui::backend::formatter::FormattedValue;
use xedis_tui::ui::stream_view::{ExecutionRecord, StreamView};

#[test]
fn test_line_count_calculation_various_types() {
    let mut records = Vec::new();

    // 1. Status
    records.push(ExecutionRecord {
        target_node: None,
        command: "PING".to_string(),
        timestamp: "12:00:00".to_string(),
        duration: Duration::from_micros(200),
        result: FormattedValue::Status("PONG".to_string()),
    });
    assert_eq!(StreamView::total_lines_count(&records, 80), 2); // Header + Status line

    // 2. Table
    records.push(ExecutionRecord {
        target_node: Some("node-1".to_string()),
        command: "HGETALL user".to_string(),
        timestamp: "12:00:01".to_string(),
        duration: Duration::from_millis(1),
        result: FormattedValue::Table {
            headers: vec!["Field".to_string(), "Value".to_string()],
            rows: vec![
                vec!["name".to_string(), "Alice".to_string()],
                vec!["role".to_string(), "admin".to_string()],
            ],
        },
    });
    // Record 1: 2 lines
    // Separator: 1 line
    // Record 2: header (1) + table (2 rows + 4 border lines = 6) = 7 lines
    // Total = 2 + 1 + 7 = 10 lines
    assert_eq!(StreamView::total_lines_count(&records, 80), 10);
}

#[test]
fn test_virtual_slice_only_renders_requested_window() {
    let mut records = Vec::new();
    // Simulate 1,000 records
    for i in 0..1000 {
        records.push(ExecutionRecord {
            target_node: Some(format!("node-{}", i % 6)),
            command: format!("GET key:{}", i),
            timestamp: "12:00:00".to_string(),
            duration: Duration::from_micros(150),
            result: FormattedValue::String(format!("value_{}", i)),
        });
    }

    let total = StreamView::total_lines_count(&records, 80);

    assert!(total >= 2999); // 1000 records * ~3 lines each

    // Test virtual slice for a small viewport (e.g. 20 lines from line 500 to 520)
    let start_line = 500;
    let end_line = 520;
    let sliced_lines = StreamView::render_virtual_lines(&records, start_line, end_line, 80, false);

    assert_eq!(sliced_lines.len(), 20, "Should only allocate exactly 20 lines");

    // Test start of stream
    let top_lines = StreamView::render_virtual_lines(&records, 0, 15, 80, false);
    assert_eq!(top_lines.len(), 15);

    // Test end of stream
    let bot_lines = StreamView::render_virtual_lines(&records, total.saturating_sub(10), total, 80, false);
    assert_eq!(bot_lines.len(), 10);
}

#[test]
fn test_stream_memory_trimming() {
    // Verify that when records exceed max_stream_records, draining oldest maintains the limit
    let max_records = 50;
    let mut records: Vec<ExecutionRecord> = Vec::new();

    for i in 0..120 {
        records.push(ExecutionRecord {
            target_node: None,
            command: format!("SET k{} v{}", i, i),
            timestamp: "12:00:00".to_string(),
            duration: Duration::from_micros(100),
            result: FormattedValue::Status("OK".to_string()),
        });

        if records.len() > max_records {
            let overflow = records.len() - max_records;
            records.drain(0..overflow);
        }
    }

    assert_eq!(records.len(), 50);
    assert_eq!(records.first().unwrap().command, "SET k70 v70");
    assert_eq!(records.last().unwrap().command, "SET k119 v119");
}
