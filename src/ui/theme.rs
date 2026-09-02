use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn toggle(&self) -> Self {
        match self {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ThemePalette {
    pub mode: ThemeMode,

    // Base text & neutral
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_dimmed: Color,
    pub border: Color,
    pub border_focused: Color,
    pub border_dimmed: Color,
    pub divider: Color,

    // Brand & Header
    pub brand_bg: Color,
    pub brand_fg: Color,
    pub brand_version: Color,
    pub conn_connected: Color,
    pub conn_offline: Color,
    pub conn_text: Color,
    pub conn_cluster: Color,
    pub conn_standalone: Color,
    pub header_badge_bg: Color,
    pub header_badge_fg: Color,
    pub focus_right_bg: Color,
    pub focus_right_fg: Color,
    pub focus_stream_bg: Color,
    pub focus_stream_fg: Color,

    // Stream & Input
    pub prompt_symbol: Color,
    pub prompt_input: Color,
    pub prompt_border_focused: Color,
    pub prompt_border_unfocused: Color,
    pub stream_border_focused: Color,
    pub stream_border_unfocused: Color,
    pub stream_border_dimmed: Color,

    // Execution Cards
    pub cmd_direct_bg: Color,
    pub cmd_direct_fg: Color,
    pub cmd_node_bg: Color,
    pub cmd_node_fg: Color,
    pub cmd_broadcast_bg: Color,
    pub cmd_broadcast_fg: Color,
    pub cmd_name_native: Color,
    pub cmd_name_macro: Color,
    pub cmd_meta: Color,

    // Formatted Values
    pub val_status: Color,
    pub val_integer: Color,
    pub val_string: Color,
    pub val_key: Color,
    pub val_info_section: Color,
    pub val_node_header: Color,
    pub val_error: Color,
    pub val_nil: Color,
    pub val_tree_root: Color,
    pub val_tree_tag: Color,
    pub val_tree_branch: Color,

    // Tables
    pub table_border: Color,
    pub table_header: Color,
    pub table_col1: Color,
    pub table_col2: Color,
    pub table_col_rest: Color,

    // JSON Highlight
    pub json_key: Color,
    pub json_string: Color,
    pub json_number: Color,
    pub json_boolean: Color,
    pub json_bracket: Color,
    pub json_colon: Color,

    // Telemetry Dashboard
    pub telemetry_label: Color,
    pub telemetry_value: Color,
    pub telemetry_card_border: Color,
    pub mem_bar_used: Color,
    pub mem_bar_rss: Color,
    pub mem_bar_free: Color,
    pub cpu_waveform: Color,
    pub status_healthy: Color,
    pub status_warning: Color,
    pub status_critical: Color,

    // Cluster View
    pub cluster_border: Color,
    pub shard_border: Color,
    pub shard_master_title: Color,
    pub shard_replica_title: Color,
    pub shard_node_id: Color,
    pub shard_slot_label: Color,
    pub shard_slot_range: Color,

    // Slowlog View
    pub slowlog_border: Color,
    pub slowlog_cmd: Color,
    pub slowlog_guidance_text: Color,

    // Autocomplete Popover
    pub auto_border: Color,
    pub auto_item_selected_bg: Color,
    pub auto_item_selected_fg: Color,
    pub auto_item_unselected_fg: Color,
    pub auto_desc_fg: Color,
    pub auto_badge_macro_bg: Color,
    pub auto_badge_macro_fg: Color,
    pub auto_badge_node_bg: Color,
    pub auto_badge_node_fg: Color,
    pub auto_badge_cmd_bg: Color,
    pub auto_badge_cmd_fg: Color,
    pub auto_badge_sub_bg: Color,
    pub auto_badge_sub_fg: Color,
    pub auto_badge_arg_bg: Color,
    pub auto_badge_arg_fg: Color,

    // Popups & Modals
    pub help_border: Color,
    pub help_brand_bg: Color,
    pub help_brand_fg: Color,
    pub help_tab_active: Color,
    pub help_tab_inactive: Color,
    pub help_tab_border: Color,
    pub help_title_yellow: Color,
    pub help_title_cyan: Color,
    pub help_title_purple: Color,
    pub help_title_green: Color,
    pub help_title_red: Color,

    // Safety Guard
    pub guard_l3_border: Color,
    pub guard_l3_badge_bg: Color,
    pub guard_l3_badge_fg: Color,
    pub guard_l2_border: Color,
    pub guard_l2_badge_bg: Color,
    pub guard_l2_badge_fg: Color,
    pub guard_cmd_title: Color,
    pub guard_reason_text: Color,
    pub guard_suggestion_title: Color,
    pub guard_suggestion_text: Color,
    pub guard_btn_confirm_bg: Color,
    pub guard_btn_confirm_fg: Color,
    pub guard_btn_cancel_bg: Color,
    pub guard_btn_cancel_fg: Color,

    // Footer
    pub footer_key_bg: Color,
    pub footer_key_cyan: Color,
    pub footer_key_yellow: Color,
    pub footer_key_purple: Color,
    pub footer_key_red: Color,
    pub footer_label: Color,
}

impl ThemePalette {
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,

            // Base
            text_primary: Color::Rgb(240, 245, 250),
            text_secondary: Color::Rgb(165, 180, 195),
            text_muted: Color::DarkGray,
            text_dimmed: Color::Rgb(100, 105, 110),
            border: Color::DarkGray,
            border_focused: Color::Cyan,
            border_dimmed: Color::Rgb(40, 50, 60),
            divider: Color::Rgb(45, 60, 75),

            // Brand & Header
            brand_bg: Color::Rgb(15, 45, 60),
            brand_fg: Color::Cyan,
            brand_version: Color::DarkGray,
            conn_connected: Color::Green,
            conn_offline: Color::Yellow,
            conn_text: Color::White,
            conn_cluster: Color::Rgb(180, 160, 255),
            conn_standalone: Color::Cyan,
            header_badge_bg: Color::Rgb(20, 30, 50),
            header_badge_fg: Color::Cyan,
            focus_right_bg: Color::Rgb(40, 30, 80),
            focus_right_fg: Color::Rgb(180, 160, 255),
            focus_stream_bg: Color::Rgb(20, 40, 60),
            focus_stream_fg: Color::Cyan,

            // Stream & Input
            prompt_symbol: Color::Cyan,
            prompt_input: Color::White,
            prompt_border_focused: Color::Cyan,
            prompt_border_unfocused: Color::DarkGray,
            stream_border_focused: Color::Cyan,
            stream_border_unfocused: Color::DarkGray,
            stream_border_dimmed: Color::Rgb(40, 50, 60),

            // Execution Cards
            cmd_direct_bg: Color::Rgb(20, 50, 60),
            cmd_direct_fg: Color::Cyan,
            cmd_node_bg: Color::Rgb(40, 30, 80),
            cmd_node_fg: Color::Rgb(180, 160, 255),
            cmd_broadcast_bg: Color::Rgb(60, 20, 70),
            cmd_broadcast_fg: Color::Rgb(255, 180, 255),
            cmd_name_native: Color::Cyan,
            cmd_name_macro: Color::Yellow,
            cmd_meta: Color::DarkGray,

            // Formatted Values
            val_status: Color::Rgb(16, 185, 129),
            val_integer: Color::Rgb(100, 200, 255),
            val_string: Color::White,
            val_key: Color::Green,
            val_info_section: Color::Cyan,
            val_node_header: Color::Rgb(180, 160, 255),
            val_error: Color::Red,
            val_nil: Color::DarkGray,
            val_tree_root: Color::Cyan,
            val_tree_tag: Color::Yellow,
            val_tree_branch: Color::DarkGray,

            // Tables
            table_border: Color::DarkGray,
            table_header: Color::Cyan,
            table_col1: Color::Green,
            table_col2: Color::Yellow,
            table_col_rest: Color::White,

            // JSON Highlight
            json_key: Color::Cyan,
            json_string: Color::Green,
            json_number: Color::Rgb(100, 200, 255),
            json_boolean: Color::Yellow,
            json_bracket: Color::DarkGray,
            json_colon: Color::DarkGray,

            // Telemetry Dashboard
            telemetry_label: Color::Rgb(165, 180, 195),
            telemetry_value: Color::White,
            telemetry_card_border: Color::Rgb(65, 80, 95),
            mem_bar_used: Color::Rgb(56, 189, 248),
            mem_bar_rss: Color::Rgb(251, 191, 36),
            mem_bar_free: Color::Rgb(45, 55, 72),
            cpu_waveform: Color::Rgb(56, 189, 248),
            status_healthy: Color::Green,
            status_warning: Color::Rgb(245, 158, 11),
            status_critical: Color::Rgb(239, 68, 68),

            // Cluster View
            cluster_border: Color::Cyan,
            shard_border: Color::Cyan,
            shard_master_title: Color::Green,
            shard_replica_title: Color::Rgb(147, 112, 219),
            shard_node_id: Color::Rgb(180, 160, 255),
            shard_slot_label: Color::Yellow,
            shard_slot_range: Color::Yellow,

            // Slowlog View
            slowlog_border: Color::Yellow,
            slowlog_cmd: Color::Yellow,
            slowlog_guidance_text: Color::White,

            // Autocomplete Popover
            auto_border: Color::Cyan,
            auto_item_selected_bg: Color::Rgb(25, 35, 55),
            auto_item_selected_fg: Color::White,
            auto_item_unselected_fg: Color::Rgb(220, 220, 220),
            auto_desc_fg: Color::DarkGray,
            auto_badge_macro_bg: Color::Rgb(50, 45, 20),
            auto_badge_macro_fg: Color::Yellow,
            auto_badge_node_bg: Color::Rgb(40, 30, 80),
            auto_badge_node_fg: Color::Rgb(180, 160, 255),
            auto_badge_cmd_bg: Color::Rgb(20, 50, 60),
            auto_badge_cmd_fg: Color::Cyan,
            auto_badge_sub_bg: Color::Rgb(20, 45, 55),
            auto_badge_sub_fg: Color::Rgb(100, 220, 255),
            auto_badge_arg_bg: Color::Rgb(45, 45, 25),
            auto_badge_arg_fg: Color::Rgb(255, 220, 120),

            // Popups & Modals
            help_border: Color::Cyan,
            help_brand_bg: Color::Rgb(15, 45, 60),
            help_brand_fg: Color::Cyan,
            help_tab_active: Color::Cyan,
            help_tab_inactive: Color::Rgb(180, 195, 210),
            help_tab_border: Color::Rgb(45, 60, 75),
            help_title_yellow: Color::Yellow,
            help_title_cyan: Color::Cyan,
            help_title_purple: Color::Rgb(180, 160, 255),
            help_title_green: Color::Green,
            help_title_red: Color::Rgb(239, 68, 68),

            // Safety Guard
            guard_l3_border: Color::Rgb(239, 68, 68),
            guard_l3_badge_bg: Color::Rgb(80, 20, 20),
            guard_l3_badge_fg: Color::Rgb(255, 100, 100),
            guard_l2_border: Color::Rgb(245, 158, 11),
            guard_l2_badge_bg: Color::Rgb(70, 50, 15),
            guard_l2_badge_fg: Color::Rgb(255, 200, 80),
            guard_cmd_title: Color::Rgb(180, 195, 210),
            guard_reason_text: Color::Rgb(240, 245, 250),
            guard_suggestion_title: Color::Rgb(16, 185, 129),
            guard_suggestion_text: Color::Rgb(215, 240, 225),
            guard_btn_confirm_bg: Color::Rgb(60, 20, 20),
            guard_btn_confirm_fg: Color::Rgb(255, 120, 120),
            guard_btn_cancel_bg: Color::Rgb(20, 45, 60),
            guard_btn_cancel_fg: Color::Cyan,

            // Footer
            footer_key_bg: Color::Rgb(30, 40, 60),
            footer_key_cyan: Color::Cyan,
            footer_key_yellow: Color::Yellow,
            footer_key_purple: Color::Rgb(180, 160, 255),
            footer_key_red: Color::Red,
            footer_label: Color::DarkGray,
        }
    }

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,

            // Base (High contrast on white background)
            text_primary: Color::Rgb(15, 23, 42),      // Deep Charcoal Slate
            text_secondary: Color::Rgb(51, 65, 85),    // Medium Slate
            text_muted: Color::Rgb(100, 116, 139),     // Slate Gray (Readable)
            text_dimmed: Color::Rgb(148, 163, 184),    // Soft Slate
            border: Color::Rgb(148, 163, 184),         // Slate-400
            border_focused: Color::Rgb(2, 132, 199),   // Cobalt Sky-600
            border_dimmed: Color::Rgb(203, 213, 225),  // Light Slate-300
            divider: Color::Rgb(203, 213, 225),

            // Brand & Header
            brand_bg: Color::Rgb(224, 242, 254),       // Sky-100
            brand_fg: Color::Rgb(3, 105, 161),         // Sky-700
            brand_version: Color::Rgb(100, 116, 139),
            conn_connected: Color::Rgb(22, 163, 74),   // Green-600
            conn_offline: Color::Rgb(202, 138, 4),     // Amber-600
            conn_text: Color::Rgb(15, 23, 42),
            conn_cluster: Color::Rgb(109, 40, 217),    // Violet-700
            conn_standalone: Color::Rgb(3, 105, 161),  // Sky-700
            header_badge_bg: Color::Rgb(241, 245, 249),// Slate-100
            header_badge_fg: Color::Rgb(2, 132, 199),
            focus_right_bg: Color::Rgb(237, 233, 254), // Violet-100
            focus_right_fg: Color::Rgb(109, 40, 217),
            focus_stream_bg: Color::Rgb(224, 242, 254),// Sky-100
            focus_stream_fg: Color::Rgb(3, 105, 161),

            // Stream & Input
            prompt_symbol: Color::Rgb(2, 132, 199),
            prompt_input: Color::Rgb(15, 23, 42),
            prompt_border_focused: Color::Rgb(2, 132, 199),
            prompt_border_unfocused: Color::Rgb(148, 163, 184),
            stream_border_focused: Color::Rgb(2, 132, 199),
            stream_border_unfocused: Color::Rgb(148, 163, 184),
            stream_border_dimmed: Color::Rgb(203, 213, 225),

            // Execution Cards
            cmd_direct_bg: Color::Rgb(224, 242, 254),  // Sky-100
            cmd_direct_fg: Color::Rgb(3, 105, 161),
            cmd_node_bg: Color::Rgb(237, 233, 254),    // Violet-100
            cmd_node_fg: Color::Rgb(109, 40, 217),
            cmd_broadcast_bg: Color::Rgb(250, 232, 255),// Fuchsia-100
            cmd_broadcast_fg: Color::Rgb(162, 28, 175),
            cmd_name_native: Color::Rgb(2, 132, 199),
            cmd_name_macro: Color::Rgb(180, 83, 9),    // Amber-700
            cmd_meta: Color::Rgb(100, 116, 139),

            // Formatted Values
            val_status: Color::Rgb(22, 163, 74),       // Emerald/Green-600
            val_integer: Color::Rgb(2, 132, 199),      // Sky-600
            val_string: Color::Rgb(15, 23, 42),
            val_key: Color::Rgb(21, 128, 61),          // Green-700
            val_info_section: Color::Rgb(3, 105, 161), // Sky-700
            val_node_header: Color::Rgb(109, 40, 217), // Violet-700
            val_error: Color::Rgb(220, 38, 38),        // Red-600
            val_nil: Color::Rgb(100, 116, 139),
            val_tree_root: Color::Rgb(3, 105, 161),
            val_tree_tag: Color::Rgb(180, 83, 9),
            val_tree_branch: Color::Rgb(148, 163, 184),

            // Tables
            table_border: Color::Rgb(148, 163, 184),
            table_header: Color::Rgb(3, 105, 161),
            table_col1: Color::Rgb(21, 128, 61),
            table_col2: Color::Rgb(180, 83, 9),
            table_col_rest: Color::Rgb(15, 23, 42),

            // JSON Highlight
            json_key: Color::Rgb(3, 105, 161),
            json_string: Color::Rgb(21, 128, 61),
            json_number: Color::Rgb(2, 132, 199),
            json_boolean: Color::Rgb(180, 83, 9),
            json_bracket: Color::Rgb(100, 116, 139),
            json_colon: Color::Rgb(100, 116, 139),

            // Telemetry Dashboard
            telemetry_label: Color::Rgb(71, 85, 105),   // Slate-600
            telemetry_value: Color::Rgb(15, 23, 42),
            telemetry_card_border: Color::Rgb(148, 163, 184),
            mem_bar_used: Color::Rgb(2, 132, 199),      // Sky-600
            mem_bar_rss: Color::Rgb(217, 119, 6),       // Amber-600
            mem_bar_free: Color::Rgb(226, 232, 240),    // Light Slate-200
            cpu_waveform: Color::Rgb(2, 132, 199),      // Sky-600
            status_healthy: Color::Rgb(22, 163, 74),
            status_warning: Color::Rgb(202, 138, 4),
            status_critical: Color::Rgb(220, 38, 38),

            // Cluster View
            cluster_border: Color::Rgb(2, 132, 199),
            shard_border: Color::Rgb(2, 132, 199),
            shard_master_title: Color::Rgb(21, 128, 61),
            shard_replica_title: Color::Rgb(124, 58, 237),
            shard_node_id: Color::Rgb(109, 40, 217),
            shard_slot_label: Color::Rgb(180, 83, 9),
            shard_slot_range: Color::Rgb(180, 83, 9),

            // Slowlog View
            slowlog_border: Color::Rgb(202, 138, 4),
            slowlog_cmd: Color::Rgb(180, 83, 9),
            slowlog_guidance_text: Color::Rgb(15, 23, 42),

            // Autocomplete Popover
            auto_border: Color::Rgb(2, 132, 199),
            auto_item_selected_bg: Color::Rgb(224, 231, 255), // Indigo-100
            auto_item_selected_fg: Color::Rgb(15, 23, 42),
            auto_item_unselected_fg: Color::Rgb(51, 65, 85),
            auto_desc_fg: Color::Rgb(100, 116, 139),
            auto_badge_macro_bg: Color::Rgb(254, 243, 199),   // Amber-100
            auto_badge_macro_fg: Color::Rgb(180, 83, 9),
            auto_badge_node_bg: Color::Rgb(237, 233, 254),    // Violet-100
            auto_badge_node_fg: Color::Rgb(109, 40, 217),
            auto_badge_cmd_bg: Color::Rgb(224, 242, 254),     // Sky-100
            auto_badge_cmd_fg: Color::Rgb(3, 105, 161),
            auto_badge_sub_bg: Color::Rgb(204, 251, 241),     // Teal-100
            auto_badge_sub_fg: Color::Rgb(15, 118, 110),
            auto_badge_arg_bg: Color::Rgb(254, 249, 195),     // Yellow-100
            auto_badge_arg_fg: Color::Rgb(161, 98, 7),

            // Popups & Modals
            help_border: Color::Rgb(2, 132, 199),
            help_brand_bg: Color::Rgb(224, 242, 254),
            help_brand_fg: Color::Rgb(3, 105, 161),
            help_tab_active: Color::Rgb(2, 132, 199),
            help_tab_inactive: Color::Rgb(100, 116, 139),
            help_tab_border: Color::Rgb(203, 213, 225),
            help_title_yellow: Color::Rgb(180, 83, 9),
            help_title_cyan: Color::Rgb(3, 105, 161),
            help_title_purple: Color::Rgb(109, 40, 217),
            help_title_green: Color::Rgb(21, 128, 61),
            help_title_red: Color::Rgb(220, 38, 38),

            // Safety Guard
            guard_l3_border: Color::Rgb(220, 38, 38),
            guard_l3_badge_bg: Color::Rgb(254, 226, 226),     // Red-100
            guard_l3_badge_fg: Color::Rgb(185, 28, 28),       // Red-700
            guard_l2_border: Color::Rgb(217, 119, 6),
            guard_l2_badge_bg: Color::Rgb(254, 243, 199),     // Amber-100
            guard_l2_badge_fg: Color::Rgb(180, 83, 9),        // Amber-700
            guard_cmd_title: Color::Rgb(71, 85, 105),
            guard_reason_text: Color::Rgb(15, 23, 42),
            guard_suggestion_title: Color::Rgb(21, 128, 61),
            guard_suggestion_text: Color::Rgb(22, 101, 52),
            guard_btn_confirm_bg: Color::Rgb(254, 226, 226),
            guard_btn_confirm_fg: Color::Rgb(185, 28, 28),
            guard_btn_cancel_bg: Color::Rgb(224, 242, 254),
            guard_btn_cancel_fg: Color::Rgb(3, 105, 161),

            // Footer
            footer_key_bg: Color::Rgb(241, 245, 249),
            footer_key_cyan: Color::Rgb(3, 105, 161),
            footer_key_yellow: Color::Rgb(180, 83, 9),
            footer_key_purple: Color::Rgb(109, 40, 217),
            footer_key_red: Color::Rgb(220, 38, 38),
            footer_label: Color::Rgb(100, 116, 139),
        }
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }
}
