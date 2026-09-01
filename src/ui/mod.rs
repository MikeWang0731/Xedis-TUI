pub mod cluster_view;
pub mod layout;
pub mod popups;
pub mod slowlog_view;
pub mod stream_view;
pub mod telemetry_view;

use crate::app::{ActiveRightTab, App, FocusedPane};
use crate::ui::cluster_view::ClusterView;
use crate::ui::layout::MainLayout;
use crate::ui::popups::{AutocompletePopup, GuardPopup, HelpPopup};
use crate::ui::slowlog_view::SlowlogView;
use crate::ui::stream_view::StreamView;
use crate::ui::telemetry_view::TelemetryView;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();
    let main_layout = MainLayout::build(size, app.layout_preset, app.custom_split);

    // 1. Render Top Header Bar
    render_header(f, main_layout.header, app);

    // 2. Render Left Pane (Command Stream + Input) with Dimming Support when autocomplete or modal is active
    let is_dimmed = app.autocomplete_active || app.pending_guard.is_some() || app.help_active;
    StreamView::render(
        f,
        main_layout.left_pane,
        &app.records,
        &app.input_buffer,
        app.cursor_pos,
        app.scroll_offset,
        app.focused_pane == FocusedPane::LeftStream && !is_dimmed,
        is_dimmed,
    );

    // 3. Render Right Pane (if not Zen mode)
    if let Some(right_area) = main_layout.right_pane {
        render_right_pane(f, right_area, app);
    }

    // 4. Render Bottom Footer
    render_footer(f, main_layout.footer, app);

    // 5. Render Autocomplete Popover (Floating above prompt with side doc inspector)
    if app.autocomplete_active && !app.autocomplete_items.is_empty() {
        let input_area = Rect {
            x: main_layout.left_pane.x,
            y: main_layout.left_pane.bottom().saturating_sub(3),
            width: main_layout.left_pane.width,
            height: 3,
        };
        AutocompletePopup::render(f, input_area, &app.autocomplete_items, app.autocomplete_idx);
    }

    // 6. Render Help Modal (F1 Handbook)
    if app.help_active {
        HelpPopup::render(f, size, app.help_tab, app.help_scroll_offset);
    }

    // 7. Render Safety Guard Interceptor Modal (Highest priority overlay)
    if let Some((_, assessment)) = &app.pending_guard {
        GuardPopup::render(f, size, assessment);
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let w = area.width;
    let (brand_len, layout_len) = if w >= 110 {
        (16, 36)
    } else if w >= 85 {
        (12, 34)
    } else if w >= 65 {
        (9, 24)
    } else {
        (9, 16)
    };

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(brand_len), // App Brand
            Constraint::Min(12),           // Connection Info
            Constraint::Length(layout_len), // Layout & Focus Indicator
        ])
        .split(area);

    // Brand
    let brand_text = if w >= 90 {
        Line::from(vec![
            Span::styled(" [XEDIS] ", Style::default().bg(Color::Rgb(15, 45, 60)).fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" v0.1.0 ", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" [XEDIS] ", Style::default().bg(Color::Rgb(15, 45, 60)).fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])
    };
    f.render_widget(Paragraph::new(brand_text), header_chunks[0]);

    // Connection Info
    let (status_tag, status_color) = if app.client.telemetry.connected {
        (" ● CONNECTED ", Color::Green)
    } else {
        (" ● OFFLINE ", Color::Yellow)
    };

    let cluster_tag = if app.client.telemetry.is_cluster {
        Span::styled(" [Cluster] ", Style::default().fg(Color::Rgb(180, 160, 255)))
    } else {
        Span::styled(" [Standalone] ", Style::default().fg(Color::Cyan))
    };

    let conn_line = if w >= 80 {
        Line::from(vec![
            Span::styled(status_tag, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled(&app.client.telemetry.server_desc, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            cluster_tag,
            Span::styled(format!(" Ping: {:.2}ms", app.client.telemetry.metrics.ping_latency_ms), Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled(status_tag, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled(&app.client.telemetry.server_desc, Style::default().fg(Color::White)),
        ])
    };
    f.render_widget(Paragraph::new(conn_line), header_chunks[1]);

    // Layout & Focus indicator
    let focus_tag = if app.focused_pane == FocusedPane::RightDashboard {
        Span::styled(" [Right] ", Style::default().bg(Color::Rgb(40, 30, 80)).fg(Color::Rgb(180, 160, 255)).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" [Stream] ", Style::default().bg(Color::Rgb(20, 40, 60)).fg(Color::Cyan))
    };

    let layout_badge = if layout_len >= 34 {
        Line::from(vec![
            Span::styled(
                format!(" Layout: {} ", app.layout_preset.name()),
                Style::default().bg(Color::Rgb(20, 30, 50)).fg(Color::Cyan),
            ),
            focus_tag,
        ])
    } else if layout_len >= 24 {
        Line::from(vec![
            Span::styled(
                format!(" {} ", app.layout_preset.name()),
                Style::default().bg(Color::Rgb(20, 30, 50)).fg(Color::Cyan),
            ),
            focus_tag,
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!(" {} ", app.layout_preset.name().split_whitespace().next().unwrap_or("")),
                Style::default().bg(Color::Rgb(20, 30, 50)).fg(Color::Cyan),
            ),
            focus_tag,
        ])
    };
    f.render_widget(Paragraph::new(layout_badge).alignment(ratatui::layout::Alignment::Right), header_chunks[2]);
}

fn render_right_pane(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let pane_w = area.width;
    let tab_titles = if pane_w >= 68 {
        vec![
            " [1] Telemetry (F2) ",
            " [2] Cluster Topology (F3) ",
            " [3] Slowlog & Troubleshoot (F4) ",
        ]
    } else if pane_w >= 52 {
        vec![
            " [1] Telemetry ",
            " [2] Cluster ",
            " [3] Slowlog ",
        ]
    } else if pane_w >= 36 {
        vec![
            " [1] Stat ",
            " [2] Clust ",
            " [3] Slow ",
        ]
    } else {
        vec![
            " 1:Stat ",
            " 2:Node ",
            " 3:Slow ",
        ]
    };

    let selected_idx = match app.active_tab {
        ActiveRightTab::Telemetry => 0,
        ActiveRightTab::Cluster => 1,
        ActiveRightTab::Slowlog => 2,
    };

    let tab_style = if app.focused_pane == FocusedPane::RightDashboard {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title_text = if pane_w >= 54 {
        " Telemetry & Cluster Dashboard "
    } else if pane_w >= 38 {
        " Dashboard "
    } else {
        " Dash "
    };

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(tab_style)
                .title(title_text),
        )
        .select(selected_idx)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, chunks[0]);

    // Render Tab Content
    match app.active_tab {
        ActiveRightTab::Telemetry => {
            TelemetryView::render(
                f,
                chunks[1],
                &app.client.telemetry,
                app.poll_interval.as_millis() as u64,
                app.is_poll_paused,
            );
        }
        ActiveRightTab::Cluster => {
            ClusterView::render(
                f,
                chunks[1],
                &app.client.telemetry.topology,
                app.cluster_scroll_offset,
            );
        }
        ActiveRightTab::Slowlog => {
            SlowlogView::render(
                f,
                chunks[1],
                &app.client.telemetry.slowlogs,
                app.slowlog_scroll_offset,
            );
        }
    }
}

fn render_footer(f: &mut Frame, area: Rect, _app: &App) {
    let w = area.width;
    let shortcuts = if w >= 95 {
        Line::from(vec![
            Span::styled(" [Tab] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Cyan)),
            Span::styled("Focus ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [F1] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Yellow)),
            Span::styled("Handbook ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [F5] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Cyan)),
            Span::styled("Layout ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [F2~F4/1~3] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Cyan)),
            Span::styled("Tabs ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [/] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Yellow)),
            Span::styled("Macros ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [@] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Rgb(180, 160, 255))),
            Span::styled("Route ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [Up/Dn] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Cyan)),
            Span::styled("Nav ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [Ctrl+C] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Red)),
            Span::styled("Quit", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" [Tab] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Cyan)),
            Span::styled("Focus ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [F1] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Yellow)),
            Span::styled("Help ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [F5] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Cyan)),
            Span::styled("Layout ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [F2-F4] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Cyan)),
            Span::styled("Tabs ", Style::default().fg(Color::DarkGray)),
            Span::styled(" [/] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Yellow)),
            Span::styled(" [Ctrl+C] ", Style::default().bg(Color::Rgb(30, 40, 60)).fg(Color::Red)),
            Span::styled("Quit", Style::default().fg(Color::DarkGray)),
        ])
    };

    f.render_widget(Paragraph::new(shortcuts), area);
}
