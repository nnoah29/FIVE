mod support;

use five_engine::fuzzy::fuzzy_score;
use five_engine::listing::list_dir;
use five_engine::search::search_dir;
use support::Sandbox;

#[test]
fn list_dir_hides_dotfiles_unless_asked() {
    let sandbox = Sandbox::new();
    sandbox.write("vu.txt", "x");
    sandbox.write(".cache", "x");
    let root = sandbox.root();
    assert_eq!(list_dir(&root, &sandbox.dir, false).unwrap().len(), 1);
    assert_eq!(list_dir(&root, &sandbox.dir, true).unwrap().len(), 2);
}

#[test]
fn list_dir_counts_children_of_folders() {
    let sandbox = Sandbox::new();
    sandbox.write("d/a", "x");
    sandbox.write("d/b", "x");
    sandbox.write("d/.c", "x");
    let items = list_dir(&sandbox.root(), &sandbox.dir, false).unwrap();
    assert_eq!(items[0].children, Some(Some(2)));
    assert_eq!(items[0].path, "d");
}

#[test]
fn entries_carry_size_and_mime() {
    let sandbox = Sandbox::new();
    sandbox.write("note.md", "12345");
    let item = &list_dir(&sandbox.root(), &sandbox.dir, false).unwrap()[0];
    assert_eq!((item.size, item.is_dir), (5, false));
    assert_eq!(item.mime.as_deref(), Some("text/markdown"));
}

#[test]
fn search_finds_names_recursively_and_ignores_hidden() {
    let sandbox = Sandbox::new();
    sandbox.write("a/b/rapport.txt", "x");
    sandbox.write(".secret/rapport.txt", "x");
    let found = search_dir(&sandbox.root(), &sandbox.dir, "RAPPORT", false, false);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, "a/b/rapport.txt");
}

#[test]
fn fuzzy_matches_subsequences_and_ranks_substrings_first() {
    assert_eq!(fuzzy_score("rpt", "rapport"), Some(1016));
    assert_eq!(fuzzy_score("port", "rapport"), Some(3));
    assert_eq!(fuzzy_score("zz", "rapport"), None);
}
