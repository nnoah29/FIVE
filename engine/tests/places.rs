mod support;

use std::fs;

use five_engine::places::existing_places;
use five_engine::views::{favorite_items, recent_items};
use support::Sandbox;

fn labels(places: &[serde_json::Value]) -> Vec<(String, String)> {
    places.iter().map(|p| (p["id"].as_str().unwrap().into(), p["path"].as_str().unwrap().into())).collect()
}

#[test]
fn places_follow_xdg_dirs_and_skip_missing_folders() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.dir.join("Mes Docs")).unwrap();
    fs::create_dir_all(sandbox.dir.join("Pictures")).unwrap();
    sandbox.write(".config/user-dirs.dirs", "XDG_DOCUMENTS_DIR=\"$HOME/Mes Docs\"\nXDG_VIDEOS_DIR=\"$HOME/Absent\"\n");
    let places = labels(&existing_places(&sandbox.root()));
    assert_eq!(places, [("docs".into(), "Mes Docs".into()), ("pictures".into(), "Pictures".into())]);
}

#[test]
fn places_list_the_trash_first_when_it_exists() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.dir.join(".local/share/Trash/files")).unwrap();
    let places = labels(&existing_places(&sandbox.root()));
    assert_eq!(places[0], ("trash".to_string(), ".local/share/Trash/files".to_string()));
}

#[test]
fn recent_items_keep_local_files_under_the_root_newest_first() {
    let sandbox = Sandbox::new();
    sandbox.write("vieux é.txt", "x");
    sandbox.write("neuf.txt", "x");
    let dir = sandbox.dir.display();
    let entry = |name: &str, date: &str| format!("<bookmark href=\"{name}\" modified=\"{date}\"/>");
    let xml = [
        entry(&format!("file://{dir}/vieux%20%C3%A9.txt"), "2026-01-01T00:00:00Z"),
        entry(&format!("file://{dir}/neuf.txt"), "2026-05-01T00:00:00Z"),
        entry("file:///etc/passwd", "2026-09-01T00:00:00Z"),
        entry("https://example.org/", "2026-10-01T00:00:00Z"),
    ]
    .concat();
    sandbox.write(".local/share/recently-used.xbel", &xml);
    let names: Vec<String> = recent_items(&sandbox.root()).into_iter().map(|e| e.name).collect();
    assert_eq!(names, ["neuf.txt", "vieux é.txt"]);
}

#[test]
fn favorites_read_gtk_bookmarks_without_duplicates() {
    let sandbox = Sandbox::new();
    fs::create_dir(sandbox.dir.join("Projets")).unwrap();
    let uri = format!("file://{}/Projets", sandbox.dir.display());
    sandbox.write(".config/gtk-3.0/bookmarks", &format!("{uri} Mes projets\nfile:///inexistant\n"));
    sandbox.write(".gtk-bookmarks", &format!("{uri}\n"));
    let names: Vec<String> = favorite_items(&sandbox.root()).into_iter().map(|e| e.name).collect();
    assert_eq!(names, ["Projets"]);
}
