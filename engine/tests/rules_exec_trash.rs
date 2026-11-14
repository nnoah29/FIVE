mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;

use five_engine::rules::run_types::Outcome;
use serde_json::json;
use support::Sandbox;
use support::rules_run::{any_file, run};

#[test]
fn trash_moves_to_the_xdg_trash_and_records_how_to_restore() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "x");
    let results = run(&sandbox, &any_file(json!([{"type": "trash"}])), "Dl/a.txt");
    assert_eq!(results[0].outcome, Outcome::Done);
    assert!(!sandbox.dir.join("Dl/a.txt").exists() && sandbox.dir.join(".local/share/Trash/files/a.txt").is_file());
    assert_eq!(results[0].undo.len(), 1);
}

#[test]
fn definitive_deletion_only_happens_in_the_trash_and_for_old_items() {
    let sandbox = Sandbox::new();
    sandbox.write(".local/share/Trash/files/vieux.txt", "x");
    sandbox.write("Dl/a.txt", "x");
    let rule = |days| {
        let mut r = any_file(json!([{"type": "empty_trash_older_than", "duration": {"n": days, "unit": "days"}}]));
        r.watch[0].path = ".local/share/Trash/files".into();
        r
    };
    assert_eq!(run(&sandbox, &rule(30), ".local/share/Trash/files/vieux.txt")[0].outcome, Outcome::Skipped);
    assert!(sandbox.dir.join(".local/share/Trash/files/vieux.txt").exists());
    assert_eq!(
        run(&sandbox, &any_file(json!([{"type": "empty_trash_older_than", "duration": {"n": 0, "unit": "days"}}])), "Dl/a.txt")[0].outcome,
        Outcome::Skipped
    );
    assert!(sandbox.dir.join("Dl/a.txt").exists());
}

#[test]
fn empty_folders_are_removed_and_full_ones_are_left() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.dir.join("Dl/vide")).unwrap();
    sandbox.write("Dl/plein/x.txt", "1");
    let rule = any_file(json!([{"type": "remove_empty_folders"}]));
    assert_eq!(run(&sandbox, &rule, "Dl/plein")[0].outcome, Outcome::Skipped);
    assert_eq!(run(&sandbox, &rule, "Dl/vide")[0].outcome, Outcome::Done);
    assert!(!sandbox.dir.join("Dl/vide").exists() && sandbox.dir.join("Dl/plein/x.txt").exists());
}

#[test]
fn attribute_actions_change_permissions_but_never_set_special_bits() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("Dl/a.sh", "#!/bin/sh");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    run(&sandbox, &any_file(json!([{"type": "set_executable", "value": true}])), "Dl/a.sh");
    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o755);
    run(&sandbox, &any_file(json!([{"type": "set_read_only", "value": true}])), "Dl/a.sh");
    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o555);
    let refused = run(&sandbox, &any_file(json!([{"type": "set_permissions", "mode": 2285}])), "Dl/a.sh");
    assert_eq!(refused[0].outcome, Outcome::Failed);
}

#[test]
fn set_modified_time_and_create_folder_work_next_to_the_file() {
    let sandbox = Sandbox::new();
    let path = sandbox.write("Dl/2020-05-17 note.txt", "x");
    run(
        &sandbox,
        &any_file(json!([{"type": "set_modified_time", "from": "name"}, {"type": "create_folder", "template": "Rangés/{stem}"}])),
        "Dl/2020-05-17 note.txt",
    );
    let modified = fs::metadata(&path).unwrap().modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    assert!((1_589_600_000..1_589_800_000).contains(&modified), "{modified}");
    assert!(sandbox.dir.join("Dl/Rangés/2020-05-17 note").is_dir());
}
