use five_gtk::model::accent::{is_hex, on_accent};
use five_gtk::model::format::{format_clock, format_size};
use five_gtk::model::numbers::group_digits;
use five_gtk::model::paths::{in_trash, join_path, name_parts, parent_of};

#[test]
fn sizes_use_french_units_and_decimals() {
    assert_eq!(format_size(999, false), "999 o");
    assert_eq!(format_size(1500, false), "1,5 Ko");
    assert_eq!(format_size(2_000_000, false), "2 Mo");
    assert_eq!(format_size(35_000_000, false), "35 Mo");
    assert_eq!(format_size(1536, true), "1,5 Kio");
    assert_eq!(format_size(493_000_000_000, false), "493 Go");
    assert_eq!(format_size(100_000_000_000, false), "100 Go");
    assert_eq!(format_size(430_000_000_000, false), "430 Go");
    assert_eq!(format_size(10_000, false), "10 Ko");
    assert_eq!(format_size(1_000_000, false), "1 Mo");
}

#[test]
fn numbers_group_thousands_with_a_narrow_space() {
    assert_eq!(group_digits(1234567), "1\u{202f}234\u{202f}567");
    assert_eq!(group_digits(12), "12");
}

#[test]
fn clocks_show_minutes_or_hours() {
    assert_eq!(format_clock(65.0), "1:05");
    assert_eq!(format_clock(3725.0), "1:02:05");
    assert_eq!(format_clock(f64::NAN), "—");
}

#[test]
fn paths_split_join_and_detect_the_trash() {
    assert_eq!(join_path("", "a"), "a");
    assert_eq!(join_path("a/b", "c"), "a/b/c");
    assert_eq!(parent_of("a/b/c"), "a/b");
    assert_eq!(parent_of("a"), "");
    assert!(in_trash(".local/share/Trash/files/x"));
    assert!(!in_trash(".local/share/Trashy"));
    assert_eq!(name_parts("rapport.final.txt"), ("rapport.final".into(), ".txt".into()));
    assert_eq!(name_parts(".bashrc"), (".bashrc".into(), String::new()));
}

#[test]
fn accent_validation_and_contrast() {
    assert!(is_hex("#e8542b") && is_hex("#abc") && !is_hex("e8542b") && !is_hex("#abcd"));
    assert_eq!(on_accent("#ffffff"), "#1b1b1b");
    assert_eq!(on_accent("#101010"), "#ffffff");
}
