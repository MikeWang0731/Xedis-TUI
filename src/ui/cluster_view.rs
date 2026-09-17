use crate::backend::cluster_info::{
    ClusterHealthStatus, ClusterNode, ClusterTopology, NodeHealthState, RedisTopologyMode,
    SlotMigration, SlotMigrationType,
};
use crate::ui::theme::ThemePalette;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct ClusterView;

impl ClusterView {
    pub fn render(f: &mut Frame, area: Rect, topology: &ClusterTopology, scroll_offset: usize, theme: &ThemePalette) {
        let (title_str, border_color) = match topology.mode {
            RedisTopologyMode::Sentinel => (" Sentinel Topology & Quorum Monitoring ", theme.sentinel_border),
            RedisTopologyMode::Cluster => (" Cluster Topology & Shards ", theme.cluster_border),
            RedisTopologyMode::Standalone if topology.is_cluster => (" Cluster Topology & Shards ", theme.cluster_border),
            RedisTopologyMode::Standalone => (" Standalone Topology & Replication ", theme.cluster_border),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                title_str,
                Style::default().fg(border_color).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let max_scroll = match topology.mode {
            RedisTopologyMode::Sentinel => {
                topology.sentinel.as_ref().map_or(0, |s| s.masters.len().saturating_sub(1))
            }
            _ => topology.shards.len().saturating_sub(1),
        };

        let mut items = Vec::new();
        let avail_width = inner.width as usize;
        let content_width = if max_scroll > 0 {
            avail_width.saturating_sub(1)
        } else {
            avail_width
        };

        match topology.mode {
            RedisTopologyMode::Sentinel => {
                Self::build_sentinel_topology_items(&mut items, topology, content_width, theme);
            }
            RedisTopologyMode::Cluster => {
                Self::build_cluster_topology_items(&mut items, topology, content_width, theme);
            }
            RedisTopologyMode::Standalone if topology.is_cluster => {
                Self::build_cluster_topology_items(&mut items, topology, content_width, theme);
            }
            RedisTopologyMode::Standalone => {
                Self::build_standalone_topology_items(&mut items, topology, content_width, theme);
            }
        }

        let total_items = items.len();
        let total_lines: usize = items.iter().map(|item| item.height()).sum();
        let is_overflowing = total_lines > inner.height as usize;
        let has_paging = max_scroll > 0 && (is_overflowing || scroll_offset > 0);

        let visible_items = if scroll_offset < total_items {
            items.into_iter().skip(scroll_offset).collect()
        } else {
            items
        };

        let list = List::new(visible_items);
        f.render_widget(list, inner);

        if has_paging && inner.width >= 2 && inner.height >= 3 {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .begin_style(Style::default().fg(theme.scrollbar_arrow))
                .end_style(Style::default().fg(theme.scrollbar_arrow))
                .track_style(Style::default().fg(theme.scrollbar_track))
                .thumb_style(Style::default().fg(theme.scrollbar_thumb));

            let mut scrollbar_state = ScrollbarState::new(max_scroll + 1)
                .position(scroll_offset.min(max_scroll))
                .viewport_content_length(1);

            f.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
        }
    }

    fn build_cluster_topology_items(items: &mut Vec<ListItem<'static>>, topology: &ClusterTopology, width: usize, theme: &ThemePalette) {
        // 1. Adaptive Summary Header with 3-tier health status (guaranteed never to truncate or overflow)
        let summary_lines = Self::format_cluster_summary_header(topology, width, theme);
        items.push(ListItem::new(summary_lines));

        // Card box width (leave 1 char margin on left/right)
        let box_w = width.saturating_sub(1).max(28);

        // 2. Render each shard in a perfectly bounded rounded card with dedicated node boxes
        for shard in &topology.shards {
            let mut lines = Vec::new();

            // Top Border: ╭── Shard #1 ──────────────────────────────╮
            let shard_title = format!(" ╭── Shard #{} ", shard.shard_index);
            let top_dash_count = box_w.saturating_sub(shard_title.chars().count() + 1);
            lines.push(Line::from(vec![
                Span::styled(shard_title, Style::default().fg(theme.shard_border).add_modifier(Modifier::BOLD)),
                Span::styled("─".repeat(top_dash_count), Style::default().fg(theme.shard_border)),
                Span::styled("╮", Style::default().fg(theme.shard_border)),
            ]));

            // Master Box sizing
            let master_box_w = box_w.saturating_sub(7).max(18);
            let outer_right_spaces = box_w.saturating_sub(4 + master_box_w + 1);
            let master_content_w = master_box_w.saturating_sub(4);

            // Master Top Border: ╭── Master #1 ────────────────────────╮
            let m_full_title = format!("Master #{}", shard.shard_index);
            let (m_prefix, m_label) = if master_box_w >= 4 + m_full_title.chars().count() + 2 {
                ("╭── ", m_full_title)
            } else if master_box_w >= 14 {
                ("╭── ", "Master".to_string())
            } else {
                ("╭─ ", "M".to_string())
            };
            let m_title_len = m_prefix.chars().count() + m_label.chars().count() + 1;
            let m_dash_count = master_box_w.saturating_sub(m_title_len + 1);
            lines.push(Line::from(vec![
                Span::styled(" │  ", Style::default().fg(theme.shard_border)),
                Span::styled(m_prefix, Style::default().fg(theme.shard_border)),
                Span::styled(m_label, Style::default().fg(theme.shard_master_title).add_modifier(Modifier::BOLD)),
                Span::styled(" ", Style::default().fg(theme.shard_border)),
                Span::styled("─".repeat(m_dash_count), Style::default().fg(theme.shard_border)),
                Span::styled("╮", Style::default().fg(theme.shard_border)),
                Span::raw(" ".repeat(outer_right_spaces)),
                Span::styled("│", Style::default().fg(theme.shard_border)),
            ]));

            // Master Info Row(s)
            let master_info_rows = Self::format_node_info(&shard.master, master_content_w, theme);
            for row_spans in master_info_rows {
                Self::push_boxed_line(&mut lines, " │  ", row_spans, master_content_w, outer_right_spaces, theme);
            }

            // Master Slots Row(s)
            let slot_prefix = "Slots: ";
            let raw_ranges = if !shard.slot_ranges.is_empty() {
                let ranges: Vec<String> = shard.slot_ranges.iter().map(|(s, e)| format!("{}-{}", s, e)).collect();
                format!("{} ({} slots)", ranges.join(", "), shard.total_slots)
            } else {
                format!("{} ({} slots)", shard.master.slots_raw, shard.total_slots)
            };

            let max_slot_val_w = master_content_w.saturating_sub(7);
            if raw_ranges.width() <= max_slot_val_w {
                let slot_spans = vec![
                    Span::styled(slot_prefix, Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                    Span::styled(raw_ranges, Style::default().fg(theme.shard_slot_range)),
                ];
                Self::push_boxed_line(&mut lines, " │  ", slot_spans, master_content_w, outer_right_spaces, theme);
            } else {
                let chunks = Self::wrap_slots(&raw_ranges, max_slot_val_w.max(10));
                for (idx, chunk) in chunks.iter().enumerate() {
                    let fit_chunk = Self::fit_str_to_width(chunk, max_slot_val_w);
                    let slot_spans = if idx == 0 {
                        vec![
                            Span::styled(slot_prefix, Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                            Span::styled(fit_chunk, Style::default().fg(theme.shard_slot_range)),
                        ]
                    } else {
                        vec![
                            Span::raw("       "),
                            Span::styled(fit_chunk, Style::default().fg(theme.shard_slot_range)),
                        ]
                    };
                    Self::push_boxed_line(&mut lines, " │  ", slot_spans, master_content_w, outer_right_spaces, theme);
                }
            }

            // Master Slot Migrations (if any)
            for mig in &shard.master.migrations {
                let mig_rows = Self::format_migration_spans(mig, master_content_w, theme);
                for row_spans in mig_rows {
                    Self::push_boxed_line(&mut lines, " │  ", row_spans, master_content_w, outer_right_spaces, theme);
                }
            }

            // Master Replication Offset (if any)
            if let Some(master_rows) = Self::format_master_repl_spans(shard.master.repl_offset, master_content_w, theme) {
                for row_spans in master_rows {
                    Self::push_boxed_line(&mut lines, " │  ", row_spans, master_content_w, outer_right_spaces, theme);
                }
            }

            // Master Bottom Border: ╰──────────────────────────────────╯
            let m_bot_dashes = master_box_w.saturating_sub(2);
            lines.push(Line::from(vec![
                Span::styled(" │  ", Style::default().fg(theme.shard_border)),
                Span::styled("╰", Style::default().fg(theme.shard_border)),
                Span::styled("─".repeat(m_bot_dashes), Style::default().fg(theme.shard_border)),
                Span::styled("╯", Style::default().fg(theme.shard_border)),
                Span::raw(" ".repeat(outer_right_spaces)),
                Span::styled("│", Style::default().fg(theme.shard_border)),
            ]));

            // Replicas Hierarchy & Boxes
            if shard.replicas.is_empty() {
                let mid_spaces = box_w.saturating_sub(6);
                lines.push(Line::from(vec![
                    Span::styled(" │  ", Style::default().fg(theme.shard_border)),
                    Span::styled("│", Style::default().fg(theme.shard_border)),
                    Span::raw(" ".repeat(mid_spaces)),
                    Span::styled("│", Style::default().fg(theme.shard_border)),
                ]));

                let text = "(No replicas configured)";
                let text_w = text.chars().count();
                let text_pad = box_w.saturating_sub(4 + 4 + text_w + 1);
                lines.push(Line::from(vec![
                    Span::styled(" │  ", Style::default().fg(theme.shard_border)),
                    Span::styled("└── ", Style::default().fg(theme.shard_border)),
                    Span::styled(text, Style::default().fg(theme.text_muted)),
                    Span::raw(" ".repeat(text_pad)),
                    Span::styled("│", Style::default().fg(theme.shard_border)),
                ]));
            } else {
                let replica_box_w = master_box_w.saturating_sub(4).max(14);
                let replica_content_w = replica_box_w.saturating_sub(4);

                for (rep_idx, replica) in shard.replicas.iter().enumerate() {
                    let is_last = rep_idx == shard.replicas.len() - 1;
                    let branch_stem = if is_last { "└── " } else { "├── " };
                    let content_stem = if is_last { "    " } else { "│   " };

                    // Vertical connecting line from above
                    let mid_spaces = box_w.saturating_sub(6);
                    lines.push(Line::from(vec![
                        Span::styled(" │  ", Style::default().fg(theme.shard_border)),
                        Span::styled("│", Style::default().fg(theme.shard_border)),
                        Span::raw(" ".repeat(mid_spaces)),
                        Span::styled("│", Style::default().fg(theme.shard_border)),
                    ]));

                    // Replica Top Border
                    let r_full_title = format!("Replica #{}", rep_idx + 1);
                    let (r_prefix, r_label) = if replica_box_w >= 4 + r_full_title.chars().count() + 2 {
                        ("╭── ", r_full_title)
                    } else if replica_box_w >= 15 {
                        ("╭── ", "Replica".to_string())
                    } else {
                        ("╭─ ", "R".to_string())
                    };
                    let r_title_len = r_prefix.chars().count() + r_label.chars().count() + 1;
                    let r_dash_count = replica_box_w.saturating_sub(r_title_len + 1);
                    lines.push(Line::from(vec![
                        Span::styled(" │  ", Style::default().fg(theme.shard_border)),
                        Span::styled(branch_stem, Style::default().fg(theme.shard_border)),
                        Span::styled(r_prefix, Style::default().fg(theme.shard_border)),
                        Span::styled(r_label, Style::default().fg(theme.shard_replica_title).add_modifier(Modifier::BOLD)),
                        Span::styled(" ", Style::default().fg(theme.shard_border)),
                        Span::styled("─".repeat(r_dash_count), Style::default().fg(theme.shard_border)),
                        Span::styled("╮", Style::default().fg(theme.shard_border)),
                        Span::raw(" ".repeat(outer_right_spaces)),
                        Span::styled("│", Style::default().fg(theme.shard_border)),
                    ]));

                    // Replica Content Row(s)
                    let replica_prefix = format!(" │  {}", content_stem);
                    let rep_rows = Self::format_node_info(replica, replica_content_w, theme);
                    for row_spans in rep_rows {
                        Self::push_boxed_line(&mut lines, &replica_prefix, row_spans, replica_content_w, outer_right_spaces, theme);
                    }

                    // Replica Slot Migrations (if any)
                    for mig in &replica.migrations {
                        let mig_rows = Self::format_migration_spans(mig, replica_content_w, theme);
                        for row_spans in mig_rows {
                            Self::push_boxed_line(&mut lines, &replica_prefix, row_spans, replica_content_w, outer_right_spaces, theme);
                        }
                    }

                    // Replica Replication Offset & Lag (if any)
                    if let Some(repl_rows) = Self::format_replica_repl_spans(
                        shard.master.repl_offset,
                        replica.repl_offset,
                        replica_content_w,
                        theme,
                    ) {
                        for row_spans in repl_rows {
                            Self::push_boxed_line(&mut lines, &replica_prefix, row_spans, replica_content_w, outer_right_spaces, theme);
                        }
                    }

                    // Replica Bottom Border
                    let r_bot_dashes = replica_box_w.saturating_sub(2);
                    lines.push(Line::from(vec![
                        Span::styled(" │  ", Style::default().fg(theme.shard_border)),
                        Span::styled(content_stem, Style::default().fg(theme.shard_border)),
                        Span::styled("╰", Style::default().fg(theme.shard_border)),
                        Span::styled("─".repeat(r_bot_dashes), Style::default().fg(theme.shard_border)),
                        Span::styled("╯", Style::default().fg(theme.shard_border)),
                        Span::raw(" ".repeat(outer_right_spaces)),
                        Span::styled("│", Style::default().fg(theme.shard_border)),
                    ]));
                }
            }

            // Bottom Border: ╰──────────────────────────────────────╯
            let bot_dash_count = box_w.saturating_sub(3);
            lines.push(Line::from(vec![
                Span::styled(" ╰", Style::default().fg(theme.shard_border)),
                Span::styled("─".repeat(bot_dash_count), Style::default().fg(theme.shard_border)),
                Span::styled("╯", Style::default().fg(theme.shard_border)),
            ]));

            lines.push(Line::from(""));
            items.push(ListItem::new(lines));
        }
    }

    fn format_cluster_summary_header(
        topology: &ClusterTopology,
        width: usize,
        theme: &ThemePalette,
    ) -> Vec<Line<'static>> {
        let (status_badge, status_color) = match topology.health_status {
            ClusterHealthStatus::Healthy => ("[Cluster: HEALTHY]", theme.status_healthy),
            ClusterHealthStatus::Degraded => ("[Cluster: DEGRADED]", theme.status_warning),
            ClusterHealthStatus::Failed => ("[Cluster: FAILED]", theme.status_critical),
        };

        let cov_text = if topology.is_fully_covered {
            format!("{}/16384 (100%)", topology.covered_slots)
        } else {
            format!("{}/16384 [WARN]", topology.covered_slots)
        };
        let cov_color = if topology.is_fully_covered {
            theme.status_healthy
        } else {
            theme.status_critical
        };

        let total_migrations: usize = topology
            .shards
            .iter()
            .map(|s| s.master.migrations.len() + s.replicas.iter().map(|r| r.migrations.len()).sum::<usize>())
            .sum();

        // Construct clean, non-redundant alert / reason text
        let alert_text = if topology.health_status == ClusterHealthStatus::Healthy {
            "Replication: In Sync · All nodes healthy".to_string()
        } else {
            let mut parts = Vec::new();
            if total_migrations > 0 {
                parts.push(format!("Active Migration: {} slot{}", total_migrations, if total_migrations > 1 { "s" } else { "" }));
            }
            if !topology.health_summary.is_empty() {
                let cleaned_summary = if total_migrations > 0 {
                    topology.health_summary
                        .split(" · ")
                        .filter(|part| !part.contains("migrat"))
                        .collect::<Vec<_>>()
                        .join(" · ")
                } else {
                    topology.health_summary.clone()
                };
                if !cleaned_summary.is_empty() {
                    parts.push(cleaned_summary);
                }
            }
            if parts.is_empty() {
                topology.health_status.badge_label().to_string()
            } else {
                parts.join(" · ")
            }
        };

        // Tier 1: Single Line (Ultra-wide terminal, only when entire line fits within width)
        let single_alert_suffix = if topology.health_status == ClusterHealthStatus::Healthy {
            "· Repl: In Sync".to_string()
        } else {
            format!("· {}", alert_text)
        };

        let single_line_spans = vec![
            Span::styled(format!(" {} ", status_badge), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled(format!("Nodes: {}/{} ", topology.healthy_nodes, topology.total_nodes), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
            Span::styled(format!("· Shards: {} ", topology.shards.len()), Style::default().fg(theme.border_focused)),
            Span::styled(format!("· Slots: {} ", cov_text), Style::default().fg(cov_color).add_modifier(Modifier::BOLD)),
            Span::styled(single_alert_suffix, Style::default().fg(if topology.health_status == ClusterHealthStatus::Healthy { theme.status_healthy } else { status_color }).add_modifier(Modifier::BOLD)),
        ];

        if Self::spans_width(&single_line_spans) <= width {
            return vec![Line::from(single_line_spans), Line::from("")];
        }

        // Tier 2: 2-Line Layout (Standard terminal width ~70-110 cols)
        let l1_spans = vec![
            Span::styled(format!(" {} ", status_badge), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled(format!("Nodes: {}/{} ", topology.healthy_nodes, topology.total_nodes), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
            Span::styled(format!("· Shards: {} ", topology.shards.len()), Style::default().fg(theme.border_focused)),
            Span::styled(format!("· Slots: {}", cov_text), Style::default().fg(cov_color).add_modifier(Modifier::BOLD)),
        ];

        if Self::spans_width(&l1_spans) <= width {
            let (prefix, color) = if topology.health_status == ClusterHealthStatus::Healthy {
                ("  ↳ Status: ", theme.status_healthy)
            } else {
                ("  ↳ Alerts: ", status_color)
            };
            let max_alert_w = width.saturating_sub(prefix.len() + 1);
            let fit_alert = Self::fit_str_to_width(&alert_text, max_alert_w);
            let l2_spans = vec![
                Span::styled(prefix, Style::default().fg(color)),
                Span::styled(fit_alert, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ];
            return vec![Line::from(l1_spans), Line::from(l2_spans), Line::from("")];
        }

        // Tier 3: Medium layout (48 ~ 70 cols)
        if width >= 48 {
            let line1 = Line::from(vec![
                Span::styled(format!(" {} ", status_badge), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("Nodes: {}/{} ", topology.healthy_nodes, topology.total_nodes), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled(format!("· Shards: {}", topology.shards.len()), Style::default().fg(theme.border_focused)),
            ]);
            let line2 = Line::from(vec![
                Span::styled("  Slots: ", Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                Span::styled(cov_text, Style::default().fg(cov_color).add_modifier(Modifier::BOLD)),
            ]);
            let prefix = "  ↳ ";
            let max_alert_w = width.saturating_sub(prefix.len() + 1);
            let fit_alert = Self::fit_str_to_width(&alert_text, max_alert_w);
            let line3 = Line::from(vec![
                Span::styled(prefix, Style::default().fg(status_color)),
                Span::styled(fit_alert, Style::default().fg(if topology.health_status == ClusterHealthStatus::Healthy { theme.status_healthy } else { status_color }).add_modifier(Modifier::BOLD)),
            ]);
            return vec![line1, line2, line3, Line::from("")];
        }

        // Tier 4: Compact layout (< 48 cols)
        let l1_badge = format!("[{}]", topology.health_status.short_label());
        let l1_nodes = format!("{}/{} Nodes", topology.healthy_nodes, topology.total_nodes);
        let l1_str = format!("{} {}", l1_badge, l1_nodes);
        let fit_l1 = Self::fit_str_to_width(&l1_str, width.saturating_sub(2));
        let line1 = Line::from(vec![
            Span::raw(" "),
            Span::styled(fit_l1, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]);

        let l2_str = format!("{} Shards · Slots: {}", topology.shards.len(), if topology.is_fully_covered { "100% OK" } else { "CRITICAL" });
        let fit_l2 = Self::fit_str_to_width(&l2_str, width.saturating_sub(3));
        let line2 = Line::from(vec![
            Span::raw("  "),
            Span::styled(fit_l2, Style::default().fg(cov_color).add_modifier(Modifier::BOLD)),
        ]);

        let max_alert_w = width.saturating_sub(3);
        let fit_alert = Self::fit_str_to_width(&alert_text, max_alert_w);
        let line3 = Line::from(vec![
            Span::raw("  "),
            Span::styled(fit_alert, Style::default().fg(status_color)),
        ]);
        vec![line1, line2, line3, Line::from("")]
    }

    fn fit_str_to_width(s: &str, max_w: usize) -> String {
        let sw = s.width();
        if sw <= max_w {
            return s.to_string();
        }
        if max_w == 0 {
            return String::new();
        }
        if max_w == 1 {
            return "…".to_string();
        }

        let target_w = max_w - 1; // 1 column for '…'
        let mut cur_w = 0;
        let mut res = String::new();
        for c in s.chars() {
            let cw = c.width().unwrap_or(0);
            if cur_w + cw > target_w {
                break;
            }
            res.push(c);
            cur_w += cw;
        }
        res.push('…');
        res
    }

    fn spans_width(spans: &[Span]) -> usize {
        spans.iter().map(|s| s.content.as_ref().width()).sum()
    }

    fn push_boxed_line(
        lines: &mut Vec<Line<'static>>,
        left_decor: &str,
        spans: Vec<Span<'static>>,
        content_w: usize,
        outer_right_spaces: usize,
        theme: &ThemePalette,
    ) {
        let sw = Self::spans_width(&spans);
        let pad_len = content_w.saturating_sub(sw);
        let mut line_spans = Vec::with_capacity(spans.len() + 6);
        line_spans.push(Span::styled(left_decor.to_string(), Style::default().fg(theme.shard_border)));
        line_spans.push(Span::styled("│ ", Style::default().fg(theme.shard_border)));
        line_spans.extend(spans);
        if pad_len > 0 {
            line_spans.push(Span::raw(" ".repeat(pad_len)));
        }
        line_spans.push(Span::styled(" │", Style::default().fg(theme.shard_border)));
        if outer_right_spaces > 0 {
            line_spans.push(Span::raw(" ".repeat(outer_right_spaces)));
        }
        line_spans.push(Span::styled("│", Style::default().fg(theme.shard_border)));
        lines.push(Line::from(line_spans));
    }

    fn format_node_info(
        node: &ClusterNode,
        content_w: usize,
        theme: &ThemePalette,
    ) -> Vec<Vec<Span<'static>>> {
        let (status_str, status_color) = match node.health_state {
            NodeHealthState::Healthy => ("[HEALTHY]", theme.status_healthy),
            NodeHealthState::Pfail => ("[PFAIL]", theme.status_warning),
            NodeHealthState::Fail => ("[FAIL]", theme.status_critical),
        };

        let id_str = format!("@{}", node.id);
        let ping_str = if node.health_state == NodeHealthState::Healthy {
            format!("Ping: {:.1}ms", node.ping_ms)
        } else {
            "Ping: --".to_string()
        };
        let short_ping = if node.health_state == NodeHealthState::Healthy {
            format!("{:.1}ms", node.ping_ms)
        } else {
            "--".to_string()
        };

        // Single line width: "@id address [HEALTHY] Ping: 3.0ms"
        let single_line_w = id_str.width() + 1 + node.address.width() + 1 + status_str.len() + 1 + ping_str.len();

        if content_w >= single_line_w {
            vec![vec![
                Span::styled(id_str, Style::default().fg(theme.shard_node_id)),
                Span::raw(" "),
                Span::styled(node.address.clone(), Style::default().fg(theme.text_primary)),
                Span::raw(" "),
                Span::styled(status_str, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(ping_str, Style::default().fg(theme.text_muted)),
            ]]
        } else {
            let line1_w = id_str.width() + 1 + status_str.len() + 1 + short_ping.len();
            if content_w >= line1_w.max(20) {
                // Tier 2: Medium layout (2 rows)
                let ping_display = if id_str.width() + 1 + status_str.len() + 1 + ping_str.len() <= content_w {
                    ping_str
                } else {
                    short_ping
                };
                let row1 = vec![
                    Span::styled(id_str, Style::default().fg(theme.shard_node_id)),
                    Span::raw(" "),
                    Span::styled(status_str, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(ping_display, Style::default().fg(theme.text_muted)),
                ];
                let row2 = vec![
                    Span::styled(Self::fit_str_to_width(&node.address, content_w), Style::default().fg(theme.text_primary)),
                ];
                vec![row1, row2]
            } else {
                // Tier 3: Compact layout (3 rows)
                let status_label = if content_w < 18 && node.health_state == NodeHealthState::Healthy {
                    "[OK]"
                } else {
                    status_str
                };
                let id_display = Self::fit_str_to_width(&id_str, content_w.saturating_sub(status_label.len() + 1));
                let row1 = vec![
                    Span::styled(id_display, Style::default().fg(theme.shard_node_id)),
                    Span::raw(" "),
                    Span::styled(status_label, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                ];
                let row2 = vec![
                    Span::styled(Self::fit_str_to_width(&node.address, content_w), Style::default().fg(theme.text_primary)),
                ];
                let row3 = vec![
                    Span::styled(Self::fit_str_to_width(&ping_str, content_w), Style::default().fg(theme.text_muted)),
                ];
                vec![row1, row2, row3]
            }
        }
    }

    fn format_number_with_commas(n: u64) -> String {
        let s = n.to_string();
        let mut result = String::new();
        let len = s.len();
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        result
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    fn format_migration_spans(
        migration: &SlotMigration,
        max_w: usize,
        theme: &ThemePalette,
    ) -> Vec<Vec<Span<'static>>> {
        let (icon, prefix, color) = match migration.migration_type {
            SlotMigrationType::Migrating => ("⇄ ", "MIGRATING ", theme.status_warning),
            SlotMigrationType::Importing => ("⇆ ", "IMPORTING ", theme.shard_master_title),
        };
        let arrow = match migration.migration_type {
            SlotMigrationType::Migrating => "->",
            SlotMigrationType::Importing => "<-",
        };
        let full_text = format!("Slot {} {} {}", migration.slot, arrow, migration.remote_node_repr);
        let single_line_w = icon.width() + prefix.len() + full_text.width();

        if single_line_w <= max_w {
            vec![vec![
                Span::styled(icon, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(full_text, Style::default().fg(theme.text_primary)),
            ]]
        } else {
            let line1 = vec![
                Span::styled(icon, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("Slot {}", migration.slot), Style::default().fg(theme.text_primary)),
            ];
            let arrow_target = format!("   {} {}", arrow, migration.remote_node_repr);
            let fit_target = Self::fit_str_to_width(&arrow_target, max_w);
            let line2 = vec![
                Span::styled(fit_target, Style::default().fg(theme.text_secondary)),
            ];
            vec![line1, line2]
        }
    }

    fn format_master_repl_spans(
        master_offset: Option<u64>,
        max_w: usize,
        theme: &ThemePalette,
    ) -> Option<Vec<Vec<Span<'static>>>> {
        let m_offset = master_offset?;
        let offset_str = format!("{} (Master)", Self::format_number_with_commas(m_offset));
        let full_text = format!("Repl Offset: {}", offset_str);
        if full_text.width() <= max_w {
            Some(vec![vec![
                Span::styled("Repl Offset: ", Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                Span::styled(offset_str, Style::default().fg(theme.text_secondary)),
            ]])
        } else {
            Some(vec![vec![
                Span::styled("Repl: ", Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                Span::styled(Self::fit_str_to_width(&offset_str, max_w.saturating_sub(6)), Style::default().fg(theme.text_secondary)),
            ]])
        }
    }

    fn format_replica_repl_spans(
        master_offset: Option<u64>,
        replica_offset: Option<u64>,
        max_w: usize,
        theme: &ThemePalette,
    ) -> Option<Vec<Vec<Span<'static>>>> {
        let rep_offset = replica_offset?;
        let offset_str = format!("Repl Offset: {}", Self::format_number_with_commas(rep_offset));

        if let Some(m_offset) = master_offset {
            let lag_bytes = m_offset.saturating_sub(rep_offset);
            let (lag_str, lag_color) = if lag_bytes == 0 {
                ("Lag: 0 B (In Sync)".to_string(), theme.status_healthy)
            } else {
                (format!("Lag: {} (behind)", Self::format_bytes(lag_bytes)), theme.status_warning)
            };

            let single_line_w = offset_str.width() + 3 + lag_str.width();
            if single_line_w <= max_w {
                Some(vec![vec![
                    Span::styled("Repl Offset: ", Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                    Span::styled(Self::format_number_with_commas(rep_offset), Style::default().fg(theme.text_secondary)),
                    Span::raw(" · "),
                    Span::styled(lag_str, Style::default().fg(lag_color).add_modifier(Modifier::BOLD)),
                ]])
            } else {
                Some(vec![
                    vec![
                        Span::styled("Repl Offset: ", Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                        Span::styled(Self::format_number_with_commas(rep_offset), Style::default().fg(theme.text_secondary)),
                    ],
                    vec![
                        Span::styled(Self::fit_str_to_width(&format!("   {}", lag_str), max_w), Style::default().fg(lag_color).add_modifier(Modifier::BOLD)),
                    ],
                ])
            }
        } else {
            Some(vec![vec![
                Span::styled("Repl Offset: ", Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                Span::styled(Self::format_number_with_commas(rep_offset), Style::default().fg(theme.text_secondary)),
            ]])
        }
    }

    fn wrap_slots(text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for part in text.split(' ') {
            if current_line.len() + part.len() + 1 > max_width && !current_line.is_empty() {
                lines.push(current_line);
                current_line = part.to_string();
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(part);
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        lines
    }

    fn build_standalone_topology_items(items: &mut Vec<ListItem<'static>>, topology: &ClusterTopology, width: usize, theme: &ThemePalette) {
        let repl_info = topology.replication.clone().unwrap_or_default();

        let role_badge = if repl_info.role == "master" {
            Span::styled("Role: Master (Standalone)", Style::default().fg(theme.status_healthy).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("Role: Replica / Slave", Style::default().fg(theme.shard_replica_title).add_modifier(Modifier::BOLD))
        };

        let summary_spans = vec![
            Span::styled(" [Standalone Instance: ", Style::default().fg(theme.text_muted)),
            role_badge.clone(),
            Span::styled(format!(" · Connected Slaves: {}]", repl_info.connected_slaves), Style::default().fg(theme.border_focused)),
        ];

        if Self::spans_width(&summary_spans) <= width {
            items.push(ListItem::new(vec![Line::from(summary_spans), Line::from("")]));
        } else {
            let line1 = Line::from(vec![
                Span::styled(" [Standalone: ", Style::default().fg(theme.text_muted)),
                role_badge,
                Span::styled("]", Style::default().fg(theme.text_muted)),
            ]);
            let line2 = Line::from(vec![
                Span::styled(format!(" [Connected Slaves: {}]", repl_info.connected_slaves), Style::default().fg(theme.border_focused)),
            ]);
            items.push(ListItem::new(vec![line1, line2, Line::from("")]));
        }

        let box_w = width.saturating_sub(1).max(24);
        let mut lines = Vec::new();

        // Node card top border
        let title = " ╭── Node Instance ";
        let dash_count = box_w.saturating_sub(title.chars().count() + 1);
        lines.push(Line::from(vec![
            Span::styled(title, Style::default().fg(theme.cluster_border).add_modifier(Modifier::BOLD)),
            Span::styled("─".repeat(dash_count), Style::default().fg(theme.cluster_border)),
            Span::styled("╮", Style::default().fg(theme.cluster_border)),
        ]));

        if let Some(node) = topology.standalone_nodes.first() {
            if width >= 55 {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.cluster_border)),
                    Span::styled("  Endpoint: ", Style::default().fg(theme.text_muted)),
                    Span::styled(node.address.clone(), Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" · Status: [HEALTHY] · Ping: {:.2}ms", node.ping_ms), Style::default().fg(theme.status_healthy)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.cluster_border)),
                    Span::styled("  Endpoint: ", Style::default().fg(theme.text_muted)),
                    Span::styled(node.address.clone(), Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.cluster_border)),
                    Span::styled("   ", Style::default().fg(theme.cluster_border)),
                    Span::styled(format!("Status: [HEALTHY] · Ping: {:.2}ms", node.ping_ms), Style::default().fg(theme.status_healthy)),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled(" │", Style::default().fg(theme.cluster_border)),
                Span::styled("  Keyspace: ", Style::default().fg(theme.text_muted)),
                Span::styled("Databases 0~15 (Standalone Store)", Style::default().fg(theme.status_warning)),
            ]));
        }

        // Replication section
        if repl_info.role == "master" {
            if width >= 55 {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.cluster_border)),
                    Span::styled("  Replication Offset: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{}", repl_info.master_repl_offset), Style::default().fg(theme.border_focused)),
                    Span::styled(format!(" · Connected Replicas: {}", repl_info.connected_slaves), Style::default().fg(theme.text_muted)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.cluster_border)),
                    Span::styled("  Repl Offset: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{}", repl_info.master_repl_offset), Style::default().fg(theme.border_focused)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.cluster_border)),
                    Span::styled("  Connected Replicas: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{}", repl_info.connected_slaves), Style::default().fg(theme.text_primary)),
                ]));
            }

            if repl_info.slaves.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.cluster_border)),
                    Span::styled("   └── ", Style::default().fg(theme.cluster_border)),
                    Span::styled("(No slave instances currently connected)", Style::default().fg(theme.text_muted)),
                ]));
            } else {
                let slaves_len = repl_info.slaves.len();
                for (i, slave) in repl_info.slaves.iter().enumerate() {
                    let is_last = i + 1 == slaves_len;
                    let branch = if is_last { "   └── " } else { "   ├── " };
                    if width >= 60 {
                        lines.push(Line::from(vec![
                            Span::styled(" │", Style::default().fg(theme.cluster_border)),
                            Span::styled(branch, Style::default().fg(theme.cluster_border)),
                            Span::styled(format!("[Slave #{}] {}:{} ", i + 1, slave.ip, slave.port), Style::default().fg(theme.shard_replica_title)),
                            Span::styled(format!("State: {} · Offset: {} · Lag: {}s", slave.state, slave.offset, slave.lag), Style::default().fg(theme.text_muted)),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(" │", Style::default().fg(theme.cluster_border)),
                            Span::styled(branch, Style::default().fg(theme.cluster_border)),
                            Span::styled(format!("[Slave #{}] {}:{}", i + 1, slave.ip, slave.port), Style::default().fg(theme.shard_replica_title)),
                        ]));
                        let pad = if is_last { "       " } else { "   │   " };
                        lines.push(Line::from(vec![
                            Span::styled(" │", Style::default().fg(theme.cluster_border)),
                            Span::styled(pad, Style::default().fg(theme.cluster_border)),
                            Span::styled(format!("State: {} · Offset: {} · Lag: {}s", slave.state, slave.offset, slave.lag), Style::default().fg(theme.text_muted)),
                        ]));
                    }
                }
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled(" │", Style::default().fg(theme.cluster_border)),
                Span::styled("  Master Link: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    format!("{}:{} ({})", repl_info.master_host.as_deref().unwrap_or("-"), repl_info.master_port.unwrap_or(0), repl_info.master_link_status.as_deref().unwrap_or("unknown")),
                    Style::default().fg(theme.status_healthy),
                ),
            ]));
        }

        // Bottom border
        let bot_dash_count = box_w.saturating_sub(3);
        lines.push(Line::from(vec![
            Span::styled(" ╰", Style::default().fg(theme.cluster_border)),
            Span::styled("─".repeat(bot_dash_count), Style::default().fg(theme.cluster_border)),
            Span::styled("╯", Style::default().fg(theme.cluster_border)),
        ]));

        items.push(ListItem::new(lines));
    }

    fn build_sentinel_topology_items(items: &mut Vec<ListItem<'static>>, topology: &ClusterTopology, width: usize, theme: &ThemePalette) {
        let fallback_sentinel;
        let sentinel = match &topology.sentinel {
            Some(s) => s,
            None => {
                fallback_sentinel = ClusterTopology::mock_sentinel_topology().sentinel.unwrap();
                &fallback_sentinel
            }
        };

        let total_masters = sentinel.masters.len();
        let total_sentinels = sentinel.total_sentinels;

        // Determine Quorum Health across all monitored masters
        let mut all_quorum_ok = true;
        for m in &sentinel.masters {
            let active_sentinels = m.sentinels.len() + 1;
            if (active_sentinels as u32) < m.quorum {
                all_quorum_ok = false;
                break;
            }
        }

        let (q_status_text, q_status_color) = if all_quorum_ok {
            ("Quorum: OK", theme.status_healthy)
        } else {
            ("Quorum: DEGRADED", theme.status_warning)
        };

        // 1. Adaptive Summary Header
        let mut summary_spans = vec![
            Span::styled(" [Sentinel Topology: ", Style::default().fg(theme.telemetry_label)),
            Span::styled(format!("Masters: {} ", total_masters), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
            Span::styled(format!("· Sentinels: {} ", total_sentinels), Style::default().fg(theme.border_focused)),
            Span::styled(format!("· {}]", q_status_text), Style::default().fg(q_status_color).add_modifier(Modifier::BOLD)),
        ];
        if sentinel.tilt_mode {
            summary_spans.push(Span::styled(" · [TILT ACTIVE]", Style::default().fg(theme.status_critical).add_modifier(Modifier::BOLD)));
        }

        if Self::spans_width(&summary_spans) <= width {
            items.push(ListItem::new(vec![Line::from(summary_spans), Line::from("")]));
        } else if width >= 48 {
            let line1 = Line::from(vec![
                Span::styled(" [Sentinel: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{} Masters · {} Sentinels]", total_masters, total_sentinels), Style::default().fg(theme.border_focused)),
            ]);
            let line2 = Line::from(vec![
                Span::styled(" [Status: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(q_status_text, Style::default().fg(q_status_color).add_modifier(Modifier::BOLD)),
                Span::styled("]", Style::default().fg(theme.telemetry_label)),
            ]);
            items.push(ListItem::new(vec![line1, line2, Line::from("")]));
        } else {
            let line1 = Line::from(vec![
                Span::styled(format!(" [Snt: {}M/{}S]", total_masters, total_sentinels), Style::default().fg(theme.border_focused)),
            ]);
            let line2 = Line::from(vec![
                Span::styled(format!(" [{}]", if all_quorum_ok { "Quorum OK" } else { "WARN" }), Style::default().fg(q_status_color).add_modifier(Modifier::BOLD)),
            ]);
            items.push(ListItem::new(vec![line1, line2, Line::from("")]));
        }

        let box_w = width.saturating_sub(1).max(24);

        // 2. Render each Monitored Master Card
        for master in &sentinel.masters {
            let mut lines = Vec::new();

            let master_status_color = if master.status == "ok" {
                theme.status_healthy
            } else if master.status == "sdown" {
                theme.status_warning
            } else {
                theme.status_critical
            };

            let status_tag = format!("[{}]", master.status.to_uppercase());

            // Card top border
            let title = format!(" ╭── Master: {} ", master.name);
            let dash_count = box_w.saturating_sub(title.chars().count() + 1);
            lines.push(Line::from(vec![
                Span::styled(title, Style::default().fg(theme.sentinel_border).add_modifier(Modifier::BOLD)),
                Span::styled("─".repeat(dash_count), Style::default().fg(theme.sentinel_border)),
                Span::styled("╮", Style::default().fg(theme.sentinel_border)),
            ]));

            // Master Endpoint & Status
            if width >= 70 {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                    Span::styled("  Endpoint: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{}:{} ", master.ip, master.port), Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("· Status: {} ", status_tag), Style::default().fg(master_status_color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("· Quorum: {} ({} in total)", master.quorum, master.sentinels.len() + 1), Style::default().fg(theme.shard_slot_label)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                    Span::styled("  Endpoint: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{}:{}", master.ip, master.port), Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                    Span::styled("   ", Style::default().fg(theme.sentinel_border)),
                    Span::styled(format!("Status: {} · Quorum: {} ({} in total)", status_tag, master.quorum, master.sentinels.len() + 1), Style::default().fg(master_status_color)),
                ]));
            }

            // Down-after & Failover Timeout
            lines.push(Line::from(vec![
                Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                Span::styled("  Monitoring: ", Style::default().fg(theme.text_muted)),
                Span::styled(format!("Down-After: {}s · Failover-Timeout: {}s", master.down_after_ms / 1000, master.failover_timeout_ms / 1000), Style::default().fg(theme.text_secondary)),
            ]));

            // Slaves / Replicas Section
            lines.push(Line::from(vec![
                Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                Span::styled("  Connected Replicas: ", Style::default().fg(theme.text_muted)),
                Span::styled(format!("{}", master.slaves.len()), Style::default().fg(theme.border_focused).add_modifier(Modifier::BOLD)),
            ]));

            if master.slaves.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                    Span::styled("   └── ", Style::default().fg(theme.sentinel_border)),
                    Span::styled("(No active replicas registered)", Style::default().fg(theme.text_muted)),
                ]));
            } else {
                let slaves_len = master.slaves.len();
                for (i, slave) in master.slaves.iter().enumerate() {
                    let is_last = i + 1 == slaves_len;
                    let branch = if is_last { "   └── " } else { "   ├── " };
                    let link_color = if slave.link_status == "ok" { theme.status_healthy } else { theme.status_critical };

                    if width >= 65 {
                        lines.push(Line::from(vec![
                            Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                            Span::styled(branch, Style::default().fg(theme.sentinel_border)),
                            Span::styled(format!("[Slave #{}] {}:{} ", i + 1, slave.ip, slave.port), Style::default().fg(theme.shard_replica_title)),
                            Span::styled(format!("Link: [{}] · Offset: {} · Lag: {}s", slave.link_status.to_uppercase(), slave.repl_offset, slave.lag_sec), Style::default().fg(link_color)),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                            Span::styled(branch, Style::default().fg(theme.sentinel_border)),
                            Span::styled(format!("[Slave #{}] {}:{}", i + 1, slave.ip, slave.port), Style::default().fg(theme.shard_replica_title)),
                        ]));
                        let pad = if is_last { "       " } else { "   │   " };
                        lines.push(Line::from(vec![
                            Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                            Span::styled(pad, Style::default().fg(theme.sentinel_border)),
                            Span::styled(format!("Link: [{}] · Offset: {} · Lag: {}s", slave.link_status.to_uppercase(), slave.repl_offset, slave.lag_sec), Style::default().fg(link_color)),
                        ]));
                    }
                }
            }

            // Quorum Sentinels Section (Myself + Peers)
            let peers_len = master.sentinels.len();
            let total_watchers = peers_len + 1;
            let watchers_summary = if peers_len == 0 {
                "Myself Only".to_string()
            } else if peers_len == 1 {
                "1 Peer + Myself".to_string()
            } else {
                format!("{} Peers + Myself", peers_len)
            };

            lines.push(Line::from(vec![
                Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                Span::styled("  Quorum Sentinels: ", Style::default().fg(theme.text_muted)),
                Span::styled(format!("{} ({})", total_watchers, watchers_summary), Style::default().fg(theme.sentinel_peer_title).add_modifier(Modifier::BOLD)),
            ]));

            let my_info = sentinel.my_sentinel.as_ref();
            let my_ip = my_info
                .map(|s| s.ip.as_str())
                .or_else(|| topology.standalone_nodes.first().and_then(|n| n.address.split(':').next()))
                .unwrap_or("127.0.0.1");
            let my_port = my_info
                .map(|s| s.port)
                .or_else(|| topology.standalone_nodes.first().and_then(|n| n.address.split(':').nth(1).and_then(|p| p.parse().ok())))
                .unwrap_or(26379);
            let my_ping = my_info
                .map(|s| s.last_ok_ping_ms as f64)
                .or_else(|| topology.standalone_nodes.first().map(|n| n.ping_ms))
                .unwrap_or(0.4);

            // [Myself] Node Row
            let my_branch = if peers_len == 0 { "   └── " } else { "   ├── " };
            if width >= 65 {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                    Span::styled(my_branch, Style::default().fg(theme.sentinel_border)),
                    Span::styled(format!("[Myself] {}:{} ", my_ip, my_port), Style::default().fg(theme.sentinel_myself_title)),
                    Span::styled("Status: [ACTIVE] ", Style::default().fg(theme.status_healthy)),
                    Span::styled(format!("· Ping: {:.1}ms · Local Node", my_ping), Style::default().fg(theme.text_muted)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                    Span::styled(my_branch, Style::default().fg(theme.sentinel_border)),
                    Span::styled(format!("[Myself] {}:{}", my_ip, my_port), Style::default().fg(theme.sentinel_myself_title)),
                ]));
                let pad = if peers_len == 0 { "       " } else { "   │   " };
                lines.push(Line::from(vec![
                    Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                    Span::styled(pad, Style::default().fg(theme.sentinel_border)),
                    Span::styled(format!("Status: [ACTIVE] · Ping: {:.1}ms · Local Node", my_ping), Style::default().fg(theme.text_muted)),
                ]));
            }

            if peers_len == 0 {
                // If there are no peers, [Myself] is the only node and has already used └──
            } else {
                for (i, peer) in master.sentinels.iter().enumerate() {
                    let is_last = i + 1 == peers_len;
                    let branch = if is_last { "   └── " } else { "   ├── " };
                    let peer_color = if peer.is_healthy { theme.status_healthy } else { theme.status_critical };
                    let peer_status = if peer.is_healthy { "[HEALTHY]" } else { "[SDOWN]" };

                    if width >= 65 {
                        lines.push(Line::from(vec![
                            Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                            Span::styled(branch, Style::default().fg(theme.sentinel_border)),
                            Span::styled(format!("[Peer #{}] {}:{} ", i + 1, peer.ip, peer.port), Style::default().fg(theme.sentinel_peer_title)),
                            Span::styled(format!("Status: {} · Ping: {}ms", peer_status, peer.last_ok_ping_ms), Style::default().fg(peer_color)),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                            Span::styled(branch, Style::default().fg(theme.sentinel_border)),
                            Span::styled(format!("[Peer #{}] {}:{}", i + 1, peer.ip, peer.port), Style::default().fg(theme.sentinel_peer_title)),
                        ]));
                        let pad = if is_last { "       " } else { "   │   " };
                        lines.push(Line::from(vec![
                            Span::styled(" │", Style::default().fg(theme.sentinel_border)),
                            Span::styled(pad, Style::default().fg(theme.sentinel_border)),
                            Span::styled(format!("Status: {} · Ping: {}ms", peer_status, peer.last_ok_ping_ms), Style::default().fg(peer_color)),
                        ]));
                    }
                }
            }

            // Card bottom border
            let bot_dash_count = box_w.saturating_sub(3);
            lines.push(Line::from(vec![
                Span::styled(" ╰", Style::default().fg(theme.sentinel_border)),
                Span::styled("─".repeat(bot_dash_count), Style::default().fg(theme.sentinel_border)),
                Span::styled("╯", Style::default().fg(theme.sentinel_border)),
            ]));

            items.push(ListItem::new(lines));
            items.push(ListItem::new(vec![Line::from("")])); // Spacing between multiple masters
        }
    }
}
