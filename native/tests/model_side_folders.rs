use five_gtk::model::settings::Settings;
use five_gtk::model::side_folders::{SideFolder, folders_to_json, move_folder, parse_folders, pin_folder, shown_label};
use serde_json::json;

fn folder(label: &str, path: &str) -> SideFolder {
    SideFolder { label: label.into(), path: path.into(), icon: "folder".into() }
}

#[test]
fn absent_list_means_automatic_mode() {
    assert_eq!(parse_folders(&json!(null)), None);
    assert_eq!(Settings::from_json(&json!({})).side_folders, None);
}

#[test]
fn list_round_trips_through_the_settings_file() {
    let settings = Settings { side_folders: Some(vec![folder("Projets", "/home/a/p"), folder("", "/home/a/q")]), ..Settings::default() };
    assert_eq!(Settings::from_json(&settings.to_json()).side_folders, settings.side_folders);
}

#[test]
fn entries_without_path_are_dropped_and_unknown_icons_become_folders() {
    let value = json!([{"label": "A", "path": "/a", "icon": "pirate"}, {"label": "B", "path": ""}]);
    assert_eq!(parse_folders(&value), Some(vec![folder("A", "/a")]));
    assert_eq!(folders_to_json(&[folder("A", "/a")])[0]["icon"], "folder");
}

#[test]
fn folders_move_within_the_list_only() {
    let mut list = vec![folder("1", "/1"), folder("2", "/2"), folder("3", "/3")];
    move_folder(&mut list, 2, -1);
    move_folder(&mut list, 0, -1);
    move_folder(&mut list, 2, 1);
    assert_eq!(list.iter().map(|f| f.label.as_str()).collect::<Vec<_>>(), ["1", "3", "2"]);
}

#[test]
fn pinning_twice_is_refused_and_empty_names_fall_back_to_the_folder_name() {
    let mut list = Vec::new();
    assert!(pin_folder(&mut list, "/home/a/Projets/"));
    assert!(!pin_folder(&mut list, "/home/a/Projets/"));
    assert_eq!(shown_label(&list[0]), "Projets");
    assert_eq!(shown_label(&folder("Boulot", "/x")), "Boulot");
}
