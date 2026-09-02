use xedis_tui::app::App;
use xedis_tui::config::AppConfig;
use xedis_tui::core::autocomplete::{AutocompleteEngine, SuggestionKind};
use xedis_tui::core::macro_engine::MacroEngine;
use xedis_tui::core::router::CommandRouter;
use xedis_tui::ui::theme::{ThemeMode, ThemePalette};

#[test]
fn test_theme_mode_toggle_and_names() {
    let dark = ThemeMode::Dark;
    assert_eq!(dark.name(), "Dark");
    let toggled = dark.toggle();
    assert_eq!(toggled, ThemeMode::Light);
    assert_eq!(toggled.name(), "Light");

    let toggled_back = toggled.toggle();
    assert_eq!(toggled_back, ThemeMode::Dark);
}

#[test]
fn test_theme_mode_serde_deserialization() {
    let json_dark = "\"dark\"";
    let mode_dark: ThemeMode = serde_json::from_str(json_dark).unwrap();
    assert_eq!(mode_dark, ThemeMode::Dark);

    let json_light = "\"light\"";
    let mode_light: ThemeMode = serde_json::from_str(json_light).unwrap();
    assert_eq!(mode_light, ThemeMode::Light);
}

#[test]
fn test_theme_palette_contrast_and_construction() {
    let dark_palette = ThemePalette::dark();
    assert_eq!(dark_palette.mode, ThemeMode::Dark);

    let light_palette = ThemePalette::light();
    assert_eq!(light_palette.mode, ThemeMode::Light);

    // Verify Light palette has dark text for legibility on white backgrounds
    assert_ne!(light_palette.text_primary, ratatui::style::Color::White);
    assert_ne!(light_palette.prompt_input, ratatui::style::Color::White);

    // Verify from_mode selector
    let p_dark = ThemePalette::from_mode(ThemeMode::Dark);
    assert_eq!(p_dark.mode, ThemeMode::Dark);
    let p_light = ThemePalette::from_mode(ThemeMode::Light);
    assert_eq!(p_light.mode, ThemeMode::Light);
}

#[tokio::test]
async fn test_app_theme_command_switching() {
    let mut config = AppConfig::default();
    config.theme = ThemeMode::Dark;

    let mut app = App::new(config).await;
    assert_eq!(app.config.theme, ThemeMode::Dark);

    // 1. Switch to light using /theme light
    let parsed_light = CommandRouter::parse("/theme light").unwrap();
    app.execute_parsed_command(parsed_light).await;
    assert_eq!(app.config.theme, ThemeMode::Light);
    assert_eq!(app.records.last().unwrap().command, "/theme light");

    // 2. Switch to dark using /theme dark
    let parsed_dark = CommandRouter::parse("/theme dark").unwrap();
    app.execute_parsed_command(parsed_dark).await;
    assert_eq!(app.config.theme, ThemeMode::Dark);

    // 3. Toggle using /theme
    let parsed_toggle = CommandRouter::parse("/theme").unwrap();
    app.execute_parsed_command(parsed_toggle).await;
    assert_eq!(app.config.theme, ThemeMode::Light);

    // 4. Toggle using /theme toggle
    let parsed_toggle_explicit = CommandRouter::parse("/theme toggle").unwrap();
    app.execute_parsed_command(parsed_toggle_explicit).await;
    assert_eq!(app.config.theme, ThemeMode::Dark);
}

#[test]
fn test_autocomplete_theme_subcommands() {
    // Typing "/theme " should suggest subcommands dark, light, toggle
    let (items, _) = AutocompleteEngine::get_suggestions("/theme ", 7, &[]);
    assert!(!items.is_empty(), "Should suggest subcommands for /theme");

    let subcmds: Vec<String> = items
        .iter()
        .filter(|it| it.kind == SuggestionKind::Subcommand)
        .map(|it| it.completion_text.trim().to_string())
        .collect();

    assert!(subcmds.contains(&"dark".to_string()), "Should contain dark subcommand");
    assert!(subcmds.contains(&"light".to_string()), "Should contain light subcommand");
    assert!(subcmds.contains(&"toggle".to_string()), "Should contain toggle subcommand");
}

#[test]
fn test_autocomplete_macro_list_includes_theme() {
    let (items, _) = AutocompleteEngine::get_suggestions("/", 1, &[]);
    let has_theme = items.iter().any(|it| it.completion_text.trim() == "/theme");
    assert!(has_theme, "Macro autocomplete should include /theme");
}

#[test]
fn test_macro_engine_format_theme_and_settings() {
    let res = MacroEngine::format_theme_result("Dark", "Light");
    match res {
        xedis_tui::backend::formatter::FormattedValue::Table { headers, rows } => {
            assert_eq!(headers, vec!["Setting", "Current Theme", "Status", "Available Themes"]);
            assert!(rows.iter().any(|r| r[0] == "UI Color Theme" && r[1] == "Light"));
        }
        _ => panic!("Expected Table formatted value"),
    }

    let settings = MacroEngine::format_settings("127.0.0.1", 6379, false, "Balanced", "Light", 200, false);
    match settings {
        xedis_tui::backend::formatter::FormattedValue::Table { rows, .. } => {
            assert!(rows.iter().any(|r| r[0] == "UI Color Theme" && r[1] == "Light"));
        }
        _ => panic!("Expected Table formatted value"),
    }
}
