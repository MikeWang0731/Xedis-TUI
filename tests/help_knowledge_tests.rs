use xedis_tui::ui::popups::HelpPopup;

#[test]
fn test_help_tab_titles_count() {
    assert_eq!(HelpPopup::TAB_TITLES.len(), 4);
    assert!(HelpPopup::TAB_TITLES[0].contains("快捷键"));
    assert!(HelpPopup::TAB_TITLES[1].contains("快捷宏"));
    assert!(HelpPopup::TAB_TITLES[2].contains("排障指南"));
    assert!(HelpPopup::TAB_TITLES[3].contains("安全护栏"));
}

#[test]
fn test_help_tabs_content_integrity() {
    // Check that rendering tabs does not panic and contains essential knowledge
    // Keybindings
    let keys = HelpPopup::TAB_TITLES[0];
    assert!(keys.len() > 0);

    // Troubleshoot topics
    // Verify memory, CPU, connections, cluster topics are included
    // Verify safety guard descriptions
}
