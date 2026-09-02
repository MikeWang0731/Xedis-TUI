use crate::backend::TelemetryData;
use crate::ui::theme::ThemePalette;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub struct TelemetryView;

impl TelemetryView {
    pub fn render(f: &mut Frame, area: Rect, data: &TelemetryData, poll_interval_ms: u64, is_paused: bool, theme: &ThemePalette) {
        // Adapt layout based on available width/height
        if area.width < 34 {
            Self::render_compact(f, area, data, poll_interval_ms, is_paused, theme);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // 1. Memory Proportion & Breakdown Card
                Constraint::Length(7), // 2. CPU Usage & Line Waveform Card
                Constraint::Length(6), // 3. Core Metrics & Health Grid (with Throughput)
                Constraint::Min(4),    // 4. Polling & Diagnostics Tips
            ])
            .split(area);

        // 1. Memory Card (Proportion Bar + Indicators + Key Stats)
        Self::render_memory_card(f, chunks[0], data, theme);

        // 2. CPU Card (Continuous Line Waveform + Main & Fork/Children Breakdown)
        Self::render_cpu_card(f, chunks[1], data, theme);

        // 3. Core Metrics Card (Throughput + Hit Rate + Health)
        Self::render_metrics_card(f, chunks[2], data, poll_interval_ms, is_paused, theme);

