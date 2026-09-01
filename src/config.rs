use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LayoutPreset {
    #[default]
    Balanced, // 58% : 42%
    Focus,    // 75% : 25%
    Monitor,  // 35% : 65%
    Zen,      // 100% : 0%
}

impl LayoutPreset {
    pub fn next(&self) -> Self {
        match self {
            LayoutPreset::Balanced => LayoutPreset::Focus,
            LayoutPreset::Focus => LayoutPreset::Monitor,
            LayoutPreset::Monitor => LayoutPreset::Zen,
            LayoutPreset::Zen => LayoutPreset::Balanced,
        }
    }

    pub fn split_ratio(&self) -> u16 {
        match self {
            LayoutPreset::Balanced => 58,
            LayoutPreset::Focus => 75,
            LayoutPreset::Monitor => 35,
            LayoutPreset::Zen => 100,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LayoutPreset::Balanced => "Balanced (58%)",
            LayoutPreset::Focus => "Focus (75%)",
            LayoutPreset::Monitor => "Monitor (35%)",
            LayoutPreset::Zen => "Zen (100%)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub cluster_mode: bool,
    #[serde(default)]
    pub default_layout: LayoutPreset,
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_max_stream_records")]
    pub max_stream_records: usize,
    #[serde(default = "default_safety_guard_enabled")]
    pub safety_guard_enabled: bool,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    6379
}

fn default_poll_interval_ms() -> u64 {
    1000
}

fn default_max_stream_records() -> usize {
    500
}

fn default_safety_guard_enabled() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            password: None,
            cluster_mode: false,
            default_layout: LayoutPreset::Balanced,
            poll_interval_ms: default_poll_interval_ms(),
            max_stream_records: default_max_stream_records(),
            safety_guard_enabled: default_safety_guard_enabled(),
        }
    }
}

impl AppConfig {
    pub fn get_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("xedis");
            p.push("config.toml");
            p
        })
    }

    pub fn load() -> Self {
        if let Some(path) = Self::get_config_path() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        AppConfig::default()
    }
}
