use five_engine::entry::Entry;
use five_gtk::model::kinds::{extension, kind_of};
use five_gtk::model::sorting::{SortKey, natural_cmp, sort_items};

fn entry(name: &str, is_dir: bool, size: u64) -> Entry {
    Entry { name: name.into(), is_dir, size, modified: 0.0, is_link: false, mime: None, path: name.into(), children: None }
}

#[test]
fn kinds_follow_extensions() {
    assert_eq!(kind_of(&entry("photo.JPG", false, 1)), "image");
    assert_eq!(kind_of(&entry("main.rs", false, 1)), "code");
    assert_eq!(kind_of(&entry("notes", false, 1)), "other");
    assert_eq!(kind_of(&entry("dossier", true, 0)), "dir");
    assert_eq!(extension("a.tar.GZ"), "gz");
}

#[test]
fn sorting_puts_folders_first_and_numbers_by_value() {
    let list = vec![entry("f10", false, 1), entry("F2", false, 1), entry("zdir", true, 0), entry("été", false, 1), entry("eta", false, 1)];
    let names: Vec<String> = sort_items(&list, SortKey::Name, true).into_iter().map(|e| e.name).collect();
    assert_eq!(names, ["zdir", "eta", "été", "F2", "f10"]);
    assert_eq!(natural_cmp("a2", "a10"), std::cmp::Ordering::Less);
}

#[test]
fn sorting_by_size_can_descend() {
    let list = vec![entry("a", false, 5), entry("b", false, 9)];
    assert_eq!(sort_items(&list, SortKey::Size, false)[0].name, "b");
}
