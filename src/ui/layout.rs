use crate::config::LayoutPreset;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct MainLayout {
    pub header: Rect,
    #[allow(dead_code)]
    pub body: Rect,
    pub footer: Rect,
    pub left_pane: Rect,
    pub right_pane: Option<Rect>,
}

impl MainLayout {
    pub fn build(area: Rect, preset: LayoutPreset, custom_split: Option<u16>) -> Self {
        // Vertical layout: Header banner (6 for standard displays), Body (Min 5), Footer (2)
        let header_height = if area.height >= 16 {
            6
        } else if area.height >= 10 {
            4
        } else {
            2
        };

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height),
                Constraint::Min(5),
                Constraint::Length(2),
            ])
            .split(area);

        let header = vertical[0];
        let body = vertical[1];
        let footer = vertical[2];

        let ratio = custom_split.unwrap_or_else(|| preset.split_ratio());

        if ratio >= 100 || preset == LayoutPreset::Zen {
            Self {
                header,
                body,
                footer,
                left_pane: body,
                right_pane: None,
            }
        } else {
            let horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(ratio),
                    Constraint::Percentage(100 - ratio),
                ])
                .split(body);

            Self {
                header,
                body,
                footer,
                left_pane: horizontal[0],
                right_pane: Some(horizontal[1]),
            }
        }
    }
}
