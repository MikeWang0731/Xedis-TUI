use crate::ui::theme::ThemePalette;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SlowlogEntry {
    pub id: u64,
    pub timestamp: String,
    pub latency_ms: f64,
    pub command: String,
    pub node: String,
    pub suggestion: Option<String>,
}

pub struct SlowlogView;

impl SlowlogView {
    pub fn render(f: &mut Frame, area: Rect, entries: &[SlowlogEntry], scroll_offset: usize, theme: &ThemePalette) {
        let w = area.width;
        let title_text = if w >= 48 {
            " Slowlog & Diagnostics (Threshold: >10ms) "
        } else if w >= 36 {
            " Slowlog & Diagnostics "
        } else {
            " Slowlog "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.slowlog_border))
            .title(Span::styled(
                title_text,
                Style::default().fg(theme.slowlog_border).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mut items = Vec::new();
        let width = inner.width as usize;

        // Top summary (Multi-Tier Adaptive Folding)
        if width >= 68 {
            let summary_line = Line::from(vec![
                Span::styled(" [Slowlog Monitor: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("Captured: {} ", entries.len()), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled("· Threshold: >10ms · Auto-Polled]", Style::default().fg(theme.telemetry_label)),
            ]);
            items.push(ListItem::new(vec![summary_line, Line::from("")]));
        } else if width >= 44 {
            let line1 = Line::from(vec![
                Span::styled(" [Slowlogs Captured: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{} ", entries.len()), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled("(Auto-Polled)]", Style::default().fg(theme.telemetry_label)),
            ]);
            let line2 = Line::from(vec![
                Span::styled(" [Threshold: >10ms · Auto-Diagnose]", Style::default().fg(theme.telemetry_label)),
            ]);
            items.push(ListItem::new(vec![line1, line2, Line::from("")]));
        } else {
            let summary_line = Line::from(vec![
                Span::styled(format!(" [Slowlogs: {} (>10ms)]", entries.len()), Style::default().fg(theme.telemetry_label)),
            ]);
            items.push(ListItem::new(vec![summary_line, Line::from("")]));
        }

        if entries.is_empty() {
            if width >= 62 {
                items.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("  [OK] No slow queries detected. Instance operating smoothly.", Style::default().fg(theme.status_healthy)),
                    ]),
                ]));
            } else if width >= 38 {
                items.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("  [OK] No slow queries detected.", Style::default().fg(theme.status_healthy).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("  Instance operating smoothly.", Style::default().fg(theme.telemetry_label)),
                    ]),
                ]));
            } else {
                items.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("  [OK] No slow queries", Style::default().fg(theme.status_healthy)),
                    ]),
                ]));
            }
        }

        let box_w = width.saturating_sub(1).max(24);

        for entry in entries {
            let (card_color, level_str) = if entry.latency_ms >= 50.0 {
                (theme.status_critical, "[CRITICAL]")
            } else if entry.latency_ms >= 10.0 {
                (theme.status_warning, "[WARNING]")
            } else {
                (theme.status_healthy, "[NOTICE]")
            };

            let mut lines = Vec::new();

            // Card Top Border: ╭── Slow Query #1 [CRITICAL] ────────╮
            let title = format!(" ╭── Slow Query #{} {} ", entry.id, level_str);
            let dash_count = box_w.saturating_sub(title.chars().count() + 1);
            lines.push(Line::from(vec![
                Span::styled(title, Style::default().fg(card_color).add_modifier(Modifier::BOLD)),
                Span::styled("─".repeat(dash_count), Style::default().fg(card_color)),
                Span::styled("╮", Style::default().fg(card_color)),
            ]));

            // Card Header details
            if width >= 56 {
                lines.push(Line::from(vec![
                    Span::styled(" │  ", Style::default().fg(card_color)),
                    Span::styled(format!("{:.1}ms ", entry.latency_ms), Style::default().fg(card_color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("· Time: {} · Node: @{}", entry.timestamp, entry.node), Style::default().fg(theme.text_muted)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(" │  ", Style::default().fg(card_color)),
                    Span::styled(format!("Latency: {:.1}ms ", entry.latency_ms), Style::default().fg(card_color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("(@{})", entry.node), Style::default().fg(theme.shard_node_id)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" │  ", Style::default().fg(card_color)),
                    Span::styled(format!("Time: {}", entry.timestamp), Style::default().fg(theme.text_muted)),
                ]));
            }

            // Command (with multiline wrapping)
            let cmd_max_w = box_w.saturating_sub(10).max(16);
            let cmd_wrapped = Self::wrap_text(&entry.command, cmd_max_w);
            for (i, cmd_part) in cmd_wrapped.iter().enumerate() {
                if i == 0 {
                    lines.push(Line::from(vec![
                        Span::styled(" │  Cmd: ", Style::default().fg(card_color)),
                        Span::styled(cmd_part.clone(), Style::default().fg(theme.slowlog_cmd).add_modifier(Modifier::BOLD)),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(" │       ", Style::default().fg(card_color)),
                        Span::styled(cmd_part.clone(), Style::default().fg(theme.slowlog_cmd).add_modifier(Modifier::BOLD)),
                    ]));
                }
            }

            // Optimization guidance (with multiline wrapping)
            if let Some(sug) = &entry.suggestion {
                let sug_max_w = box_w.saturating_sub(16).max(16);
                let sug_wrapped = Self::wrap_text(sug, sug_max_w);
                for (i, part) in sug_wrapped.iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(" │  Guidance: ", Style::default().fg(card_color).add_modifier(Modifier::BOLD)),
                            Span::styled(part.clone(), Style::default().fg(theme.slowlog_guidance_text)),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(" │            ", Style::default().fg(card_color)),
                            Span::styled(part.clone(), Style::default().fg(theme.slowlog_guidance_text)),
                        ]));
                    }
                }
            }

            // Card Bottom Border: ╰──────────────────────────────────╯
            let bot_dash_count = box_w.saturating_sub(2);
            lines.push(Line::from(vec![
                Span::styled(" ╰", Style::default().fg(card_color)),
                Span::styled("─".repeat(bot_dash_count), Style::default().fg(card_color)),
                Span::styled("╯", Style::default().fg(card_color)),
            ]));

            lines.push(Line::from(""));
            items.push(ListItem::new(lines));
        }

        let total_items = items.len();
        let visible_items = if scroll_offset < total_items {
            items.into_iter().skip(scroll_offset).collect()
        } else {
            items
        };

        let list = List::new(visible_items);
        f.render_widget(list, inner);
    }

    /// Word-wrapping with Chinese double-width character awareness
    fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for ch in text.chars() {
            let ch_w = if ch.is_ascii() { 1 } else { 2 };
            if current_width + ch_w > max_width && !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
                current_width = 0;
            }
            current_line.push(ch);
            current_width += ch_w;
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        lines
    }
}
