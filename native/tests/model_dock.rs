use five_gtk::model::dock::{parse_dock, pin_dock, unpin_dock};
use five_gtk::model::settings::Settings;
use serde_json::json;

#[test]
fn absent_or_bad_list_gives_an_empty_dock() {
    assert!(parse_dock(&json!(null)).is_empty());
    assert_eq!(parse_dock(&json!(["/a", "", 3, "/b"])), vec!["/a", "/b"]);
}

#[test]
fn pin_and_unpin_keep_order_and_refuse_duplicates() {
    let mut list = vec![];
    assert!(pin_dock(&mut list, "/a"));
    assert!(pin_dock(&mut list, "/b"));
    assert!(!pin_dock(&mut list, "/a"));
    assert!(unpin_dock(&mut list, "/a"));
    assert!(!unpin_dock(&mut list, "/a"));
    assert_eq!(list, vec!["/b"]);
}

#[test]
fn dock_round_trips_through_the_settings_file() {
    let settings = Settings { dock: vec!["/home/a/x.pdf".into(), "/home/a/p".into()], ..Settings::default() };
    assert_eq!(Settings::from_json(&settings.to_json()).dock, settings.dock);
}
