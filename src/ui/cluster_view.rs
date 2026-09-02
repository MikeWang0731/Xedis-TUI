use crate::backend::cluster_info::ClusterTopology;
use crate::ui::theme::ThemePalette;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

pub struct ClusterView;

impl ClusterView {
    pub fn render(f: &mut Frame, area: Rect, topology: &ClusterTopology, scroll_offset: usize, theme: &ThemePalette) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.cluster_border))
            .title(Span::styled(
                " Cluster Topology & Shards ",
                Style::default().fg(theme.cluster_border).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mut items = Vec::new();
        let avail_width = inner.width as usize;

        if topology.is_cluster {
            Self::build_cluster_topology_items(&mut items, topology, avail_width, theme);
        } else {
            Self::build_standalone_topology_items(&mut items, topology, avail_width, theme);
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

    fn build_cluster_topology_items(items: &mut Vec<ListItem<'static>>, topology: &ClusterTopology, width: usize, theme: &ThemePalette) {
        // 1. Adaptive Summary Header
        let (cov_text, cov_color) = if topology.is_fully_covered {
            (format!("{}/16384 (100%) [OK]", topology.covered_slots), theme.status_healthy)
        } else {
            (format!("{}/16384 [WARN]", topology.covered_slots), theme.status_warning)
        };

        if width >= 78 {
            // Wide layout (Single line)
            let summary_line = Line::from(vec![
                Span::styled(" [Topology: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("Nodes: {} ", topology.total_nodes), Style::default().fg(theme.telemetry_value).add_modifier(Modifier::BOLD)),
                Span::styled(format!("· Healthy: {} ", topology.healthy_nodes), Style::default().fg(theme.status_healthy)),
                Span::styled(format!("· Shards: {} ", topology.shards.len()), Style::default().fg(theme.border_focused)),
                Span::styled(format!("· Coverage: {}]", cov_text), Style::default().fg(cov_color).add_modifier(Modifier::BOLD)),
            ]);
            items.push(ListItem::new(vec![summary_line, Line::from("")]));
        } else if width >= 48 {
            // Medium layout (2 lines)
            let line1 = Line::from(vec![
                Span::styled(" [Nodes: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{}/{} Healthy", topology.healthy_nodes, topology.total_nodes), Style::default().fg(theme.status_healthy).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" · Shards: {}]", topology.shards.len()), Style::default().fg(theme.border_focused)),
            ]);
            let line2 = Line::from(vec![
                Span::styled(" [Coverage: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(cov_text, Style::default().fg(cov_color).add_modifier(Modifier::BOLD)),
                Span::styled("]", Style::default().fg(theme.telemetry_label)),
            ]);
            items.push(ListItem::new(vec![line1, line2, Line::from("")]));
        } else {
            // Compact layout (2 short lines)
            let line1 = Line::from(vec![
                Span::styled(" [Nodes: ", Style::default().fg(theme.telemetry_label)),
                Span::styled(format!("{}/{}", topology.healthy_nodes, topology.total_nodes), Style::default().fg(theme.status_healthy).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" · Shards: {}]", topology.shards.len()), Style::default().fg(theme.border_focused)),
            ]);
            let line2 = Line::from(vec![
                Span::styled(format!(" [Cov: {}]", if topology.is_fully_covered { "100% OK" } else { "WARN" }), Style::default().fg(cov_color).add_modifier(Modifier::BOLD)),
            ]);
            items.push(ListItem::new(vec![line1, line2, Line::from("")]));
        }

        // Card box width (leave 1 char margin on left/right)
        let box_w = width.saturating_sub(1).max(24);

        // 2. Render each shard in a perfectly bounded rounded card
        for shard in &topology.shards {
            let mut lines = Vec::new();

            let master_status_color = if shard.master.is_healthy { theme.status_healthy } else { theme.status_critical };
            let master_status_str = if shard.master.is_healthy { "[HEALTHY]" } else { "[FAIL]" };

            // Top Border: ╭── Shard #1 ──────────────────────────────╮
            let shard_title = format!(" ╭── Shard #{} ", shard.shard_index);
            let top_dash_count = box_w.saturating_sub(shard_title.chars().count() + 1);
            lines.push(Line::from(vec![
                Span::styled(shard_title, Style::default().fg(theme.shard_border).add_modifier(Modifier::BOLD)),
                Span::styled("─".repeat(top_dash_count), Style::default().fg(theme.shard_border)),
                Span::styled("╮", Style::default().fg(theme.shard_border)),
            ]));

            // Master Info Row(s)
            if width >= 65 {
                // Tier 1: Wide layout (Single row)
                lines.push(Line::from(vec![
                    Span::styled(" │ ", Style::default().fg(theme.shard_border)),
                    Span::styled("[Master] ", Style::default().fg(theme.shard_master_title).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("@{} ", shard.master.id), Style::default().fg(theme.shard_node_id)),
                    Span::styled(format!("{} ", shard.master.address), Style::default().fg(theme.text_primary)),
                    Span::styled(format!("{} ", master_status_str), Style::default().fg(master_status_color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("Ping: {:.1}ms", shard.master.ping_ms), Style::default().fg(theme.text_muted)),
                ]));
            } else if width >= 42 {
                // Tier 2: Medium layout (Stacked 2 rows)
                lines.push(Line::from(vec![
                    Span::styled(" │ ", Style::default().fg(theme.shard_border)),
                    Span::styled("[Master] ", Style::default().fg(theme.shard_master_title).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("@{} ", shard.master.id), Style::default().fg(theme.shard_node_id)),
                    Span::styled(format!("{}", master_status_str), Style::default().fg(master_status_color).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" │   ", Style::default().fg(theme.shard_border)),
                    Span::styled(format!("{} · Ping: {:.1}ms", shard.master.address, shard.master.ping_ms), Style::default().fg(theme.text_primary)),
                ]));
            } else {
                // Tier 3: Ultra-compact layout (Stacked 3 rows)
                lines.push(Line::from(vec![
                    Span::styled(" │ ", Style::default().fg(theme.shard_border)),
                    Span::styled("[Master] ", Style::default().fg(theme.shard_master_title).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("@{}", shard.master.id), Style::default().fg(theme.shard_node_id)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" │   ", Style::default().fg(theme.shard_border)),
                    Span::styled(shard.master.address.clone(), Style::default().fg(theme.text_primary)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" │   ", Style::default().fg(theme.shard_border)),
                    Span::styled(format!("{} · {:.1}ms", master_status_str, shard.master.ping_ms), Style::default().fg(master_status_color)),
                ]));
            }

            // Slots Info (with multiline wrapping)
            let slot_prefix = "Slots: ";
            let raw_ranges = if !shard.slot_ranges.is_empty() {
                let ranges: Vec<String> = shard.slot_ranges.iter().map(|(s, e)| format!("{}-{}", s, e)).collect();
                format!("{} ({} slots)", ranges.join(", "), shard.total_slots)
            } else {
                format!("{} ({} slots)", shard.master.slots_raw, shard.total_slots)
            };

            let max_slot_w = box_w.saturating_sub(12).max(16);
            if raw_ranges.len() <= max_slot_w {
                lines.push(Line::from(vec![
                    Span::styled(" │   ", Style::default().fg(theme.shard_border)),
                    Span::styled(slot_prefix, Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                    Span::styled(raw_ranges, Style::default().fg(theme.shard_slot_range)),
                ]));
            } else {
                let chunks = Self::wrap_slots(&raw_ranges, max_slot_w);
                for (idx, chunk) in chunks.iter().enumerate() {
                    if idx == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(" │   ", Style::default().fg(theme.shard_border)),
                            Span::styled(slot_prefix, Style::default().fg(theme.shard_slot_label).add_modifier(Modifier::BOLD)),
                            Span::styled(chunk.clone(), Style::default().fg(theme.shard_slot_range)),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(" │          ", Style::default().fg(theme.shard_border)),
                            Span::styled(chunk.clone(), Style::default().fg(theme.shard_slot_range)),
                        ]));
                    }
                }
            }

            // Replicas Info
            if shard.replicas.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(" │   └── (No replicas configured)", Style::default().fg(theme.text_muted)),
                ]));
            } else {
                for (rep_idx, replica) in shard.replicas.iter().enumerate() {
                    let is_last = rep_idx == shard.replicas.len() - 1;
                    let branch_stem = if is_last { " │   └──" } else { " │   ├──" };
                    let rep_status_color = if replica.is_healthy { theme.status_healthy } else { theme.status_critical };
                    let rep_status_str = if replica.is_healthy { "[HEALTHY]" } else { "[FAIL]" };

                    if width >= 65 {
                        lines.push(Line::from(vec![
                            Span::styled(format!("{} [Replica] ", branch_stem), Style::default().fg(theme.shard_replica_title)),
                            Span::styled(format!("@{} ", replica.id), Style::default().fg(theme.shard_node_id)),
                            Span::styled(format!("{} ", replica.address), Style::default().fg(theme.text_secondary)),
                            Span::styled(format!("{} ", rep_status_str), Style::default().fg(rep_status_color)),
                            Span::styled(format!("{:.1}ms", replica.ping_ms), Style::default().fg(theme.text_muted)),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(format!("{} [Replica] ", branch_stem), Style::default().fg(theme.shard_replica_title)),
                            Span::styled(format!("@{} ", replica.id), Style::default().fg(theme.shard_node_id)),
                            Span::styled(format!("{}", rep_status_str), Style::default().fg(rep_status_color)),
                        ]));
                        let pad = if is_last { " │       " } else { " │   │   " };
                        lines.push(Line::from(vec![
                            Span::styled(pad, Style::default().fg(theme.shard_border)),
                            Span::styled(format!("{} · {:.1}ms", replica.address, replica.ping_ms), Style::default().fg(theme.text_muted)),
                        ]));
                    }
                }
            }

            // Bottom Border: ╰──────────────────────────────────────╯
            let bot_dash_count = box_w.saturating_sub(2);
            lines.push(Line::from(vec![
                Span::styled(" ╰", Style::default().fg(theme.shard_border)),
                Span::styled("─".repeat(bot_dash_count), Style::default().fg(theme.shard_border)),
                Span::styled("╯", Style::default().fg(theme.shard_border)),
            ]));

            lines.push(Line::from(""));
            items.push(ListItem::new(lines));
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

        if width >= 65 {
            let summary_line = Line::from(vec![
                Span::styled(" [Standalone Instance: ", Style::default().fg(theme.text_muted)),
                role_badge,
                Span::styled(format!(" · Connected Slaves: {}]", repl_info.connected_slaves), Style::default().fg(theme.border_focused)),
            ]);
            items.push(ListItem::new(vec![summary_line, Line::from("")]));
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
                    Span::styled(" │  Endpoint: ", Style::default().fg(theme.text_muted)),
                    Span::styled(node.address.clone(), Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" · Status: [HEALTHY] · Ping: {:.2}ms", node.ping_ms), Style::default().fg(theme.status_healthy)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(" │  Endpoint: ", Style::default().fg(theme.text_muted)),
                    Span::styled(node.address.clone(), Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" │   ", Style::default().fg(theme.cluster_border)),
                    Span::styled(format!("Status: [HEALTHY] · Ping: {:.2}ms", node.ping_ms), Style::default().fg(theme.status_healthy)),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled(" │  Keyspace: ", Style::default().fg(theme.text_muted)),
                Span::styled("Databases 0~15 (Standalone Store)", Style::default().fg(theme.status_warning)),
            ]));
        }

        // Replication section
        if repl_info.role == "master" {
            if width >= 55 {
                lines.push(Line::from(vec![
                    Span::styled(" │  Replication Offset: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{}", repl_info.master_repl_offset), Style::default().fg(theme.border_focused)),
                    Span::styled(format!(" · Connected Replicas: {}", repl_info.connected_slaves), Style::default().fg(theme.text_muted)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(" │  Repl Offset: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{}", repl_info.master_repl_offset), Style::default().fg(theme.border_focused)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" │  Connected Replicas: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{}", repl_info.connected_slaves), Style::default().fg(theme.text_primary)),
                ]));
            }

            if repl_info.slaves.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(" │  (No slave instances currently connected)", Style::default().fg(theme.text_muted)),
                ]));
            } else {
                for (i, slave) in repl_info.slaves.iter().enumerate() {
                    if width >= 60 {
                        lines.push(Line::from(vec![
                            Span::styled(format!(" │   ├── [Slave #{}] {}:{} ", i + 1, slave.ip, slave.port), Style::default().fg(theme.shard_replica_title)),
                            Span::styled(format!("State: {} · Offset: {} · Lag: {}s", slave.state, slave.offset, slave.lag), Style::default().fg(theme.text_muted)),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(format!(" │   ├── [Slave #{}] {}:{}", i + 1, slave.ip, slave.port), Style::default().fg(theme.shard_replica_title)),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(" │   │   ", Style::default().fg(theme.cluster_border)),
                            Span::styled(format!("State: {} · Offset: {} · Lag: {}s", slave.state, slave.offset, slave.lag), Style::default().fg(theme.text_muted)),
                        ]));
                    }
                }
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled(" │  Master Link: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    format!("{}:{} ({})", repl_info.master_host.as_deref().unwrap_or("-"), repl_info.master_port.unwrap_or(0), repl_info.master_link_status.as_deref().unwrap_or("unknown")),
                    Style::default().fg(theme.status_healthy),
                ),
            ]));
        }

        // Bottom border
        let bot_dash_count = box_w.saturating_sub(2);
        lines.push(Line::from(vec![
            Span::styled(" ╰", Style::default().fg(theme.cluster_border)),
            Span::styled("─".repeat(bot_dash_count), Style::default().fg(theme.cluster_border)),
            Span::styled("╯", Style::default().fg(theme.cluster_border)),
        ]));

        items.push(ListItem::new(lines));
    }
}
