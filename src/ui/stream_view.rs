use crate::backend::formatter::FormattedValue;
use crate::ui::theme::ThemePalette;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
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
        theme: &ThemePalette,
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
                Style::default().fg(theme.stream_border_dimmed),
                Style::default().fg(theme.text_dimmed),
            )
        } else if is_focused {
            (
                Style::default().fg(theme.stream_border_focused),
                Style::default().fg(theme.stream_border_focused).add_modifier(Modifier::BOLD),
            )
        } else {
            (
                Style::default().fg(theme.stream_border_unfocused),
                Style::default().fg(theme.stream_border_unfocused),
            )
        };

        let stream_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(Span::styled(" Command Stream ", title_style));

        let inner_stream = stream_block.inner(stream_area);
        f.render_widget(stream_block, stream_area);

        let max_w = inner_stream.width;
        let visible_height = inner_stream.height as usize;
        let total_lines = Self::total_lines_count(records, max_w);

        let start_line = if total_lines <= visible_height {
            0
        } else {
            let max_scroll = total_lines.saturating_sub(visible_height);
            let effective_scroll = scroll_offset.min(max_scroll);
            max_scroll.saturating_sub(effective_scroll)
        };
        let end_line = (start_line + visible_height).min(total_lines);

        let visible_lines: Vec<ListItem> = if records.is_empty() {
            let welcome = Self::render_welcome_banner(max_w, theme);
            let sliced = if start_line < welcome.len() {
                let end = (start_line + visible_height).min(welcome.len());
                welcome[start_line..end]
                    .iter()
                    .cloned()
                    .map(|l| if is_dimmed { Self::dim_line(l, theme) } else { l })
                    .map(ListItem::new)
                    .collect()
            } else {
                Vec::new()
            };
            sliced
        } else {
            Self::render_virtual_lines(records, start_line, end_line, max_w, is_dimmed, theme)
                .into_iter()
                .map(ListItem::new)
                .collect()
        };

        let list_widget = List::new(visible_lines);
        f.render_widget(list_widget, inner_stream);

        // 2. Prompt / Input Bar
        let prompt_border = if is_focused {
            Style::default().fg(theme.prompt_border_focused)
        } else {
            Style::default().fg(theme.prompt_border_unfocused)
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(prompt_border)
            .title(Span::styled(
                " Prompt (Commands, / Macros, @ Node Routing, Tab Autocomplete) ",
                Style::default().fg(theme.prompt_border_focused).add_modifier(Modifier::BOLD),
            ));

        let input_inner = input_block.inner(input_area);
        f.render_widget(input_block, input_area);

        let input_text = Paragraph::new(Line::from(vec![
            Span::styled("❯ ", Style::default().fg(theme.prompt_symbol).add_modifier(Modifier::BOLD)),
            Span::styled(input, Style::default().fg(theme.prompt_input)),
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

    /// Virtual viewport renderer: only allocates and renders lines in [start_line, end_line)
    pub fn render_virtual_lines(
        records: &[ExecutionRecord],
        start_line: usize,
        end_line: usize,
        max_width: u16,
        is_dimmed: bool,
        theme: &ThemePalette,
    ) -> Vec<Line<'static>> {
        if start_line >= end_line || records.is_empty() {
            return Vec::new();
        }

        let mut current_line_offset = 0;
        let mut result_lines = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            let record_lines_count = Self::single_record_line_count(record, idx > 0, max_width);
            let record_end_offset = current_line_offset + record_lines_count;

            // Check if this record intersects with [start_line, end_line)
            if record_end_offset > start_line && current_line_offset < end_line {
                let card_lines = Self::render_single_record(record, idx > 0, max_width, theme);

                let rec_start = if start_line > current_line_offset {
                    start_line - current_line_offset
                } else {
                    0
                };

                let rec_end = if end_line < record_end_offset {
                    end_line - current_line_offset
                } else {
                    card_lines.len()
                };

                if rec_start < card_lines.len() {
                    let safe_end = rec_end.min(card_lines.len());
                    for line in &card_lines[rec_start..safe_end] {
                        if is_dimmed {
                            result_lines.push(Self::dim_line(line.clone(), theme));
                        } else {
                            result_lines.push(line.clone());
                        }
                    }
                }
            }

            current_line_offset = record_end_offset;
            if current_line_offset >= end_line {
                break;
            }
        }

        result_lines
    }

    pub fn render_welcome_banner(max_width: u16, theme: &ThemePalette) -> Vec<Line<'static>> {
        let w = (max_width as usize).saturating_sub(4);
        let mut lines = Vec::new();
        lines.push(Line::from(""));

        // Header Title
        if w >= 75 {
            lines.push(Line::from(vec![
                Span::styled(" [XEDIS] ", Style::default().bg(theme.brand_bg).fg(theme.brand_fg).add_modifier(Modifier::BOLD)),
                Span::styled(" Welcome to Xedis-TUI", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                Span::styled(" · Terminal Workbench for Redis & Custom Middleware", Style::default().fg(theme.text_secondary)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(" [XEDIS] ", Style::default().bg(theme.brand_bg).fg(theme.brand_fg).add_modifier(Modifier::BOLD)),
                Span::styled(" Welcome to Xedis-TUI", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled("Terminal Workbench for Redis & Custom Middleware", Style::default().fg(theme.text_secondary)),
            ]));
        }
        lines.push(Line::from(""));

        // Section header
        lines.push(Line::from(vec![
            Span::styled(" Quick Start & Tips:", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
        ]));

        // Tip 1: Commands
        if w >= 80 {
            lines.push(Line::from(vec![
                Span::styled("  • Command:     ", Style::default().fg(theme.text_secondary)),
                Span::styled("Type any Redis command (e.g. ", Style::default().fg(theme.text_muted)),
                Span::styled("PING", Style::default().fg(theme.cmd_name_native)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("INFO", Style::default().fg(theme.cmd_name_native)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("GET <key>", Style::default().fg(theme.cmd_name_native)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("SET <k> <v>", Style::default().fg(theme.cmd_name_native)),
                Span::styled(") and press Enter", Style::default().fg(theme.text_muted)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("  • Command: ", Style::default().fg(theme.cmd_name_native).add_modifier(Modifier::BOLD)),
                Span::styled("PING", Style::default().fg(theme.cmd_name_native)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("INFO", Style::default().fg(theme.cmd_name_native)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("GET <key>", Style::default().fg(theme.cmd_name_native)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("SET <k> <v>", Style::default().fg(theme.cmd_name_native)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("Type any command and press Enter to execute", Style::default().fg(theme.text_muted)),
            ]));
        }

        // Tip 2: Macros
        if w >= 75 {
            lines.push(Line::from(vec![
                Span::styled("  • Macros:      ", Style::default().fg(theme.text_secondary)),
                Span::styled("Type ", Style::default().fg(theme.text_muted)),
                Span::styled("/scan", Style::default().fg(theme.help_title_yellow)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("/bigkeys", Style::default().fg(theme.val_key)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("/theme", Style::default().fg(theme.help_title_cyan)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("/interval", Style::default().fg(theme.help_title_purple)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("/clear", Style::default().fg(theme.text_muted)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("/help", Style::default().fg(theme.help_title_yellow)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("  • Macros:  ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("/scan", Style::default().fg(theme.help_title_yellow)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("/theme", Style::default().fg(theme.help_title_cyan)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("/slowlog", Style::default().fg(theme.help_title_yellow)),
                Span::styled(", ", Style::default().fg(theme.text_muted)),
                Span::styled("/help", Style::default().fg(theme.help_title_yellow)),
            ]));
        }

        // Tip 3: Navigation
        if w >= 88 {
            lines.push(Line::from(vec![
                Span::styled("  • Navigation:  ", Style::default().fg(theme.text_secondary)),
                Span::styled("[Tab] Focus Pane · [F1] Handbook · [F2~F4] Dashboard · [F5] Layout · [PageUp/Dn] Scroll", Style::default().fg(theme.text_muted)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("  • Navigation: ", Style::default().fg(theme.cmd_name_native)),
                Span::styled("[Tab] Focus · [F1] Handbook · [F5] Layout · [F2~F4] Dash", Style::default().fg(theme.text_muted)),
            ]));
        }

        lines.push(Line::from(""));
        lines
    }

    pub fn single_record_line_count(record: &ExecutionRecord, has_separator: bool, max_width: u16) -> usize {
        let mut count = if has_separator { 1 } else { 0 };
        count += 1; // Header line

        let content_width = (max_width as usize).saturating_sub(6).max(20);

        match &record.result {
            FormattedValue::Status(_) => count += 1,
            FormattedValue::Integer(_) => count += 1,
            FormattedValue::String(s) => {
                let clean = s.replace("\r\n", "\n").replace('\r', "");
                for line in clean.lines() {
                    let wrapped_cnt = Self::wrap_count(line, content_width);
                    count += wrapped_cnt.max(1);
                }
            }
            FormattedValue::Json(json_str) => {
                for line in json_str.lines() {
                    let wrapped_cnt = Self::wrap_count(line, content_width);
                    count += wrapped_cnt.max(1);
                }
            }
            FormattedValue::Table { rows, .. } => {
                count += rows.len() + 4; // top border + header + mid border + bot border
            }
            FormattedValue::Tree { items, .. } => {
                count += items.len() + 1; // root + items
            }
            FormattedValue::List(items) => {
                for item in items {
                    let wrapped_cnt = Self::wrap_count(item, content_width.saturating_sub(6));
                    count += wrapped_cnt.max(1);
                }
            }
            FormattedValue::Nil => count += 1,
            FormattedValue::Error(err) => {
                let wrapped_cnt = Self::wrap_count(err, content_width);
                count += wrapped_cnt.max(1);
            }
        }
        count
    }

    pub fn total_lines_count(records: &[ExecutionRecord], max_width: u16) -> usize {
        if records.is_empty() {
            return Self::render_welcome_banner(max_width, &ThemePalette::dark()).len();
        }
        let mut count = 0;
        for (idx, record) in records.iter().enumerate() {
            count += Self::single_record_line_count(record, idx > 0, max_width);
        }
        count
    }

    fn wrap_count(text: &str, max_width: usize) -> usize {
        if text.len() <= max_width || max_width == 0 {
            1
        } else {
            (text.len() + max_width - 1) / max_width
        }
    }

    pub fn render_single_record(
        record: &ExecutionRecord,
        has_separator: bool,
        max_width: u16,
        theme: &ThemePalette,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        if has_separator {
            lines.push(Line::from(""));
        }

        let avail_w = (max_width as usize).saturating_sub(4).max(20);

        // Card Header
        let mut header_spans = Vec::new();
        if let Some(node) = &record.target_node {
            let (bg_col, fg_col) = if node == "all" || node == "cluster" {
                (theme.cmd_broadcast_bg, theme.cmd_broadcast_fg)
            } else {
                (theme.cmd_node_bg, theme.cmd_node_fg)
            };
            header_spans.push(Span::styled(
                format!(" @{} ", node),
                Style::default().bg(bg_col).fg(fg_col).add_modifier(Modifier::BOLD),
            ));
        } else {
            header_spans.push(Span::styled(
                " DIRECT ",
                Style::default().bg(theme.cmd_direct_bg).fg(theme.cmd_direct_fg).add_modifier(Modifier::BOLD),
            ));
        }
        header_spans.push(Span::raw(" "));

        let cmd_color = if record.command.trim_start().starts_with('/') {
            theme.cmd_name_macro
        } else {
            theme.cmd_name_native
        };

        header_spans.push(Span::styled(
            record.command.clone(),
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
            Style::default().fg(theme.cmd_meta),
        ));

        lines.push(Line::from(header_spans));

        // Card Body with adaptive wrapping
        match &record.result {
            FormattedValue::Status(s) => {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(s.clone(), Style::default().fg(theme.val_status).add_modifier(Modifier::BOLD)),
                ]));
            }
            FormattedValue::Integer(i) => {
                lines.push(Line::from(vec![
                    Span::styled("  (integer) ", Style::default().fg(theme.text_secondary)),
                    Span::styled(i.to_string(), Style::default().fg(theme.val_integer)),
                ]));
            }
            FormattedValue::String(s) => {
                let clean = s.replace("\r\n", "\n").replace('\r', "");
                for line in clean.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("--- Node:") {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(line.to_string(), Style::default().fg(theme.val_node_header).add_modifier(Modifier::BOLD)),
                        ]));
                    } else if trimmed.starts_with('#') {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(line.to_string(), Style::default().fg(theme.val_info_section).add_modifier(Modifier::BOLD)),
                        ]));
                    } else if let Some((k, v)) = line.split_once(':') {
                        let total_len = line.len();
                        if total_len > avail_w && avail_w > 20 {
                            let chunks = Self::chunk_string(v.trim(), avail_w.saturating_sub(k.len() + 6));
                            for (ci, chunk) in chunks.iter().enumerate() {
                                if ci == 0 {
                                    lines.push(Line::from(vec![
                                        Span::raw("  "),
                                        Span::styled(format!("{}:", k), Style::default().fg(theme.val_key)),
                                        Span::styled(format!(" {}", chunk), Style::default().fg(theme.val_string)),
                                    ]));
                                } else {
                                    lines.push(Line::from(vec![
                                        Span::raw("    "),
                                        Span::styled(chunk.clone(), Style::default().fg(theme.val_string)),
                                    ]));
                                }
                            }
                        } else {
                            lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled(format!("{}:", k), Style::default().fg(theme.val_key)),
                                Span::styled(format!(" {}", v), Style::default().fg(theme.val_string)),
                            ]));
                        }
                    } else {
                        let chunks = Self::chunk_string(line, avail_w.saturating_sub(4));
                        for chunk in chunks {
                            lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled(chunk, Style::default().fg(theme.val_string)),
                            ]));
                        }
                    }
                }
            }
            FormattedValue::Json(json_str) => {
                for line in json_str.lines() {
                    let chunks = Self::chunk_string(line, avail_w.saturating_sub(4));
                    for chunk in chunks {
                        lines.push(Self::syntax_highlight_json_line(&chunk, theme));
                    }
                }
            }
            FormattedValue::Table { headers, rows } => {
                let table_lines = Self::render_adaptive_table(headers, rows, max_width, theme);
                lines.extend(table_lines);
            }
            FormattedValue::Tree { root, items } => {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("[Tree] ", Style::default().fg(theme.val_tree_tag)),
                    Span::styled(root.clone(), Style::default().fg(theme.val_tree_root).add_modifier(Modifier::BOLD)),
                ]));
                for (i, (k, v)) in items.iter().enumerate() {
                    let is_last = i == items.len() - 1;
                    let branch = if is_last { "  └── " } else { "  ├── " };
                    lines.push(Line::from(vec![
                        Span::styled(branch, Style::default().fg(theme.val_tree_branch)),
                        Span::styled(format!("{}: ", k), Style::default().fg(theme.val_key)),
                        Span::styled(v.clone(), Style::default().fg(theme.val_string)),
                    ]));
                }
            }
            FormattedValue::List(items) => {
                for (i, item) in items.iter().enumerate() {
                    let chunks = Self::chunk_string(item, avail_w.saturating_sub(8));
                    for (ci, chunk) in chunks.iter().enumerate() {
                        if ci == 0 {
                            lines.push(Line::from(vec![
                                Span::styled(format!("  {:>2}) ", i + 1), Style::default().fg(theme.text_muted)),
                                Span::styled(chunk.clone(), Style::default().fg(theme.val_string)),
                            ]));
                        } else {
                            lines.push(Line::from(vec![
                                Span::raw("      "),
                                Span::styled(chunk.clone(), Style::default().fg(theme.val_string)),
                            ]));
                        }
                    }
                }
            }
            FormattedValue::Nil => {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("(nil)", Style::default().fg(theme.val_nil).add_modifier(Modifier::ITALIC)),
                ]));
            }
            FormattedValue::Error(err) => {
                let chunks = Self::chunk_string(err, avail_w.saturating_sub(4));
                for chunk in chunks {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(chunk, Style::default().fg(theme.val_error).add_modifier(Modifier::BOLD)),
                    ]));
                }
            }
        }

        lines
    }

    fn chunk_string(s: &str, chunk_size: usize) -> Vec<String> {
        let size = chunk_size.max(10);
        if s.len() <= size {
            return vec![s.to_string()];
        }
        let mut result = Vec::new();
        let mut chars = s.chars().peekable();
        while chars.peek().is_some() {
            let chunk: String = chars.by_ref().take(size).collect();
            result.push(chunk);
        }
        result
    }

    pub fn dim_line<'a>(line: Line<'a>, theme: &ThemePalette) -> Line<'a> {
        let dimmed_spans: Vec<Span<'a>> = line
            .spans
            .into_iter()
            .map(|mut span| {
                span.style = Style::default().fg(theme.text_dimmed);
                span
            })
            .collect();
        Line::from(dimmed_spans)
    }

    fn syntax_highlight_json_line(line: &str, theme: &ThemePalette) -> Line<'static> {
        let mut spans = vec![Span::raw("  ")];

        if let Some(colon_idx) = line.find(':') {
            let key_part = line[..colon_idx].to_string();
            let val_part = line[colon_idx + 1..].to_string();

            spans.push(Span::styled(key_part, Style::default().fg(theme.json_key)));
            spans.push(Span::styled(":", Style::default().fg(theme.json_colon)));

            let trimmed_val = val_part.trim();
            if trimmed_val.starts_with('"') {
                spans.push(Span::styled(val_part, Style::default().fg(theme.json_string)));
            } else if trimmed_val.parse::<f64>().is_ok() {
                spans.push(Span::styled(val_part, Style::default().fg(theme.json_number)));
            } else if trimmed_val == "true" || trimmed_val == "false" || trimmed_val == "null" {
                spans.push(Span::styled(val_part, Style::default().fg(theme.json_boolean)));
            } else {
                spans.push(Span::styled(val_part, Style::default().fg(theme.val_string)));
            }
        } else {
            let trimmed = line.trim();
            if trimmed.starts_with('"') {
                spans.push(Span::styled(line.to_string(), Style::default().fg(theme.json_string)));
            } else if trimmed == "{" || trimmed == "}" || trimmed == "[" || trimmed == "]" || trimmed == "}," || trimmed == "]," {
                spans.push(Span::styled(line.to_string(), Style::default().fg(theme.json_bracket)));
            } else {
                spans.push(Span::styled(line.to_string(), Style::default().fg(theme.val_string)));
            }
        }

        Line::from(spans)
    }

    fn render_adaptive_table(headers: &[String], rows: &[Vec<String>], max_width: u16, theme: &ThemePalette) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let num_cols = headers.len();
        if num_cols == 0 {
            return lines;
        }

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
        lines.push(Line::from(Span::styled(top_border, Style::default().fg(theme.table_border))));

        // Header Row: │ Header 1 │ Header 2 │
        let mut h_spans = vec![Span::styled("  │", Style::default().fg(theme.table_border))];
        for (i, h) in headers.iter().enumerate() {
            let w = col_widths[i];
            let truncated = if h.len() > w {
                format!("{}…", &h[..w.saturating_sub(1)])
            } else {
                h.clone()
            };
            h_spans.push(Span::styled(
                format!(" {:<width$} ", truncated, width = w),
                Style::default().fg(theme.table_header).add_modifier(Modifier::BOLD),
            ));
            h_spans.push(Span::styled("│", Style::default().fg(theme.table_border)));
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
        lines.push(Line::from(Span::styled(mid_border, Style::default().fg(theme.table_border))));

        // Data Rows
        for row in rows {
            let mut r_spans = vec![Span::styled("  │", Style::default().fg(theme.table_border))];
            for (i, w) in col_widths.iter().enumerate() {
                let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
                let truncated = if cell.len() > *w {
                    format!("{}…", &cell[..w.saturating_sub(1)])
                } else {
                    cell.to_string()
                };
                let cell_col = match i {
                    0 => theme.table_col1,
                    1 => theme.table_col2,
                    _ => theme.table_col_rest,
                };
                r_spans.push(Span::styled(
                    format!(" {:<width$} ", truncated, width = *w),
                    Style::default().fg(cell_col),
                ));
                r_spans.push(Span::styled("│", Style::default().fg(theme.table_border)));
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
        lines.push(Line::from(Span::styled(bot_border, Style::default().fg(theme.table_border))));

        lines
    }
}
