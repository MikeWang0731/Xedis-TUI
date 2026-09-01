use crate::backend::XedisClient;
use crate::config::{AppConfig, LayoutPreset};
use crate::core::autocomplete::{AutocompleteEngine, SuggestionItem};
use crate::core::guard::{DangerAssessment, SafetyGuard};
use crate::core::history::HistoryManager;
use crate::core::macro_engine::MacroEngine;
use crate::core::router::{CommandRouter, CommandType, ParsedCommand};
use crate::ui::stream_view::{ExecutionRecord, StreamView};
use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    LeftStream,
    RightDashboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveRightTab {
    Telemetry,
    Cluster,
    Slowlog,
}

pub struct App {
    pub should_quit: bool,
    pub config: AppConfig,
    pub client: XedisClient,
    pub layout_preset: LayoutPreset,
    pub custom_split: Option<u16>,
    pub focused_pane: FocusedPane,
    pub active_tab: ActiveRightTab,
    pub input_buffer: String,
    pub cursor_pos: usize,
    pub scroll_offset: usize,
    pub cluster_scroll_offset: usize,
    pub slowlog_scroll_offset: usize,
    pub poll_interval: Duration,
    pub is_poll_paused: bool,
    pub last_poll_instant: Instant,
    pub records: Vec<ExecutionRecord>,
    pub history: HistoryManager,
    pub autocomplete_items: Vec<SuggestionItem>,
    pub autocomplete_idx: usize,
    pub autocomplete_active: bool,
    pub autocomplete_replace_range: (usize, usize),
    pub pending_guard: Option<(ParsedCommand, DangerAssessment)>,
    pub help_active: bool,
    pub help_tab: usize,
    pub help_scroll_offset: usize,
}

impl App {
    pub async fn new(config: AppConfig) -> Self {
        let conn_url = match &config.password {
            Some(pwd) if !pwd.is_empty() => format!("redis://:{}@{}:{}", pwd, config.host, config.port),
            _ => format!("redis://{}:{}", config.host, config.port),
        };
        let client = XedisClient::connect(&conn_url, config.cluster_mode).await;

        let initial_records = Vec::new();

        let poll_interval = Duration::from_millis(config.poll_interval_ms.max(50));
        let history = HistoryManager::default();

        Self {
            should_quit: false,
            layout_preset: config.default_layout,
            custom_split: None,
            focused_pane: FocusedPane::LeftStream,
            active_tab: ActiveRightTab::Telemetry,
            input_buffer: String::new(),
            cursor_pos: 0,
            scroll_offset: 0,
            cluster_scroll_offset: 0,
            slowlog_scroll_offset: 0,
            poll_interval,
            is_poll_paused: false,
            last_poll_instant: Instant::now(),
            records: initial_records,
            client,
            config,
            history,
            autocomplete_items: Vec::new(),
            autocomplete_idx: 0,
            autocomplete_active: false,
            autocomplete_replace_range: (0, 0),
            pending_guard: None,
            help_active: false,
            help_tab: 0,
            help_scroll_offset: 0,
        }
    }

    pub fn update_autocomplete(&mut self) {
        let (items, range) = AutocompleteEngine::get_suggestions(
            &self.input_buffer,
            self.cursor_pos,
            &self.client.telemetry.nodes(),
        );

        if !items.is_empty() {
            self.autocomplete_items = items;
            self.autocomplete_replace_range = range;
            self.autocomplete_idx = self.autocomplete_idx.min(self.autocomplete_items.len().saturating_sub(1));
            self.autocomplete_active = true;
        } else {
            self.autocomplete_items.clear();
            self.autocomplete_idx = 0;
            self.autocomplete_active = false;
        }
    }

    pub fn apply_autocomplete(&mut self) {
        if !self.autocomplete_active || self.autocomplete_items.is_empty() {
            return;
        }

        let selected = &self.autocomplete_items[self.autocomplete_idx];
        let (start, end) = self.autocomplete_replace_range;

        let safe_start = start.min(self.input_buffer.len());
        let safe_end = end.min(self.input_buffer.len());

        let before = &self.input_buffer[..safe_start];
        let after = &self.input_buffer[safe_end..];

        let inserted = &selected.completion_text;
        self.input_buffer = format!("{}{}{}", before, inserted, after);
        self.cursor_pos = safe_start + inserted.len();

        self.autocomplete_active = false;
        self.autocomplete_items.clear();
        self.autocomplete_idx = 0;
    }

