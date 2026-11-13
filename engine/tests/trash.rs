mod support;

use std::fs;

use five_engine::delete::{Deleted, delete_entry};
use five_engine::folders::make_folder;
use support::Sandbox;

#[test]
fn delete_moves_to_the_xdg_trash_with_an_info_file() {
    let sandbox = Sandbox::new();
    sandbox.write("doc é.txt", "x");
    let mode = delete_entry(&sandbox.root(), "doc é.txt").unwrap();
    assert_eq!(mode, Deleted::Trash);
    assert!(sandbox.dir.join(".local/share/Trash/files/doc é.txt").exists());
    let info = fs::read_to_string(sandbox.dir.join(".local/share/Trash/info/doc é.txt.trashinfo")).unwrap();
    assert!(info.starts_with("[Trash Info]\nPath="));
    assert!(info.contains("doc%20%C3%A9.txt\nDeletionDate=20"));
}

#[test]
fn delete_keeps_both_files_when_names_collide() {
    let sandbox = Sandbox::new();
    for _ in 0..2 {
        sandbox.write("a.txt", "x");
        delete_entry(&sandbox.root(), "a.txt").unwrap();
    }
    assert!(sandbox.dir.join(".local/share/Trash/files/a.txt").exists());
    assert!(sandbox.dir.join(".local/share/Trash/files/a.txt.2").exists());
}

#[test]
fn delete_from_the_trash_is_definitive() {
    let sandbox = Sandbox::new();
    sandbox.write("a.txt", "x");
    delete_entry(&sandbox.root(), "a.txt").unwrap();
    let mode = delete_entry(&sandbox.root(), ".local/share/Trash/files/a.txt").unwrap();
    assert_eq!(mode, Deleted::Definitive);
    assert!(!sandbox.dir.join(".local/share/Trash/files/a.txt").exists());
    assert!(!sandbox.dir.join(".local/share/Trash/info/a.txt.trashinfo").exists());
}

#[test]
fn delete_refuses_the_root_and_folders_holding_the_trash() {
    let sandbox = Sandbox::new();
    sandbox.write("a.txt", "x");
    delete_entry(&sandbox.root(), "a.txt").unwrap();
    for rel in ["", ".local", ".local/share", ".local/share/Trash", ".local/share/Trash/files"] {
        assert!(delete_entry(&sandbox.root(), rel).is_err(), "{rel:?}");
    }
    assert!(sandbox.dir.join(".local/share/Trash/files").exists());
}

#[test]
fn delete_removes_a_symlink_not_its_target() {
    let sandbox = Sandbox::new();
    let target = sandbox.write("cible.txt", "x");
    std::os::unix::fs::symlink(&target, sandbox.dir.join("lien")).unwrap();
    delete_entry(&sandbox.root(), "lien").unwrap();
    assert!(target.exists());
}

#[test]
fn make_folder_creates_once_and_refuses_the_trash() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    assert_eq!(make_folder(&root, "", "Neuf").unwrap(), "Neuf");
    assert_eq!(make_folder(&root, "", "Neuf").unwrap_err().code, 409);
    assert_eq!(make_folder(&root, "", "a/b").unwrap_err().code, 400);
    sandbox.write("z", "x");
    delete_entry(&root, "z").unwrap();
    assert_eq!(make_folder(&root, ".local/share/Trash/files", "d").unwrap_err().code, 400);
}
