mod support;

use std::time::Duration;

use five_engine::duplicates::find_duplicates;
use five_engine::text::read_text;
use five_engine::usage::usage_of;
use support::Sandbox;

#[test]
fn usage_sums_children_and_sorts_by_size() {
    let sandbox = Sandbox::new();
    sandbox.write("gros/a", &"x".repeat(1000));
    sandbox.write("gros/sous/b", &"x".repeat(500));
    sandbox.write("petit.txt", "xx");
    let report = usage_of(&sandbox.root(), &sandbox.dir, Duration::from_secs(5)).unwrap();
    assert_eq!(report["total"], 1502);
    assert_eq!(report["items"][0]["n"], "gros");
    assert_eq!(report["items"][0]["s"], 1500);
    assert_eq!(report["items"][0]["d"], true);
    assert_eq!(report["partial"], false);
}

#[test]
fn duplicates_group_identical_files_only() {
    let sandbox = Sandbox::new();
    let same = "a".repeat(2000);
    sandbox.write("un.bin", &same);
    sandbox.write("d/deux.bin", &same);
    sandbox.write("d/autre.bin", &"b".repeat(2000));
    sandbox.write("petit1", "x");
    sandbox.write("petit2", "x");
    let report = find_duplicates(&sandbox.root(), &sandbox.dir, 1024, Duration::from_secs(5));
    let groups = report["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["files"], serde_json::json!(["d/deux.bin", "un.bin"]));
    assert_eq!(report["wasted"], 2000);
}

#[test]
fn duplicates_skip_hidden_folders() {
    let sandbox = Sandbox::new();
    let same = "a".repeat(2000);
    sandbox.write("un.bin", &same);
    sandbox.write(".cache/deux.bin", &same);
    let report = find_duplicates(&sandbox.root(), &sandbox.dir, 1024, Duration::from_secs(5));
    assert!(report["groups"].as_array().unwrap().is_empty());
}

#[test]
fn text_reads_utf8_and_flags_binary() {
    let sandbox = Sandbox::new();
    let note = sandbox.write("n.txt", "été");
    let blob = sandbox.write("b.bin", "ab\0cd");
    assert_eq!(read_text(&note).unwrap()["text"], "été");
    assert_eq!(read_text(&blob).unwrap()["binary"], true);
}

#[test]
fn text_truncates_after_200_kb() {
    let sandbox = Sandbox::new();
    let big = sandbox.write("g.txt", &"x".repeat(250_000));
    let view = read_text(&big).unwrap();
    assert_eq!(view["truncated"], true);
    assert_eq!(view["text"].as_str().unwrap().len(), 200_000);
}
