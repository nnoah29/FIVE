mod support;

use std::fs;

use five_engine::archive::list_archive;
use five_engine::rules::run_types::Outcome;
use serde_json::json;
use support::Sandbox;
use support::rules_run::{any_file, run};

#[test]
fn move_to_creates_dated_subfolders_and_keeps_the_content() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "contenu");
    fs::create_dir(sandbox.dir.join("Docs")).unwrap();
    let rule = any_file(json!([{"type": "move_to_by_kind", "base": "Docs"}]));
    let results = run(&sandbox, &rule, "Dl/a.txt");
    assert_eq!(results[0].outcome, Outcome::Done, "{results:?}");
    assert_eq!(fs::read_to_string(sandbox.dir.join("Docs/Documents/a.txt")).unwrap(), "contenu");
    assert!(!sandbox.dir.join("Dl/a.txt").exists());
}

#[test]
fn a_destination_name_conflict_follows_the_policy() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "nouveau");
    sandbox.write("Docs/a.txt", "ancien");
    run(&sandbox, &any_file(json!([{"type": "move_to", "folder": "Docs"}])), "Dl/a.txt");
    assert_eq!(fs::read_to_string(sandbox.dir.join("Docs/a.txt")).unwrap(), "ancien");
    assert_eq!(fs::read_to_string(sandbox.dir.join("Docs/a (2).txt")).unwrap(), "nouveau");
    sandbox.write("Dl/b.txt", "1");
    sandbox.write("Docs/b.txt", "2");
    let mut skip = any_file(json!([{"type": "move_to", "folder": "Docs"}]));
    skip.on_conflict = serde_json::from_value(json!("skip")).unwrap();
    assert_eq!(run(&sandbox, &skip, "Dl/b.txt")[0].outcome, Outcome::Skipped);
}

#[test]
fn copy_to_keeps_the_original() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "x");
    let results = run(&sandbox, &any_file(json!([{"type": "copy_to", "folder": "Copie"}])), "Dl/a.txt");
    assert_eq!(results[0].outcome, Outcome::Done);
    assert!(sandbox.dir.join("Dl/a.txt").is_file() && sandbox.dir.join("Copie/a.txt").is_file());
}

#[test]
fn a_destination_that_climbs_out_is_refused() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "x");
    let results = run(&sandbox, &any_file(json!([{"type": "move_to", "folder": "../dehors"}])), "Dl/a.txt");
    assert_eq!(results[0].outcome, Outcome::Failed);
    assert!(sandbox.dir.join("Dl/a.txt").exists());
}

#[test]
fn zip_one_for_all_makes_a_single_archive_and_unzip_extracts_next_to_it() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "AAA");
    sandbox.write("Dl/b.txt", "BBB");
    let rule = any_file(json!([{"type": "zip", "mode": "one_for_all", "name_template": "Lot"}]));
    let jobs = five_engine::jobs::registry::Jobs::new();
    let planned = [support::rules_run::plan(&rule, "Dl/a.txt"), support::rules_run::plan(&rule, "Dl/b.txt")].concat();
    let root = sandbox.root();
    five_engine::rules::exec_pass::run_plan(&five_engine::rules::run_types::RunEnv { root: &root, jobs: &jobs }, &planned);
    assert_eq!(list_archive(&sandbox.dir.join("Dl/Lot.zip")).unwrap()["items"].as_array().unwrap().len(), 2);
    let again = any_file(json!([{"type": "unzip", "then": "trash_archive"}]));
    assert_eq!(run(&sandbox, &again, "Dl/Lot.zip")[0].outcome, Outcome::Done);
    assert_eq!(fs::read_to_string(sandbox.dir.join("Dl/Lot/a.txt")).unwrap(), "AAA");
    assert!(!sandbox.dir.join("Dl/Lot.zip").exists());
}
