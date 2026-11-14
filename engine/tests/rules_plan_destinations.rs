mod support;

use serde_json::json;
use support::rules_helpers::{dated, texts};
use support::rules_json::file;
#[test]
fn name_edits_keep_the_extension_and_ignore_what_changes_nothing() {
    let ctx = file("Dl/le grand PLAN.TXT");
    let out =
        texts(&ctx, json!([{"type": "change_case", "mode": "title"}, {"type": "replace_in_name", "find": "an", "replace": "AN"}, {"type": "remove_spaces"}]));
    assert_eq!(out, vec![Some("Le Grand Plan.TXT".to_string()), Some("Le GrANd PlAN.TXT".to_string()), Some("LeGrANdPlAN.TXT".to_string())]);
    assert!(texts(&file("Dl/déjà.txt"), json!([{"type": "replace_in_name", "find": "zzz", "replace": "y"}])).is_empty());
}

#[test]
fn dates_and_counters_are_added_before_or_after_the_name() {
    let prefix = texts(&dated(), json!([{"type": "add_date", "from": "modified", "position": "prefix", "format": "AAAA-MM-JJ"}]));
    let suffix = texts(
        &dated(),
        json!([{"type": "add_date", "from": "today", "position": "suffix", "format": "AAAA"}, {"type": "add_counter", "start": 10, "width": 3}]),
    );
    assert_eq!(prefix, vec![Some("2024-01-15 Mon Été.JPEG".to_string())]);
    assert_eq!(suffix, vec![Some("Mon Été 2023.JPEG".to_string()), Some("Mon Été 2023_014.JPEG".to_string())]);
}

#[test]
fn destinations_are_built_from_dates_kinds_and_extensions() {
    let out = texts(&dated(), json!([{"type": "move_to_dated", "folder": "Photos", "pattern": "{modified:AAAA}/{modified:MM}"}]));
    assert_eq!(out, vec![Some("Photos/2024/01".to_string())]);
    let kind = texts(&file("Dl/clip.MP4"), json!([{"type": "move_to_by_kind", "base": "Rangé"}]));
    assert_eq!(kind, vec![Some("Rangé/Vidéos".to_string())]);
    assert_eq!(texts(&file("Dl/LISEZMOI"), json!([{"type": "move_to_by_extension", "base": "X"}])), vec![Some("X/sans-extension".to_string())]);
}
