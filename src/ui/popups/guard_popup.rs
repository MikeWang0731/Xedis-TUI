use crate::core::guard::{DangerAssessment, RiskLevel};
use crate::ui::theme::ThemePalette;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct GuardPopup;

impl GuardPopup {
    pub fn render(f: &mut Frame, area: Rect, assessment: &DangerAssessment, theme: &ThemePalette) {
        // Calculate centered modal rect
        let popup_w = (area.width.saturating_sub(10)).clamp(46, 78);
        let popup_h = 16.min(area.height.saturating_sub(4)).max(12);

        let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
        let popup_y = area.y + (area.height.saturating_sub(popup_h)) / 2;

        let modal_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_w,
            height: popup_h,
        };

        // 1. Clear background
        f.render_widget(Clear, modal_area);

        let (border_color, badge_bg, badge_fg) = match assessment.level {
            RiskLevel::Level3Blocking => (
                theme.guard_l3_border,
                theme.guard_l3_badge_bg,
                theme.guard_l3_badge_fg,
            ),
            RiskLevel::Level2Warning => (
                theme.guard_l2_border,
                theme.guard_l2_badge_bg,
                theme.guard_l2_badge_fg,
            ),
        };

        let modal_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(border_color))
            .title(Line::from(vec![
                Span::styled(" [SAFETY GUARD] ", Style::default().bg(badge_bg).fg(badge_fg).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", assessment.level.badge_title()), Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
            ]));

        let inner = modal_block.inner(modal_area);
        f.render_widget(modal_block, modal_area);

        // Layout inner chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Command title
                Constraint::Length(1), // Spacer
                Constraint::Min(4),    // Danger Reason & Suggestion
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Action buttons
            ])
            .split(inner);

        // Header: Command attempted
        let cmd_line = Line::from(vec![
            Span::styled("拦截指令: ", Style::default().fg(theme.guard_cmd_title)),
            Span::styled(&assessment.command_str, Style::default().fg(theme.cmd_name_macro).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  ({})", assessment.title), Style::default().fg(theme.cmd_name_native)),
        ]);
        f.render_widget(Paragraph::new(cmd_line), chunks[0]);

        // Middle: Reason + Suggestion
        let mut desc_lines = Vec::new();
        desc_lines.push(Line::from(vec![
            Span::styled("风险剖析: ", Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
            Span::styled(&assessment.reason, Style::default().fg(theme.guard_reason_text)),
        ]));
        desc_lines.push(Line::from(""));
        desc_lines.push(Line::from(vec![
            Span::styled("安全建议: ", Style::default().fg(theme.guard_suggestion_title).add_modifier(Modifier::BOLD)),
            Span::styled(&assessment.suggestion, Style::default().fg(theme.guard_suggestion_text)),
        ]));

        let desc_widget = Paragraph::new(desc_lines).wrap(Wrap { trim: true });
        f.render_widget(desc_widget, chunks[2]);

        // Footer: Action Buttons
        let action_line = Line::from(vec![
            Span::styled(" [ Enter / 'y' ] ", Style::default().bg(theme.guard_btn_confirm_bg).fg(theme.guard_btn_confirm_fg).add_modifier(Modifier::BOLD)),
            Span::styled(" 强制放行执行   ", Style::default().fg(theme.text_primary)),
            Span::styled(" [ Esc / 'n' ] ", Style::default().bg(theme.guard_btn_cancel_bg).fg(theme.guard_btn_cancel_fg).add_modifier(Modifier::BOLD)),
            Span::styled(" 放弃并修改", Style::default().fg(theme.text_primary)),
        ]);
        let action_widget = Paragraph::new(action_line).alignment(ratatui::layout::Alignment::Center);
        f.render_widget(action_widget, chunks[4]);
    }
}
