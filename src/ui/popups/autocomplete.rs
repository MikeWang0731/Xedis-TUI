use crate::core::autocomplete::{SuggestionItem, SuggestionKind};
use crate::ui::theme::ThemePalette;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub struct AutocompletePopup;

impl AutocompletePopup {
    pub fn render(
        f: &mut Frame,
        input_area: Rect,
        items: &[SuggestionItem],
        selected_idx: usize,
        theme: &ThemePalette,
    ) {
        if items.is_empty() {
            return;
        }

        let max_visible = 6;
        let visible_count = items.len().min(max_visible);
        let list_h = (visible_count as u16) + 2; // +2 for top/bottom borders
        let list_y = input_area.y.saturating_sub(list_h);
        let popup_x = input_area.x + 2;

        let total_avail_w = input_area.width.saturating_sub(4);
        let has_doc_popup = total_avail_w >= 54;

        let list_w = if has_doc_popup {
            (total_avail_w / 2).min(34).max(24)
        } else {
            total_avail_w.min(76).max(28)
        };

        let list_area = Rect {
            x: popup_x,
            y: list_y,
            width: list_w,
            height: list_h,
        };

        // 1. Render Left Suggestion List Popup
        f.render_widget(Clear, list_area);

        let list_title = if list_w >= 32 {
            " Suggestions [↑/↓ Tab/Enter] "
        } else {
            " Suggestions "
        };

        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.auto_border))
            .title(Span::styled(
                list_title,
                Style::default().fg(theme.auto_border).add_modifier(Modifier::BOLD),
            ));

        let inner_list_area = list_block.inner(list_area);
        f.render_widget(list_block, list_area);

        // Windowed items
        let start_idx = if selected_idx >= visible_count {
            selected_idx - visible_count + 1
        } else {
            0
        };
        let end_idx = (start_idx + visible_count).min(items.len());

        let mut list_items = Vec::new();
        for (i, item) in items[start_idx..end_idx].iter().enumerate() {
            let actual_idx = start_idx + i;
            let is_selected = actual_idx == selected_idx;

            let (badge_bg, badge_fg) = match item.kind {
                SuggestionKind::Macro => (theme.auto_badge_macro_bg, theme.auto_badge_macro_fg),
                SuggestionKind::Node => (theme.auto_badge_node_bg, theme.auto_badge_node_fg),
                SuggestionKind::Command => (theme.auto_badge_cmd_bg, theme.auto_badge_cmd_fg),
                SuggestionKind::Subcommand => (theme.auto_badge_sub_bg, theme.auto_badge_sub_fg),
                SuggestionKind::Argument => (theme.auto_badge_arg_bg, theme.auto_badge_arg_fg),
            };

            let prefix_icon = if is_selected { "▶ " } else { "  " };

            let mut spans = vec![
                Span::styled(
                    prefix_icon,
                    if is_selected {
                        Style::default().fg(theme.auto_border).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text_muted)
                    },
                ),
                Span::styled(
                    format!(" {} ", item.kind.badge()),
                    Style::default().bg(badge_bg).fg(badge_fg).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    &item.display_title,
                    if is_selected {
                        Style::default().fg(theme.auto_item_selected_fg).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.auto_item_unselected_fg)
                    },
                ),
            ];

            // If no side doc popup, show short description if space permits
            if !has_doc_popup && list_w >= 50 && !item.description.is_empty() {
                spans.push(Span::styled(" - ", Style::default().fg(theme.text_muted)));
                let max_desc_len = (list_w as usize).saturating_sub(item.display_title.len() + 18);
                let desc_cut = if item.description.len() > max_desc_len {
                    format!("{}…", &item.description[..max_desc_len.saturating_sub(1)])
                } else {
                    item.description.clone()
                };
                spans.push(Span::styled(
                    desc_cut,
                    if is_selected {
                        Style::default().fg(theme.auto_border)
                    } else {
                        Style::default().fg(theme.auto_desc_fg)
                    },
                ));
            }

            let item_style = if is_selected {
                Style::default().bg(theme.auto_item_selected_bg)
            } else {
                Style::default()
            };

            list_items.push(ListItem::new(Line::from(spans)).style(item_style));
        }

        let list = List::new(list_items);
        f.render_widget(list, inner_list_area);

        // 2. Render Right Attached Documentation / Inspector Card with Independent Adaptive Height
        if has_doc_popup && selected_idx < items.len() {
            let selected_item = &items[selected_idx];
            let doc_x = popup_x + list_w + 1;
            let doc_w = total_avail_w.saturating_sub(list_w + 2).min(52);

            // Calculate needed height for documentation content independently
            let inner_w = (doc_w.saturating_sub(10) as usize).max(12);
            let desc_char_count = selected_item.description.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum::<usize>();
            let desc_lines = if desc_char_count > 0 { (desc_char_count / inner_w) + 1 } else { 0 };
            let total_content_lines = 1 /* Syntax */ + desc_lines.max(1) /* Desc */ + 1 /* Usage */;
            let doc_h = ((total_content_lines as u16) + 2).clamp(6, 8);
            let doc_y = input_area.y.saturating_sub(doc_h);

            let doc_area = Rect {
                x: doc_x,
                y: doc_y,
                width: doc_w,
                height: doc_h,
            };

            f.render_widget(Clear, doc_area);

            let (doc_border_color, doc_badge_title) = match selected_item.kind {
                SuggestionKind::Macro => (theme.status_warning, format!(" Macro: {} ", selected_item.display_title)),
                SuggestionKind::Node => (theme.shard_node_id, format!(" Node: {} ", selected_item.display_title)),
                SuggestionKind::Command => (theme.border_focused, format!(" Command: {} ", selected_item.display_title)),
                SuggestionKind::Subcommand => (theme.val_integer, format!(" Subcommand: {} ", selected_item.display_title)),
                SuggestionKind::Argument => (theme.status_warning, format!(" Option: {} ", selected_item.display_title)),
            };

            let doc_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(doc_border_color))
                .title(Span::styled(
                    doc_badge_title,
                    Style::default().fg(doc_border_color).add_modifier(Modifier::BOLD),
                ));

            let inner_doc = doc_block.inner(doc_area);
            f.render_widget(doc_block, doc_area);

            let mut doc_lines = Vec::new();

            // Syntax Line
            if !selected_item.syntax.is_empty() {
                doc_lines.push(Line::from(vec![
                    Span::styled("Syntax: ", Style::default().fg(theme.telemetry_label)),
                    Span::styled(&selected_item.syntax, Style::default().fg(theme.border_focused).add_modifier(Modifier::BOLD)),
                ]));
            }

            // Description Line (with auto wrap)
            if !selected_item.description.is_empty() {
                if let Some(rest) = selected_item.description.strip_prefix("[!]") {
                    doc_lines.push(Line::from(vec![
                        Span::styled("Desc:   ", Style::default().fg(theme.telemetry_label)),
                        Span::styled("[!] ", Style::default().fg(theme.status_critical).add_modifier(Modifier::BOLD)),
                        Span::styled(rest.trim_start(), Style::default().fg(theme.text_primary)),
                    ]));
                } else {
                    doc_lines.push(Line::from(vec![
                        Span::styled("Desc:   ", Style::default().fg(theme.telemetry_label)),
                        Span::styled(&selected_item.description, Style::default().fg(theme.text_primary)),
                    ]));
                }
            }

            // Example Line
            if !selected_item.example.is_empty() {
                doc_lines.push(Line::from(vec![
                    Span::styled("Usage:  ", Style::default().fg(theme.telemetry_label)),
                    Span::styled(&selected_item.example, Style::default().fg(theme.status_warning)),
                ]));
            }

            let doc_paragraph = Paragraph::new(doc_lines)
                .wrap(Wrap { trim: true });

            f.render_widget(doc_paragraph, inner_doc);
        }
    }
}
