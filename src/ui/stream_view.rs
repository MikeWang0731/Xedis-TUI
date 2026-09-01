use crate::backend::formatter::FormattedValue;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub target_node: Option<String>,
    pub command: String,
    pub timestamp: String,
    pub duration: Duration,
    pub result: FormattedValue,
}

pub struct StreamView;

impl StreamView {
    pub fn render(
        f: &mut Frame,
        area: Rect,
        records: &[ExecutionRecord],
        input: &str,
        cursor_pos: usize,
        scroll_offset: usize,
        is_focused: bool,
        is_dimmed: bool,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        let stream_area = chunks[0];
        let input_area = chunks[1];

        // 1. Stream Card Block
        let (border_style, title_style) = if is_dimmed {
            (
                Style::default().fg(Color::Rgb(40, 50, 60)),
                Style::default().fg(Color::Rgb(80, 95, 110)),
            )
        } else if is_focused {
            (
                Style::default().fg(Color::Cyan),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )
        } else {
            (
                Style::default().fg(Color::DarkGray),
                Style::default().fg(Color::DarkGray),
            )
        };

        let stream_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(Span::styled(" Command Stream ", title_style));

        let inner_stream = stream_block.inner(stream_area);
        f.render_widget(stream_block, stream_area);

        // Collect all lines for smooth scrolling
        let mut all_lines: Vec<Line> = Vec::new();

        // Welcome banner if records are empty
        if records.is_empty() {
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(vec![
                Span::styled(" [XEDIS] ", Style::default().bg(Color::Rgb(15, 45, 60)).fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" Welcome to Xedis-TUI", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(" · Terminal Workbench for Redis & Custom Middleware", Style::default().fg(Color::Rgb(165, 180, 195))),
            ]));
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(vec![
                Span::styled(" Quick Start & Tips:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            all_lines.push(Line::from(vec![
                Span::styled("  • Command:     ", Style::default().fg(Color::Rgb(165, 180, 195))),
                Span::styled("Type any Redis command (e.g. ", Style::default().fg(Color::DarkGray)),
                Span::styled("PING", Style::default().fg(Color::Cyan)),
                Span::styled(", ", Style::default().fg(Color::DarkGray)),
                Span::styled("INFO", Style::default().fg(Color::Cyan)),
                Span::styled(", ", Style::default().fg(Color::DarkGray)),
                Span::styled("GET <key>", Style::default().fg(Color::Cyan)),
                Span::styled(", ", Style::default().fg(Color::DarkGray)),
                Span::styled("SET <k> <v>", Style::default().fg(Color::Cyan)),
                Span::styled(") and press Enter", Style::default().fg(Color::DarkGray)),
            ]));
            all_lines.push(Line::from(vec![
                Span::styled("  • Macros:      ", Style::default().fg(Color::Rgb(165, 180, 195))),
                Span::styled("Type ", Style::default().fg(Color::DarkGray)),
                Span::styled("/scan", Style::default().fg(Color::Yellow)),
                Span::styled(", ", Style::default().fg(Color::DarkGray)),
                Span::styled("/bigkeys", Style::default().fg(Color::Green)),
                Span::styled(", ", Style::default().fg(Color::DarkGray)),
                Span::styled("/interval", Style::default().fg(Color::Cyan)),
                Span::styled(", ", Style::default().fg(Color::DarkGray)),
                Span::styled("/clear", Style::default().fg(Color::Rgb(180, 160, 255))),
                Span::styled(", ", Style::default().fg(Color::DarkGray)),
                Span::styled("/help", Style::default().fg(Color::Yellow)),
            ]));
            all_lines.push(Line::from(vec![
                Span::styled("  • Navigation:  ", Style::default().fg(Color::Rgb(165, 180, 195))),
                Span::styled("[Tab] Focus Pane · [F2~F4] Dashboard · [F5] Layout · [PageUp/Dn] Scroll", Style::default().fg(Color::DarkGray)),
            ]));
            all_lines.push(Line::from(""));
        }

        for (idx, record) in records.iter().enumerate() {
            if idx > 0 {
                all_lines.push(Line::from("")); // Card separator
            }

            // Card Header
            let mut header_spans = Vec::new();
            if let Some(node) = &record.target_node {
                let (bg_col, fg_col) = if node == "all" || node == "cluster" {
                    (Color::Rgb(60, 20, 70), Color::Rgb(255, 180, 255))
                } else {
                    (Color::Rgb(40, 30, 80), Color::Rgb(180, 160, 255))
                };
                header_spans.push(Span::styled(
                    format!(" @{} ", node),
                    Style::default().bg(bg_col).fg(fg_col).add_modifier(Modifier::BOLD),
                ));
            } else {
                header_spans.push(Span::styled(
                    " DIRECT ",
                    Style::default().bg(Color::Rgb(20, 50, 60)).fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ));
            }
            header_spans.push(Span::raw(" "));

            // Command highlight
            let cmd_color = if record.command.trim_start().starts_with('/') {
                Color::Yellow
            } else {
                Color::Cyan
            };

            header_spans.push(Span::styled(
                &record.command,
                Style::default().fg(cmd_color).add_modifier(Modifier::BOLD),
            ));

            let elapsed_str = if record.duration.as_millis() == 0 {
                let micros = record.duration.as_micros();
                if micros < 1000 {
                    format!("{:.2}ms", micros as f64 / 1000.0)
                } else {
                    format!("{}μs", micros)
                }
            } else {
                format!("{}ms", record.duration.as_millis())
            };

            header_spans.push(Span::styled(
                format!("  [{} · {}]", record.timestamp, elapsed_str),
                Style::default().fg(Color::DarkGray),
            ));

            all_lines.push(Line::from(header_spans));

            // Card Body Formatted Values
            match &record.result {
                FormattedValue::Status(s) => {
                    all_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(s, Style::default().fg(Color::Rgb(16, 185, 129)).add_modifier(Modifier::BOLD)),
                    ]));
                }
                FormattedValue::Integer(i) => {
                    all_lines.push(Line::from(vec![
                        Span::raw("  (integer) "),
                        Span::styled(i.to_string(), Style::default().fg(Color::Rgb(100, 200, 255))),
                    ]));
                }
                FormattedValue::String(s) => {
                    let clean = s.replace("\r\n", "\n").replace('\r', "");
                    for line in clean.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("--- Node:") {
                            all_lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled(line.to_string(), Style::default().fg(Color::Rgb(180, 160, 255)).add_modifier(Modifier::BOLD)),
                            ]));
                        } else if trimmed.starts_with('#') {
                            all_lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled(line.to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            ]));
                        } else if let Some((k, v)) = line.split_once(':') {
                            all_lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled(format!("{}:", k), Style::default().fg(Color::Green)),
                                Span::styled(format!(" {}", v), Style::default().fg(Color::White)),
                            ]));
                        } else {
                            all_lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled(line.to_string(), Style::default().fg(Color::White)),
                            ]));
                        }
                    }
                }
                FormattedValue::Json(json_str) => {
                    for line in json_str.lines() {
                        let highlighted = Self::syntax_highlight_json_line(line);
                        all_lines.push(highlighted);
                    }
                }
                FormattedValue::Table { headers, rows } => {
                    let table_lines = Self::render_adaptive_table(headers, rows, inner_stream.width);
                    all_lines.extend(table_lines);
                }
                FormattedValue::Tree { root, items } => {
                    all_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled("[Tree] ", Style::default().fg(Color::Yellow)),
                        Span::styled(root, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    ]));
                    for (i, (k, v)) in items.iter().enumerate() {
                        let is_last = i == items.len() - 1;
                        let branch = if is_last { "  └── " } else { "  ├── " };
                        all_lines.push(Line::from(vec![
                            Span::styled(branch, Style::default().fg(Color::DarkGray)),
                            Span::styled(format!("{}: ", k), Style::default().fg(Color::Green)),
                            Span::styled(v, Style::default().fg(Color::White)),
                        ]));
                    }
                }
                FormattedValue::List(items) => {
                    for (i, item) in items.iter().enumerate() {
                        all_lines.push(Line::from(vec![
                            Span::styled(format!("  {:>2}) ", i + 1), Style::default().fg(Color::DarkGray)),
                            Span::styled(item, Style::default().fg(Color::White)),
                        ]));
                    }
                }
                FormattedValue::Nil => {
                    all_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled("(nil)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                    ]));
                }
                FormattedValue::Error(err) => {
                    all_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(err, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    ]));
                }
            }
        }

        // Apply dimming if autocomplete is active
        if is_dimmed {
            all_lines = all_lines.into_iter().map(Self::dim_line).collect();
        }

        // Apply viewport scroll offset
        let total_lines = all_lines.len();
        let visible_height = inner_stream.height as usize;

        let start_line = if total_lines <= visible_height {
            0
        } else {
            let max_scroll = total_lines.saturating_sub(visible_height);
            let effective_scroll = scroll_offset.min(max_scroll);
            max_scroll.saturating_sub(effective_scroll)
        };

        let end_line = (start_line + visible_height).min(total_lines);

        let visible_lines: Vec<ListItem> = if total_lines > 0 && start_line < total_lines {
            all_lines[start_line..end_line]
                .iter()
                .cloned()
                .map(ListItem::new)
                .collect()
        } else {
            Vec::new()
        };

        let list_widget = List::new(visible_lines);
        f.render_widget(list_widget, inner_stream);

        // 2. Prompt / Input Bar (Always stays highlighted and bright)
        let prompt_border = if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(prompt_border)
            .title(Span::styled(
                " Prompt (Commands, / Macros, @ Node Routing, Tab Autocomplete) ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));

        let input_inner = input_block.inner(input_area);
        f.render_widget(input_block, input_area);

        let input_text = Paragraph::new(Line::from(vec![
            Span::styled("❯ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(input, Style::default().fg(Color::White)),
        ]));
        f.render_widget(input_text, input_inner);

        // Render Cursor in Prompt
        if is_focused {
            f.set_cursor_position((
                input_inner.x + 2 + cursor_pos as u16,
                input_inner.y,
            ));
        }
    }

    pub fn total_lines_count(records: &[ExecutionRecord]) -> usize {
        if records.is_empty() {
            return 8;
        }
        let mut count = 0;
        for (idx, record) in records.iter().enumerate() {
            if idx > 0 {
                count += 1;
            }
            count += 1; // Header line
            match &record.result {
                FormattedValue::Status(_) => count += 1,
                FormattedValue::Integer(_) => count += 1,
                FormattedValue::String(s) => {
                    let clean = s.replace("\r\n", "\n").replace('\r', "");
                    count += clean.lines().count().max(1);
                }
                FormattedValue::Json(json_str) => {
                    count += json_str.lines().count().max(1);
                }
                FormattedValue::Table { rows, .. } => {
                    count += rows.len() + 4;
                }
                FormattedValue::Tree { items, .. } => {
                    count += items.len() + 1;
                }
                FormattedValue::List(items) => {
                    count += items.len();
                }
                FormattedValue::Nil => count += 1,
                FormattedValue::Error(_) => count += 1,
            }
        }
        count
    }

    fn dim_line<'a>(line: Line<'a>) -> Line<'a> {
        let dimmed_spans: Vec<Span<'a>> = line
            .spans
            .into_iter()
            .map(|mut span| {
                let fg = span.style.fg.unwrap_or(Color::White);
                let dimmed_fg = match fg {
                    Color::Cyan | Color::Rgb(100, 200, 255) => Color::Rgb(60, 90, 100),
                    Color::Green | Color::Rgb(16, 185, 129) => Color::Rgb(50, 90, 60),
                    Color::Yellow | Color::Rgb(245, 158, 11) => Color::Rgb(100, 90, 50),
                    Color::Red | Color::Rgb(239, 68, 68) => Color::Rgb(110, 50, 50),
                    Color::Rgb(180, 160, 255) | Color::Rgb(147, 112, 219) => Color::Rgb(80, 70, 110),
                    Color::White => Color::Rgb(100, 105, 110),
                    Color::DarkGray | Color::Gray => Color::Rgb(50, 55, 60),
                    _ => Color::Rgb(65, 70, 75),
                };
                span.style = Style::default().fg(dimmed_fg);
                span
            })
            .collect();
        Line::from(dimmed_spans)
    }

    fn syntax_highlight_json_line(line: &str) -> Line<'static> {
        let mut spans = vec![Span::raw("  ")];

        if let Some(colon_idx) = line.find(':') {
            let key_part = line[..colon_idx].to_string();
            let val_part = line[colon_idx + 1..].to_string();

            spans.push(Span::styled(key_part, Style::default().fg(Color::Cyan)));
            spans.push(Span::styled(":", Style::default().fg(Color::DarkGray)));

            let trimmed_val = val_part.trim();
            if trimmed_val.starts_with('"') {
                spans.push(Span::styled(val_part, Style::default().fg(Color::Green)));
            } else if trimmed_val.parse::<f64>().is_ok() {
                spans.push(Span::styled(val_part, Style::default().fg(Color::Rgb(100, 200, 255))));
            } else if trimmed_val == "true" || trimmed_val == "false" || trimmed_val == "null" {
                spans.push(Span::styled(val_part, Style::default().fg(Color::Yellow)));
            } else {
                spans.push(Span::styled(val_part, Style::default().fg(Color::White)));
            }
        } else {
            let trimmed = line.trim();
            if trimmed.starts_with('"') {
                spans.push(Span::styled(line.to_string(), Style::default().fg(Color::Green)));
            } else if trimmed == "{" || trimmed == "}" || trimmed == "[" || trimmed == "]" || trimmed == "}," || trimmed == "]," {
                spans.push(Span::styled(line.to_string(), Style::default().fg(Color::DarkGray)));
            } else {
                spans.push(Span::styled(line.to_string(), Style::default().fg(Color::White)));
            }
        }

        Line::from(spans)
    }

    fn render_adaptive_table(headers: &[String], rows: &[Vec<String>], max_width: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let num_cols = headers.len();
        if num_cols == 0 {
            return lines;
        }

        // Calculate max content width for each column
        let mut col_widths = vec![4; num_cols];
        for (i, h) in headers.iter().enumerate() {
            col_widths[i] = col_widths[i].max(h.len());
        }
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < num_cols {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        // Cap total width to available space
        let total_avail = (max_width as usize).saturating_sub(4 + num_cols * 3);
        let current_total: usize = col_widths.iter().sum();
        if current_total > total_avail && total_avail > 20 {
            for w in col_widths.iter_mut() {
                *w = (*w).min(total_avail / num_cols);
            }
        }

        // Top Border: ┌─────┬─────┐
        let mut top_border = String::from("  ┌");
        for (i, w) in col_widths.iter().enumerate() {
            top_border.push_str(&"─".repeat(*w + 2));
            if i < num_cols - 1 {
                top_border.push('┬');
            } else {
                top_border.push('┐');
            }
        }
        lines.push(Line::from(Span::styled(top_border, Style::default().fg(Color::DarkGray))));

        // Header Row: │ Header 1 │ Header 2 │
        let mut h_spans = vec![Span::styled("  │", Style::default().fg(Color::DarkGray))];
        for (i, h) in headers.iter().enumerate() {
            let w = col_widths[i];
            let truncated = if h.len() > w {
                format!("{}…", &h[..w.saturating_sub(1)])
            } else {
                h.clone()
            };
            h_spans.push(Span::styled(
                format!(" {:<width$} ", truncated, width = w),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));
            h_spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
        }
        lines.push(Line::from(h_spans));

        // Middle Divider: ├─────┼─────┤
        let mut mid_border = String::from("  ├");
        for (i, w) in col_widths.iter().enumerate() {
            mid_border.push_str(&"─".repeat(*w + 2));
            if i < num_cols - 1 {
                mid_border.push('┼');
            } else {
                mid_border.push('┤');
            }
        }
        lines.push(Line::from(Span::styled(mid_border, Style::default().fg(Color::DarkGray))));

        // Data Rows
        for row in rows {
            let mut r_spans = vec![Span::styled("  │", Style::default().fg(Color::DarkGray))];
            for (i, w) in col_widths.iter().enumerate() {
                let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
                let truncated = if cell.len() > *w {
                    format!("{}…", &cell[..w.saturating_sub(1)])
                } else {
                    cell.to_string()
                };
                let cell_col = match i {
                    0 => Color::Green,
                    1 => Color::Yellow,
                    _ => Color::White,
                };
                r_spans.push(Span::styled(
                    format!(" {:<width$} ", truncated, width = *w),
                    Style::default().fg(cell_col),
                ));
                r_spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
            }
            lines.push(Line::from(r_spans));
        }

        // Bottom Border: └─────┴─────┘
        let mut bot_border = String::from("  └");
        for (i, w) in col_widths.iter().enumerate() {
            bot_border.push_str(&"─".repeat(*w + 2));
            if i < num_cols - 1 {
                bot_border.push('┴');
            } else {
                bot_border.push('┘');
            }
        }
        lines.push(Line::from(Span::styled(bot_border, Style::default().fg(Color::DarkGray))));

        lines
    }
}
