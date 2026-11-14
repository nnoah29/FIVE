mod support;

use five_engine::rules::context::FileContext;
use five_engine::rules::eval::condition_holds;
use serde_json::json;
use support::rules_json::{condition, file};

fn holds(ctx: &FileContext, test: serde_json::Value) -> bool {
    condition_holds(ctx, &condition(json!({"test": test})))
}

#[test]
fn nature_conditions_read_the_flags_of_the_context() {
    let hidden = file("Dl/.cache");
    let folder = FileContext { is_dir: true, empty: true, ..file("Dl/Vide") };
    assert!(holds(&hidden, json!({"type": "is_hidden"})) && holds(&hidden, json!({"type": "is_file"})));
    assert!(holds(&folder, json!({"type": "is_folder"})) && holds(&folder, json!({"type": "is_empty"})) && !holds(&folder, json!({"type": "is_file"})));
    let link = FileContext { is_symlink: true, broken_symlink: true, ..file("Dl/lien") };
    assert!(holds(&link, json!({"type": "is_symlink"})) && holds(&link, json!({"type": "is_broken_symlink"})));
}

#[test]
fn kind_and_mime_follow_the_extension() {
    let photo = FileContext { mime: Some("image/png".into()), ..file("Dl/a.PNG") };
    assert!(holds(&photo, json!({"type": "kind", "in_list": ["image", "video"]})) && !holds(&photo, json!({"type": "kind", "in_list": ["audio"]})));
    assert!(holds(&photo, json!({"type": "mime", "matcher": {"op": "starts_with", "value": "image/"}})));
    assert!(holds(&file("Dl/x.rs"), json!({"type": "kind", "in_list": ["code"]})) && holds(&file("Dl/x.zst"), json!({"type": "kind", "in_list": ["archive"]})));
}

#[test]
fn size_conditions_compare_and_need_the_data() {
    let big = FileContext { size: 5_000_000, ..file("Dl/a.iso") };
    assert!(holds(&big, json!({"type": "size", "cmp": "gt", "bytes": 1_000_000})) && !holds(&big, json!({"type": "size", "cmp": "le", "bytes": 1_000_000})));
    assert!(!holds(&big, json!({"type": "child_count", "cmp": "ge", "n": 0})));
    let folder = FileContext { is_dir: true, child_count: Some(3), file_count: Some(10), folder_size: Some(2_000), ..file("Dl/D") };
    assert!(
        holds(&folder, json!({"type": "child_count", "cmp": "eq", "n": 3}))
            && holds(&folder, json!({"type": "folder_total_size", "cmp": "lt", "bytes": 3_000}))
    );
}

#[test]
fn age_conditions_use_the_current_time() {
    let day = 86_400.0;
    let old = FileContext {
        modified: Some(1_700_000_000.0 - 40.0 * day),
        created: Some(1_700_000_000.0 - 50.0 * day),
        accessed: Some(1_700_000_000.0 - 50.0 * day),
        ..file("Dl/a.txt")
    };
    assert!(
        holds(&old, json!({"type": "older_than", "duration": {"n": 30, "unit": "days"}}))
            && !holds(&old, json!({"type": "modified_within", "duration": {"n": 1, "unit": "weeks"}}))
    );
    assert!(holds(&old, json!({"type": "accessed_older_than", "duration": {"n": 1, "unit": "months"}})) && holds(&old, json!({"type": "never_opened"})));
    assert!(holds(&old, json!({"type": "modified_before", "at": 1_700_000_000.0})) && !holds(&old, json!({"type": "created_after", "at": 1_700_000_000.0})));
}

#[test]
fn calendar_conditions_read_weekday_month_and_year() {
    let monday = FileContext { modified: Some(1_705_320_000.0), ..file("Dl/a.txt") };
    assert!(
        holds(&monday, json!({"type": "modified_on_weekday", "days": ["mon", "tue"]}))
            && !holds(&monday, json!({"type": "modified_on_weekday", "days": ["sun"]}))
    );
    assert!(holds(&monday, json!({"type": "modified_in_month", "months": [1]})) && holds(&monday, json!({"type": "modified_in_year", "years": [2024]})));
    assert!(!holds(&file("Dl/inconnu.txt"), json!({"type": "modified_in_year", "years": [2024]})));
}