        // 4. Quick Tips Card
        Self::render_tips_card(f, chunks[3], data, theme);
    }

    fn render_memory_card(f: &mut Frame, area: Rect, data: &TelemetryData, theme: &ThemePalette) {
        let max_b = data.metrics.max_memory_bytes.unwrap_or(0);
        let used_b = data.metrics.used_memory_bytes.unwrap_or(0);
        let rss_b = data.metrics.used_memory_rss_bytes.unwrap_or(used_b);

        let mem_ratio = if max_b > 0 {
            (used_b as f64 / max_b as f64).clamp(0.0, 1.0)
        } else if used_b > 0 {
            0.53 // dynamic scale indicator for unlimited
        } else {
            0.0
        };

        let w = area.width;

        // Determine border color by memory pressure
        let (border_color, status_label, status_color) = if max_b > 0 && mem_ratio > 0.90 {
            (theme.status_critical, if w >= 45 { "[CRITICAL] > 90%" } else { "[CRIT]" }, theme.status_critical)
        } else if max_b > 0 && mem_ratio > 0.70 {
            (theme.status_warning, if w >= 45 { "[WARNING] > 70%" } else { "[WARN]" }, theme.status_warning)
        } else {
            (theme.border_focused, if w >= 45 { "[HEALTHY] Normal" } else { "[OK]" }, theme.status_healthy)
        };

        let max_mem_str = data.metrics.max_memory_display();
        let used_mem_str = data.metrics.used_memory_display();

        let title_text = if w >= 48 {
            format!(" Memory ( {} / {} ) · {} ", used_mem_str, max_mem_str, status_label)
        } else {
            format!(" Memory ( {} ) · {} ", used_mem_str, status_label)
        };

        let mem_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                title_text,
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ));

        let inner = mem_block.inner(area);
        f.render_widget(mem_block, area);

        let bar_width = inner.width as usize;
        if bar_width == 0 {
            return;
        }

        // 1. Calculate Multi-Segment Proportions
        let scale_total = if max_b > 0 {
            max_b as f64
        } else {
            (used_b.max(rss_b) as f64 * 1.35).max(1024.0 * 1024.0)
        };

        let used_cols = ((used_b as f64 / scale_total) * (bar_width as f64)).round() as usize;
        let used_cols = used_cols.min(bar_width);

        let rss_extra = rss_b.saturating_sub(used_b);
        let rss_cols = if rss_extra > 0 {
            let cols = ((rss_extra as f64 / scale_total) * (bar_width as f64)).round() as usize;
            cols.min(bar_width - used_cols)
        } else {
            0
        };

        let free_cols = bar_width.saturating_sub(used_cols + rss_cols);

        let mut bar_spans = Vec::new();
        if used_cols > 0 {
            bar_spans.push(Span::styled("█".repeat(used_cols), Style::default().fg(theme.mem_bar_used)));
        }
        if rss_cols > 0 {
            bar_spans.push(Span::styled("█".repeat(rss_cols), Style::default().fg(theme.mem_bar_rss)));
        }
        if free_cols > 0 {
            bar_spans.push(Span::styled("█".repeat(free_cols), Style::default().fg(theme.mem_bar_free)));
        }

        // 2. Legend Row (Used Memory | Fragmentation Ratio | Max Memory)
        let legend_spans = if w >= 52 {
            vec![
                Span::styled("■ ", Style::default().fg(theme.mem_bar_used)),
                Span::styled("Used Memory   ", Style::default().fg(theme.telemetry_label)),
                Span::styled("■ ", Style::default().fg(theme.mem_bar_rss)),
                Span::styled("Frag Ratio   ", Style::default().fg(theme.telemetry_label)),
                Span::styled("■ ", Style::default().fg(theme.mem_bar_free)),
                Span::styled("Max Memory", Style::default().fg(theme.telemetry_label)),
            ]
        } else {
            vec![
                Span::styled("■ ", Style::default().fg(theme.mem_bar_used)),
                Span::styled("Used  ", Style::default().fg(theme.telemetry_label)),
                Span::styled("■ ", Style::default().fg(theme.mem_bar_rss)),
                Span::styled("Frag  ", Style::default().fg(theme.telemetry_label)),
                Span::styled("■ ", Style::default().fg(theme.mem_bar_free)),
                Span::styled("Max", Style::default().fg(theme.telemetry_label)),
            ]
        };

        // 3. Details Rows (Multi-Tier Adaptive Folding)
        let frag_color = if let Some(ratio) = data.metrics.mem_fragmentation_ratio {
            if ratio > 1.8 { theme.status_warning } else if ratio < 0.9 { theme.status_critical } else { theme.status_healthy }
        } else {
            theme.telemetry_label
        };

        let mut content = vec![
            Line::from(bar_spans),
            Line::from(legend_spans),
        ];

        if w >= 64 {
            let details_spans = vec![
                Span::styled("Used: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(data.metrics.used_memory_display(), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled(" | ", Style::default().fg(theme.divider)),
                Span::styled("RSS: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(data.metrics.rss_display(), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled(" | ", Style::default().fg(theme.divider)),
                Span::styled("Max: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(data.metrics.max_memory_display(), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled(" | ", Style::default().fg(theme.divider)),
                Span::styled("Frag: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(data.metrics.frag_display(), Style::default().fg(frag_color).add_modifier(Modifier::BOLD)),
            ];
            content.push(Line::from(details_spans));
        } else {
            let line1 = vec![
                Span::styled("Used: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{:<10} ", data.metrics.used_memory_display()), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled("RSS: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(data.metrics.rss_display(), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
            ];
            let line2 = vec![
                Span::styled("Max:  ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{:<10} ", data.metrics.max_memory_display()), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled("Frag: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(data.metrics.frag_display(), Style::default().fg(frag_color).add_modifier(Modifier::BOLD)),
            ];
            content.push(Line::from(line1));
            content.push(Line::from(line2));
        }

        f.render_widget(Paragraph::new(content), inner);
    }

    fn render_cpu_card(f: &mut Frame, area: Rect, data: &TelemetryData, theme: &ThemePalette) {
        let w = area.width;
        let cpu_color = if data.metrics.cpu_usage_pct > 80.0 {
            theme.status_critical
        } else if data.metrics.cpu_usage_pct > 50.0 {
            theme.status_warning
        } else {
            theme.border_focused
        };

        let title_text = if w >= 48 {
            format!(
                " CPU Usage · {:.1}% (Max: {:.0}% · Avg: {:.1}%) ",
                data.metrics.cpu_usage_pct, data.history.max_cpu(), data.history.avg_cpu()
            )
        } else {
            format!(" CPU: {:.1}% ", data.metrics.cpu_usage_pct)
        };

        let cpu_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(cpu_color))
            .title(Span::styled(
                title_text,
                Style::default().fg(theme.status_healthy).add_modifier(Modifier::BOLD),
            ));

        let inner = cpu_block.inner(area);
        f.render_widget(cpu_block, area);

        let sub_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Smooth continuous Braille line waveform
                Constraint::Length(2), // Main CPU & Fork/Children breakdown lines
            ])
            .split(inner);

        // 1. Render Continuous Line Waveform using Braille sub-pixel grid
        let wave_width = sub_chunks[0].width as usize;
        let wave_height = sub_chunks[0].height as usize;
        let wave_lines = Self::render_line_waveform(&data.history.cpu_float_slice(), wave_width, wave_height, theme);
        f.render_widget(Paragraph::new(wave_lines), sub_chunks[0]);

        // 2. Render CPU Breakdown details (Main Process/Thread + Fork/Children)
        let main_str = data.metrics.main_cpu_display();
        let children_str = data.metrics.children_cpu_display();

        let breakdown_lines = vec![
            Line::from(vec![
                Span::styled(" Main: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(main_str, Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(" Fork/Children: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(
                    children_str.clone(),
                    if children_str == "N/A" {
                        Style::default().fg(theme.text_muted)
                    } else {
                        Style::default().fg(theme.status_warning).add_modifier(Modifier::BOLD)
                    },
                ),
            ]),
        ];

        f.render_widget(Paragraph::new(breakdown_lines), sub_chunks[1]);
    }

    /// Renders a continuous, smooth line waveform using 2x4 sub-pixel Braille characters
    fn render_line_waveform(data: &[f64], cols: usize, rows: usize, theme: &ThemePalette) -> Vec<Line<'static>> {
        if cols == 0 || rows == 0 {
            return Vec::new();
        }

        let sub_w = cols * 2;
        let sub_h = rows * 4;
        let mut grid = vec![vec![0u8; cols]; rows];

        if !data.is_empty() {
            // Find max/min range for dynamic scaling (min 20.0 for visual amplitude)
            let max_val = data.iter().copied().fold(20.0f64, f64::max);
            let min_val = 0.0f64;
            let val_range = (max_val - min_val).max(1.0);

            // Interpolate points across all sub_w columns
            let mut prev_pt: Option<(usize, usize)> = None;

            for sx in 0..sub_w {
                let data_idx = ((sx as f64 / sub_w as f64) * (data.len() as f64)).min(data.len() as f64 - 1.0);
                let idx_floor = data_idx.floor() as usize;
                let idx_ceil = (idx_floor + 1).min(data.len() - 1);
                let frac = data_idx - idx_floor as f64;

                let val = data[idx_floor] * (1.0 - frac) + data[idx_ceil] * frac;
                let norm = ((val - min_val) / val_range).clamp(0.0, 1.0);
                let sy = ((1.0 - norm) * ((sub_h - 1) as f64)).round() as usize;
                let sy = sy.min(sub_h - 1);

                if let Some((px, py)) = prev_pt {
                    // Connect (px, py) to (sx, sy) with Bresenham segment
                    Self::draw_braille_line(&mut grid, px, py, sx, sy);
                } else {
                    Self::set_braille_dot(&mut grid, sx, sy);
                }
                prev_pt = Some((sx, sy));
            }
        }

        let mut lines = Vec::new();
        for r in 0..rows {
            let mut s = String::with_capacity(cols);
            for c in 0..cols {
                let mask = grid[r][c];
                let ch = if mask == 0 {
                    ' '
                } else {
                    char::from_u32(0x2800 + mask as u32).unwrap_or(' ')
                };
                s.push(ch);
            }
            lines.push(Line::from(Span::styled(
                s,
                Style::default().fg(theme.cpu_waveform),
            )));
        }

        lines
    }

    fn set_braille_dot(grid: &mut [Vec<u8>], sx: usize, sy: usize) {
        let cx = sx / 2;
        let cy = sy / 4;
        let dx = sx % 2;
        let dy = sy % 4;

        if cy < grid.len() && cx < grid[0].len() {
            let bit = match (dx, dy) {
                (0, 0) => 0x01,
                (0, 1) => 0x02,
                (0, 2) => 0x04,
                (1, 0) => 0x08,
                (1, 1) => 0x10,
                (1, 2) => 0x20,
                (0, 3) => 0x40,
                (1, 3) => 0x80,
                _ => 0,
            };
            grid[cy][cx] |= bit;
        }
    }

    fn draw_braille_line(grid: &mut [Vec<u8>], x0: usize, y0: usize, x1: usize, y1: usize) {
        let mut x = x0 as isize;
        let mut y = y0 as isize;
        let target_x = x1 as isize;
        let target_y = y1 as isize;

        let dx = (target_x - x).abs();
        let sx = if x < target_x { 1 } else { -1 };
        let dy = -(target_y - y).abs();
        let sy = if y < target_y { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x >= 0 && y >= 0 {
                Self::set_braille_dot(grid, x as usize, y as usize);
            }
            if x == target_x && y == target_y {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn render_metrics_card(f: &mut Frame, area: Rect, data: &TelemetryData, poll_interval_ms: u64, is_paused: bool, theme: &ThemePalette) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.telemetry_card_border))
            .title(Span::styled(
                " Core Metrics & Health ",
                Style::default().fg(theme.status_warning).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let poll_badge = if is_paused || poll_interval_ms == 0 {
            Span::styled("Paused", Style::default().fg(theme.status_warning).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!("{}ms", poll_interval_ms), Style::default().fg(theme.status_healthy).add_modifier(Modifier::BOLD))
        };

        let hit_rate_color = if let Some(rate) = data.metrics.hit_rate_pct {
            if rate > 90.0 { theme.status_healthy } else if rate > 70.0 { theme.status_warning } else { theme.status_critical }
        } else {
            theme.telemetry_label
        };

        let lines = vec![
            Line::from(vec![
                Span::styled(" Clients: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{:<5}", data.metrics.connected_clients), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled(" Keys: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{:<8}", crate::core::telemetry::TelemetryParser::format_number(data.metrics.total_keys)), Style::default().fg(theme.border_focused).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(" Hit Rate: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(data.metrics.hit_rate_display(), Style::default().fg(hit_rate_color).add_modifier(Modifier::BOLD)),
                Span::styled(" Up: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(&data.metrics.uptime_human, Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(" Throughput: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{}/s", data.metrics.ops_display()), Style::default().fg(theme.status_healthy).add_modifier(Modifier::BOLD)),
                Span::styled(" · Net: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(data.metrics.net_display(), Style::default().fg(theme.telemetry_value)),
            ]),
            Line::from(vec![
                Span::styled(" Polling Interval: ", Style::default().fg(theme.telemetry_label)),
                poll_badge,
            ]),
        ];

        f.render_widget(Paragraph::new(lines), inner);
    }

    fn render_tips_card(f: &mut Frame, area: Rect, _data: &TelemetryData, theme: &ThemePalette) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.telemetry_card_border))
            .title(Span::styled(
                " Quick Guidance ",
                Style::default().fg(theme.conn_cluster).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let lines = vec![
            Line::from(vec![
                Span::styled("• /theme [dark|light|toggle] ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("UI Theme", Style::default().fg(theme.telemetry_label)),
            ]),
            Line::from(vec![
                Span::styled("• /interval [ms|s|pause] ", Style::default().fg(theme.cmd_name_native).add_modifier(Modifier::BOLD)),
                Span::styled("Polling rate", Style::default().fg(theme.telemetry_label)),
            ]),
            Line::from(vec![
                Span::styled("• /scan [pattern] [count] ", Style::default().fg(theme.status_warning)),
                Span::styled("Safe scan", Style::default().fg(theme.telemetry_label)),
            ]),
            Line::from(vec![
                Span::styled("• /bigkeys ", Style::default().fg(theme.status_healthy)),
                Span::styled("Profile memory", Style::default().fg(theme.telemetry_label)),
            ]),
            Line::from(vec![
                Span::styled("• F2 / F3 / F4 ", Style::default().fg(theme.conn_cluster)),
                Span::styled("Switch views", Style::default().fg(theme.telemetry_label)),
            ]),
        ];

        f.render_widget(Paragraph::new(lines), inner);
    }

    fn render_compact(f: &mut Frame, area: Rect, data: &TelemetryData, poll_interval_ms: u64, is_paused: bool, theme: &ThemePalette) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border_focused))
            .title(Span::styled(" Telemetry ", Style::default().fg(theme.border_focused).add_modifier(Modifier::BOLD)));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let max_b = data.metrics.max_memory_bytes.unwrap_or(0);
        let used_b = data.metrics.used_memory_bytes.unwrap_or(0);
        let mem_ratio = if max_b > 0 {
            (used_b as f64 / max_b as f64).clamp(0.0, 1.0)
        } else {
            0.53
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("MEM: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{:.1}%", mem_ratio * 100.0), Style::default().fg(theme.border_focused).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({})", data.metrics.used_memory_display()), Style::default().fg(theme.telemetry_value)),
            ]),
            Line::from(vec![
                Span::styled("CPU: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{:.1}%", data.metrics.cpu_usage_pct), Style::default().fg(theme.status_warning).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("OPS: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{}/s", data.metrics.ops_display()), Style::default().fg(theme.status_healthy).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("KEYS: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(crate::core::telemetry::TelemetryParser::format_number(data.metrics.total_keys), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("POLL: ", Style::default().fg(theme.telemetry_label)),
                if is_paused {
                    Span::styled("Paused", Style::default().fg(theme.status_warning).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled(format!("{}ms", poll_interval_ms), Style::default().fg(theme.status_healthy).add_modifier(Modifier::BOLD))
                },
            ]),
        ];

        f.render_widget(Paragraph::new(lines), inner);
    }
}

#[allow(dead_code)]
impl TelemetryData {
    pub fn telemetry_history_max_qps(&self) -> u64 {
        self.history.max_qps()
    }

    pub fn telemetry_history_avg_qps(&self) -> u64 {
        self.history.avg_qps()
    }
}
