mod support_dir;

use five_gtk::chooser::listing::{read_rows, selectable};
use five_gtk::chooser::request::{Filter, FilterRule, Mode};

fn names(rows: &[five_gtk::chooser::listing::Row]) -> Vec<&str> {
    rows.iter().map(|r| r.name.as_str()).collect()
}

#[test]
fn folders_come_first_then_files_by_name_without_hidden_ones() {
    let dir = support_dir::sample();
    let rows = read_rows(&dir, false).unwrap();
    assert_eq!(names(&rows), vec!["alpha", "Beta", "a.txt", "b.png", "C.md"]);
    assert!(names(&read_rows(&dir, true).unwrap()).contains(&".hidden"));
}

#[test]
fn folder_mode_keeps_only_folders() {
    let dir = support_dir::sample();
    let rows = selectable(read_rows(&dir, false).unwrap(), Mode::Folder, None);
    assert_eq!(names(&rows), vec!["alpha", "Beta"]);
}

#[test]
fn a_filter_hides_other_files_but_never_folders() {
    let dir = support_dir::sample();
    let filter = Filter { name: "Images".into(), rules: vec![FilterRule { kind: 0, pattern: "*.png".into() }] };
    let rows = selectable(read_rows(&dir, false).unwrap(), Mode::Open, Some(&filter));
    assert_eq!(names(&rows), vec!["alpha", "Beta", "b.png"]);
}
