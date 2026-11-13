mod support;

use std::fs;
use std::os::unix::fs::symlink;

use five_engine::names::clean_name;
use support::Sandbox;

#[test]
fn resolve_accepts_paths_inside_the_root() {
    let sandbox = Sandbox::new();
    sandbox.write("a/b.txt", "x");
    let full = sandbox.root().resolve("a/b.txt").unwrap();
    assert_eq!(full, sandbox.dir.join("a/b.txt"));
}

#[test]
fn resolve_refuses_parent_escapes() {
    let sandbox = Sandbox::new();
    let refus = sandbox.root().resolve("../..").unwrap_err();
    assert_eq!(refus.code, 403);
}

#[test]
fn resolve_refuses_symlinks_pointing_outside() {
    let sandbox = Sandbox::new();
    symlink("/etc", sandbox.dir.join("evasion")).unwrap();
    let refus = sandbox.root().resolve("evasion/passwd").unwrap_err();
    assert_eq!(refus.code, 403);
}

#[test]
fn resolve_entry_does_not_follow_the_last_link() {
    let sandbox = Sandbox::new();
    symlink("/etc", sandbox.dir.join("lien")).unwrap();
    let full = sandbox.root().resolve_entry("lien").unwrap();
    assert_eq!(full, sandbox.dir.join("lien"));
}

#[test]
fn resolve_entry_refuses_root_and_missing_items() {
    let sandbox = Sandbox::new();
    fs::create_dir(sandbox.dir.join("d")).unwrap();
    assert_eq!(sandbox.root().resolve_entry("").unwrap_err().code, 400);
    assert_eq!(sandbox.root().resolve_entry("d/absent").unwrap_err().code, 404);
}

#[test]
fn clean_name_rejects_unsafe_names() {
    for bad in ["", " ", ".", "..", "a/b", "a\0b"] {
        assert_eq!(clean_name(bad).unwrap_err().code, 400, "{bad:?}");
    }
    assert_eq!(clean_name("  ok.txt ").unwrap(), "ok.txt");
}
