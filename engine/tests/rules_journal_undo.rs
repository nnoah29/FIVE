mod support;

use std::fs;

use five_engine::jobs::registry::Jobs;
use five_engine::rules::exec_pass::run_plan;
use five_engine::rules::journal::{append, journal_path, read_recent};
use five_engine::rules::run_types::RunEnv;
use five_engine::rules::undo::undo_last;
use serde_json::json;
use support::Sandbox;
use support::rules_run::{any_file, plan};

fn run_and_log(sandbox: &Sandbox, run_id: &str, actions: serde_json::Value, path: &str) {
    let (root, jobs) = (sandbox.root(), Jobs::new());
    let results = run_plan(&RunEnv { root: &root, jobs: &jobs }, &plan(&any_file(actions), path));
    append(&journal_path(&sandbox.dir), run_id, &results).unwrap();
}

#[test]
fn the_journal_keeps_one_line_per_action_newest_first_without_file_content() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a b.txt", "SECRET");
    run_and_log(&sandbox, "r1", json!([{"type": "slugify"}, {"type": "add_prefix", "text": "x-"}]), "Dl/a b.txt");
    let lines = read_recent(&journal_path(&sandbox.dir), 10);
    assert_eq!((lines.len(), lines[0]["action"].as_str(), lines[0]["run"].as_str()), (2, Some("add_prefix"), Some("r1")));
    assert!(!fs::read_to_string(journal_path(&sandbox.dir)).unwrap().contains("SECRET"));
}

#[test]
fn undo_last_restores_renames_moves_and_trash_in_reverse_order() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a b.txt", "x");
    run_and_log(&sandbox, "r1", json!([{"type": "slugify"}, {"type": "move_to", "folder": "Docs"}]), "Dl/a b.txt");
    assert!(sandbox.dir.join("Docs/a-b.txt").exists());
    let report = undo_last(&sandbox.root(), &journal_path(&sandbox.dir)).unwrap();
    assert_eq!((report.run.as_str(), report.undone, report.failed.len()), ("r1", 2, 0));
    assert!(sandbox.dir.join("Dl/a b.txt").is_file() && !sandbox.dir.join("Docs/a-b.txt").exists());
    assert!(undo_last(&sandbox.root(), &journal_path(&sandbox.dir)).is_none());
}

#[test]
fn trash_and_created_archives_are_undone_too() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "x");
    run_and_log(&sandbox, "r1", json!([{"type": "zip_folder_then_trash"}]), "Dl/a.txt");
    assert!(!sandbox.dir.join("Dl/a.txt").exists() && sandbox.dir.join("Dl/a.txt.zip").exists());
    undo_last(&sandbox.root(), &journal_path(&sandbox.dir)).unwrap();
    assert!(sandbox.dir.join("Dl/a.txt").is_file() && !sandbox.dir.join("Dl/a.txt.zip").exists());
}

#[test]
fn undo_never_overwrites_something_that_changed_since() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a b.txt", "x");
    run_and_log(&sandbox, "r1", json!([{"type": "slugify"}]), "Dl/a b.txt");
    sandbox.write("Dl/a b.txt", "autre");
    let report = undo_last(&sandbox.root(), &journal_path(&sandbox.dir)).unwrap();
    assert_eq!((report.undone, report.failed.len()), (0, 1));
    assert_eq!(fs::read_to_string(sandbox.dir.join("Dl/a b.txt")).unwrap(), "autre");
}

#[test]
fn a_tampered_journal_cannot_make_the_undo_touch_files_outside_the_root() {
    let sandbox = Sandbox::new();
    let outside = std::env::temp_dir().join(format!("five-hors-{}", std::process::id()));
    fs::write(&outside, "à ne pas toucher").unwrap();
    let line = json!({"run": "x", "outcome": "done", "undo": [{"kind": "trash_created", "path": outside.to_string_lossy()}]});
    fs::write(journal_path(&sandbox.dir), format!("{line}\n")).unwrap();
    let report = undo_last(&sandbox.root(), &journal_path(&sandbox.dir)).unwrap();
    assert_eq!((report.undone, report.failed.len()), (0, 1));
    assert!(outside.exists());
    fs::remove_file(outside).unwrap();
}
