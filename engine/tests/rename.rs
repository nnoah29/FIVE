mod support;

use std::fs;

use five_engine::rename::rename_many;
use support::Sandbox;

fn pair(from: &str, to: &str) -> (String, String) {
    (from.to_string(), to.to_string())
}

#[test]
fn rename_changes_the_name() {
    let sandbox = Sandbox::new();
    sandbox.write("a.txt", "A");
    assert_eq!(rename_many(&sandbox.root(), &[pair("a.txt", "b.txt")]).unwrap(), 1);
    assert_eq!(fs::read_to_string(sandbox.dir.join("b.txt")).unwrap(), "A");
}

#[test]
fn rename_supports_swaps_without_losing_data() {
    let sandbox = Sandbox::new();
    sandbox.write("a", "A");
    sandbox.write("b", "B");
    rename_many(&sandbox.root(), &[pair("a", "b"), pair("b", "a")]).unwrap();
    assert_eq!(fs::read_to_string(sandbox.dir.join("a")).unwrap(), "B");
    assert_eq!(fs::read_to_string(sandbox.dir.join("b")).unwrap(), "A");
}

#[test]
fn rename_never_overwrites_and_changes_nothing_on_conflict() {
    let sandbox = Sandbox::new();
    sandbox.write("a", "A");
    sandbox.write("b", "B");
    sandbox.write("c", "C");
    let refus = rename_many(&sandbox.root(), &[pair("a", "z"), pair("b", "c")]).unwrap_err();
    assert_eq!(refus.code, 409);
    assert!(sandbox.dir.join("a").exists() && !sandbox.dir.join("z").exists());
}

#[test]
fn rename_refuses_duplicate_targets_and_bad_names() {
    let sandbox = Sandbox::new();
    sandbox.write("a", "A");
    sandbox.write("b", "B");
    let root = sandbox.root();
    assert_eq!(rename_many(&root, &[pair("a", "x"), pair("b", "x")]).unwrap_err().code, 409);
    assert_eq!(rename_many(&root, &[pair("a", "../x")]).unwrap_err().code, 400);
}

#[test]
fn rename_with_the_same_name_counts_zero() {
    let sandbox = Sandbox::new();
    sandbox.write("a", "A");
    assert_eq!(rename_many(&sandbox.root(), &[pair("a", "a")]).unwrap(), 0);
}
