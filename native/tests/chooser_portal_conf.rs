use five_gtk::chooser::portal_conf::{KEY, is_enabled, set_chooser};

#[test]
fn enabling_on_an_empty_file_creates_the_section() {
    let text = set_chooser("", true);
    assert_eq!(text, format!("[preferred]\n{KEY}=five;gtk\n"));
    assert!(is_enabled(&text));
}

#[test]
fn enabling_keeps_other_lines_and_sections() {
    let before = "[preferred]\ndefault=hyprland;gtk\norg.freedesktop.impl.portal.Screenshot=hyprland\n\n[other]\nx=1\n";
    let after = set_chooser(before, true);
    assert!(after.contains("default=hyprland;gtk") && after.contains("Screenshot=hyprland") && after.contains("[other]\nx=1"));
    assert!(is_enabled(&after) && after.matches(KEY).count() == 1);
}

#[test]
fn enabling_twice_changes_nothing_and_replaces_another_choice() {
    let once = set_chooser("[preferred]\n", true);
    assert_eq!(set_chooser(&once, true), once);
    let replaced = set_chooser(&format!("[preferred]\n{KEY}=gtk\n"), true);
    assert_eq!(replaced, format!("[preferred]\n{KEY}=five;gtk\n"));
}

#[test]
fn disabling_removes_only_our_line() {
    let on = set_chooser("[preferred]\ndefault=gtk\n", true);
    let off = set_chooser(&on, false);
    assert_eq!(off, "[preferred]\ndefault=gtk\n");
    let other = format!("[preferred]\n{KEY}=kde\n");
    assert_eq!(set_chooser(&other, false), other);
}

#[test]
fn a_key_outside_the_preferred_section_does_not_count() {
    assert!(!is_enabled(&format!("[other]\n{KEY}=five\n")));
}