    pub fn scroll_stream_up(&mut self, delta: usize) {
        let total = StreamView::total_lines_count(&self.records, 80);
        let max_scroll = total.saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + delta).min(max_scroll);
    }


    pub fn scroll_stream_down(&mut self, delta: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(delta);
    }

    pub fn scroll_cluster_up(&mut self, delta: usize) {
        self.cluster_scroll_offset = self.cluster_scroll_offset.saturating_sub(delta);
    }

    pub fn scroll_cluster_down(&mut self, delta: usize) {
        let max_s = self.client.telemetry.topology.shards.len().saturating_sub(1);
        self.cluster_scroll_offset = (self.cluster_scroll_offset + delta).min(max_s);
    }

    pub fn scroll_slowlog_up(&mut self, delta: usize) {
        self.slowlog_scroll_offset = self.slowlog_scroll_offset.saturating_sub(delta);
    }

    pub fn scroll_slowlog_down(&mut self, delta: usize) {
        let max_s = self.client.telemetry.slowlogs.len().saturating_sub(1);
        self.slowlog_scroll_offset = (self.slowlog_scroll_offset + delta).min(max_s);
    }

    pub async fn handle_key(&mut self, key: KeyEvent) {
        // 0. Safety Guard Modal Interception (Top Priority)
        if self.pending_guard.is_some() {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Some((parsed, _)) = self.pending_guard.take() {
                        self.execute_parsed_command(parsed).await;
                    }
                    return;
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.pending_guard = None;
                    return;
                }
                _ => return,
            }
        }

        // 0.1 Help Modal Interception
        if self.help_active {
            match key.code {
                KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.help_active = false;
                    return;
                }
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                    self.help_tab = (self.help_tab + 1) % 4;
                    self.help_scroll_offset = 0;
                    return;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.help_tab = if self.help_tab == 0 { 3 } else { self.help_tab - 1 };
                    self.help_scroll_offset = 0;
                    return;
                }
                KeyCode::Char('1') => {
                    self.help_tab = 0;
                    self.help_scroll_offset = 0;
                    return;
                }
                KeyCode::Char('2') => {
                    self.help_tab = 1;
                    self.help_scroll_offset = 0;
                    return;
                }
                KeyCode::Char('3') => {
                    self.help_tab = 2;
                    self.help_scroll_offset = 0;
                    return;
                }
                KeyCode::Char('4') => {
                    self.help_tab = 3;
                    self.help_scroll_offset = 0;
                    return;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
                    return;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_add(1);
                    return;
                }
                KeyCode::PageUp => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_sub(5);
                    return;
                }
                KeyCode::PageDown => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_add(5);
                    return;
                }
                KeyCode::Home => {
                    self.help_scroll_offset = 0;
                    return;
                }
                _ => return,
            }
        }

        // 1. Global Shortcuts
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('q') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('b') => {
                    self.layout_preset = self.layout_preset.next();
                    self.custom_split = None;
                    return;
                }
                KeyCode::Char('[') => {
                    let current = self.custom_split.unwrap_or_else(|| self.layout_preset.split_ratio());
                    self.custom_split = Some(current.saturating_sub(5).max(20));
                    return;
                }
                KeyCode::Char(']') => {
                    let current = self.custom_split.unwrap_or_else(|| self.layout_preset.split_ratio());
                    self.custom_split = Some((current + 5).min(90));
                    return;
                }
                KeyCode::Char('l') => {
                    self.records.clear();
                    return;
                }
                KeyCode::Char('u') => {
                    self.input_buffer.clear();
                    self.cursor_pos = 0;
                    self.update_autocomplete();
                    return;
                }
                KeyCode::Char('p') => {
                    self.scroll_stream_up(5);
                    return;
                }
                KeyCode::Char('n') => {
                    self.scroll_stream_down(5);
                    return;
                }
                _ => {}
            }
        }

        // 2. Autocomplete Active Interception
        if self.autocomplete_active && !self.autocomplete_items.is_empty() {
            match key.code {
                KeyCode::Up => {
                    self.autocomplete_idx = self.autocomplete_idx.saturating_sub(1);
                    return;
                }
                KeyCode::Down => {
                    if self.autocomplete_idx + 1 < self.autocomplete_items.len() {
                        self.autocomplete_idx += 1;
                    }
                    return;
                }
                KeyCode::Tab | KeyCode::Enter => {
                    self.apply_autocomplete();
                    return;
                }
                KeyCode::Esc => {
                    self.autocomplete_active = false;
                    self.autocomplete_items.clear();
                    self.autocomplete_idx = 0;
                    return;
                }
                _ => {}
            }
        }

        // 3. Tab navigation and View switches
        match key.code {
            KeyCode::F(1) => {
                self.help_active = !self.help_active;
                self.help_scroll_offset = 0;
                return;
            }
            KeyCode::F(5) => {
                self.layout_preset = self.layout_preset.next();
                self.custom_split = None;
                return;
            }
            KeyCode::F(2) => {
                self.active_tab = ActiveRightTab::Telemetry;
                return;
            }
            KeyCode::F(3) => {
                self.active_tab = ActiveRightTab::Cluster;
                return;
            }
            KeyCode::F(4) => {
                self.active_tab = ActiveRightTab::Slowlog;
                return;
            }
            KeyCode::Tab => {
                self.focused_pane = match self.focused_pane {
                    FocusedPane::LeftStream => FocusedPane::RightDashboard,
                    FocusedPane::RightDashboard => FocusedPane::LeftStream,
                };
                return;
            }
            _ => {}
        }

        // 4. Right Pane Focused Navigation
        if self.focused_pane == FocusedPane::RightDashboard {
            match key.code {
                KeyCode::Up => {
                    match self.active_tab {
                        ActiveRightTab::Cluster => self.scroll_cluster_up(1),
                        ActiveRightTab::Slowlog => self.scroll_slowlog_up(1),
                        _ => {}
                    }
                    return;
                }
                KeyCode::Down => {
                    match self.active_tab {
                        ActiveRightTab::Cluster => self.scroll_cluster_down(1),
                        ActiveRightTab::Slowlog => self.scroll_slowlog_down(1),
                        _ => {}
                    }
                    return;
                }
                KeyCode::PageUp => {
                    match self.active_tab {
                        ActiveRightTab::Cluster => self.scroll_cluster_up(5),
                        ActiveRightTab::Slowlog => self.scroll_slowlog_up(5),
                        _ => {}
                    }
                    return;
                }
                KeyCode::PageDown => {
                    match self.active_tab {
                        ActiveRightTab::Cluster => self.scroll_cluster_down(5),
                        ActiveRightTab::Slowlog => self.scroll_slowlog_down(5),
                        _ => {}
                    }
                    return;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.active_tab = match self.active_tab {
                        ActiveRightTab::Telemetry => ActiveRightTab::Slowlog,
                        ActiveRightTab::Cluster => ActiveRightTab::Telemetry,
                        ActiveRightTab::Slowlog => ActiveRightTab::Cluster,
                    };
                    return;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.active_tab = match self.active_tab {
                        ActiveRightTab::Telemetry => ActiveRightTab::Cluster,
                        ActiveRightTab::Cluster => ActiveRightTab::Slowlog,
                        ActiveRightTab::Slowlog => ActiveRightTab::Telemetry,
                    };
                    return;
                }
                KeyCode::Char('1') => {
                    self.active_tab = ActiveRightTab::Telemetry;
                    return;
                }
                KeyCode::Char('2') => {
                    self.active_tab = ActiveRightTab::Cluster;
                    return;
                }
                KeyCode::Char('3') => {
                    self.active_tab = ActiveRightTab::Slowlog;
                    return;
                }
                KeyCode::Home => {
                    self.cluster_scroll_offset = 0;
                    self.slowlog_scroll_offset = 0;
                    return;
                }
                KeyCode::Esc => {
                    self.focused_pane = FocusedPane::LeftStream;
                    return;
                }
                _ => {}
            }
        }

        // 5. Left Pane Input & History
        match key.code {
            KeyCode::PageUp => {
                self.scroll_stream_up(6);
            }
            KeyCode::PageDown => {
                self.scroll_stream_down(6);
            }
            KeyCode::Up => {
                if let Some(prev_cmd) = self.history.navigate_prev(&self.input_buffer) {
                    self.input_buffer = prev_cmd;
                    self.cursor_pos = self.input_buffer.len();
                    self.update_autocomplete();
                }
            }
            KeyCode::Down => {
                if let Some(next_cmd) = self.history.navigate_next() {
                    self.input_buffer = next_cmd;
                    self.cursor_pos = self.input_buffer.len();
                    self.update_autocomplete();
                }
            }
            KeyCode::Enter => {
                self.submit_command().await;
            }
            KeyCode::Char(c) => {
                self.input_buffer.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.history.reset_nav();
                self.update_autocomplete();
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.input_buffer.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    self.history.reset_nav();
                    self.update_autocomplete();
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.update_autocomplete();
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input_buffer.len() {
                    self.cursor_pos += 1;
                    self.update_autocomplete();
                }
            }
            KeyCode::Esc => {
                self.autocomplete_active = false;
                self.autocomplete_items.clear();
            }
            _ => {}
        }
    }

    pub async fn submit_command(&mut self) {
        let input = self.input_buffer.trim().to_string();
        if input.is_empty() {
            return;
        }

        self.history.push(&input);
        self.input_buffer.clear();
        self.cursor_pos = 0;
        self.scroll_offset = 0; // Auto-scroll to bottom on new execution
        self.autocomplete_active = false;
        self.autocomplete_items.clear();

        let parsed = match CommandRouter::parse(&input) {
            Some(p) => p,
            None => return,
        };

        // Open help modal on /help or /?
        if let CommandType::Macro { name, .. } = &parsed.command_type {
            if name == "/help" || name == "/?" {
                self.help_active = true;
                self.help_tab = 0;
                self.help_scroll_offset = 0;
            }
        }

        // Safety Guard Check
        if self.config.safety_guard_enabled {
            if let Some(assessment) = SafetyGuard::inspect(&parsed) {
                self.pending_guard = Some((parsed, assessment));
                return;
            }
        }

        self.execute_parsed_command(parsed).await;
    }

    pub async fn execute_parsed_command(&mut self, parsed: ParsedCommand) {
        let input = parsed.raw_input.clone();
        let target_node = parsed.target_node.clone();
        let now = Local::now().format("%H:%M:%S").to_string();

        let (result, duration) = match parsed.command_type {
            CommandType::Macro { name, args } => {
                let lower_name = name.to_lowercase();
                if lower_name == "/clear" {
                    self.records.clear();
                    return;
                }

                if lower_name == "/interval" || lower_name == "/poll" {
                    let start = Instant::now();
                    if let Some(arg) = args.first() {
                        let arg_lower = arg.to_lowercase();
                        if arg_lower == "pause" || arg_lower == "stop" || arg_lower == "0" {
                            self.is_poll_paused = true;
                        } else if arg_lower == "resume" || arg_lower == "start" {
                            self.is_poll_paused = false;
                        } else if arg_lower.ends_with("ms") {
                            if let Ok(ms) = arg_lower.trim_end_matches("ms").parse::<u64>() {
                                self.poll_interval = Duration::from_millis(ms.max(50));
                                self.is_poll_paused = false;
                            }
                        } else if arg_lower.ends_with('s') {
                            if let Ok(s) = arg_lower.trim_end_matches('s').parse::<u64>() {
                                self.poll_interval = Duration::from_secs(s.max(1));
                                self.is_poll_paused = false;
                            }
                        } else if let Ok(ms) = arg_lower.parse::<u64>() {
                            if ms == 0 {
                                self.is_poll_paused = true;
                            } else {
                                self.poll_interval = Duration::from_millis(ms.max(50));
                                self.is_poll_paused = false;
                            }
                        }
                    }
                    let res = MacroEngine::format_interval_result(
                        self.poll_interval.as_millis() as u64,
                        self.is_poll_paused,
                    );
                    (res, start.elapsed())
                } else if lower_name == "/settings" || lower_name == "/config" {
                    let start = Instant::now();
                    let res = MacroEngine::format_settings(
                        &self.config.host,
                        self.config.port,
                        self.client.telemetry.is_cluster,
                        self.layout_preset.name(),
                        self.poll_interval.as_millis() as u64,
                        self.is_poll_paused,
                    );
                    (res, start.elapsed())
                } else {
                    self.client.execute_macro(target_node.as_deref(), &name, &args).await
                }
            }
            CommandType::Native { cmd, args } => {
                self.client.execute_command(target_node.as_deref(), &cmd, &args).await
            }
        };

        self.records.push(ExecutionRecord {
            target_node,
            command: input,
            timestamp: now,
            duration,
            result,
        });

        // Trim records to config.max_stream_records
        if self.records.len() > self.config.max_stream_records {
            let overflow = self.records.len() - self.config.max_stream_records;
            self.records.drain(0..overflow);
        }
    }

    pub async fn on_tick(&mut self) {
        if !self.is_poll_paused && self.last_poll_instant.elapsed() >= self.poll_interval {
            self.client.poll_telemetry().await;
            self.last_poll_instant = Instant::now();
        }
    }
}
