mod support;

use five_engine::rules::context::{DuplicateGroup, Duplicates, FileContext};
use five_engine::rules::eval::condition_holds;
use serde_json::json;
use support::rules_json::{condition, file};

fn holds(ctx: &FileContext, test: serde_json::Value) -> bool {
    condition_holds(ctx, &condition(json!({"test": test})))
}

#[test]
fn duplicate_conditions_depend_on_the_scope() {
    let group = |others, newest| DuplicateGroup { others, newest, oldest: !newest, shortest: false, same_name: 1 };
    let ctx = FileContext { duplicates: Some(Duplicates { folder: group(0, true), watched: group(2, true), root: group(2, false) }), ..file("Dl/a.bin") };
    assert!(
        !holds(&ctx, json!({"type": "duplicate_of_another", "scope": "folder"})) && holds(&ctx, json!({"type": "duplicate_of_another", "scope": "watched"}))
    );
    assert!(
        holds(&ctx, json!({"type": "is_newest_of_duplicates", "scope": "watched"})) && holds(&ctx, json!({"type": "is_oldest_of_duplicates", "scope": "root"}))
    );
    assert!(holds(&ctx, json!({"type": "same_name_as", "scope": "root"})) && !holds(&ctx, json!({"type": "duplicate_of_another", "scope": "ailleurs"})));
}

#[test]
fn git_origin_and_disk_conditions_read_optional_fields() {
    let ctx = FileContext {
        git_status: Some("untracked".into()),
        origin_url: Some("https://exemple.org/a.zip".into()),
        disk: Some("Données".into()),
        ..file("Dl/a.zip")
    };
    assert!(
        holds(&ctx, json!({"type": "git_status", "in_list": ["modified", "untracked"]})) && !holds(&ctx, json!({"type": "git_status", "in_list": ["ignored"]}))
    );
    assert!(
        holds(&ctx, json!({"type": "downloaded_from", "matcher": {"op": "contains", "value": "exemple.org"}}))
            && holds(&ctx, json!({"type": "from_disk", "disk": "données"}))
    );
}

#[test]
fn system_conditions_check_space_mounts_owner_and_permissions() {
    let ctx = FileContext {
        free_folder: Some(1_000),
        free_disk: Some(9_000_000),
        mounted_disks: vec!["Sauvegarde".into()],
        owned_by_me: true,
        mode: Some(0o100644),
        ..file("Dl/a.txt")
    };
    assert!(
        holds(&ctx, json!({"type": "free_space_below", "bytes": 5_000, "on": "folder"}))
            && !holds(&ctx, json!({"type": "free_space_below", "bytes": 5_000, "on": "disk"}))
    );
    assert!(
        holds(&ctx, json!({"type": "disk_is_mounted", "disk": "sauvegarde"}))
            && holds(&ctx, json!({"type": "owned_by_me"}))
            && holds(&ctx, json!({"type": "permissions", "mode": 420}))
    );
}

#[test]
fn interface_conditions_read_favorites_recents_and_pins() {
    let ctx = FileContext { favorite: true, pinned: true, ..file("Dl/Projets") };
    assert!(holds(&ctx, json!({"type": "is_favorite"})) && holds(&ctx, json!({"type": "is_pinned_folder"})) && !holds(&ctx, json!({"type": "in_recents"})));
    assert!(holds(&FileContext { in_git: true, in_trash: true, ..file("Dl/x") }, json!({"type": "inside_git_repo"})));
}
