mod support;

use five_engine::rules::context::{ArchiveInfo, FileContext, ImageInfo, MediaInfo};
use five_engine::rules::eval::condition_holds;
use serde_json::json;
use support::rules_json::{condition, file};

fn holds(ctx: &FileContext, test: serde_json::Value) -> bool {
    condition_holds(ctx, &condition(json!({"test": test})))
}

#[test]
fn text_conditions_read_the_loaded_text_only() {
    let script = FileContext { text: Some("#!/bin/sh\necho 1\necho 2\n".into()), encoding: Some("utf8".into()), ..file("Dl/run.sh") };
    assert!(holds(&script, json!({"type": "has_shebang"})) && holds(&script, json!({"type": "line_count", "cmp": "eq", "n": 3})));
    assert!(
        holds(&script, json!({"type": "first_line", "matcher": {"op": "contains", "value": "/bin/sh"}}))
            && holds(&script, json!({"type": "text_contains", "matcher": {"op": "regex", "value": "echo\\s2"}}))
    );
    assert!(holds(&script, json!({"type": "encoding", "value": "utf8"})) && !holds(&file("Dl/run.sh"), json!({"type": "has_shebang"})));
}

#[test]
fn image_conditions_use_dimensions_and_orientation() {
    let photo = FileContext { image: Some(ImageInfo { width: 4000, height: 3000, has_alpha: true }), ..file("Dl/a.png") };
    assert!(holds(&photo, json!({"type": "image_width", "cmp": "ge", "px": 4000})) && holds(&photo, json!({"type": "image_height", "cmp": "lt", "px": 4000})));
    assert!(holds(&photo, json!({"type": "orientation", "value": "landscape"})) && !holds(&photo, json!({"type": "orientation", "value": "portrait"})));
    assert!(holds(&photo, json!({"type": "megapixels", "cmp": "eq", "n": 12.0})) && holds(&photo, json!({"type": "has_alpha"})));
}

#[test]
fn media_and_archive_conditions_use_their_own_information() {
    let film = FileContext { media: Some(MediaInfo { duration: 125.0, height: 1080, has_audio: true }), ..file("Dl/a.mp4") };
    assert!(
        holds(&film, json!({"type": "duration", "cmp": "gt", "seconds": 120.0}))
            && holds(&film, json!({"type": "video_resolution", "cmp": "ge", "px": 1080}))
            && holds(&film, json!({"type": "has_audio_track"}))
    );
    let zip =
        FileContext { archive: Some(ArchiveInfo { readable: true, encrypted: false, entries: vec!["a/".into(), "a/notes.md".into()] }), ..file("Dl/a.zip") };
    assert!(
        holds(&zip, json!({"type": "archive_is_readable"}))
            && !holds(&zip, json!({"type": "archive_is_encrypted"}))
            && holds(&zip, json!({"type": "archive_entry_count", "cmp": "eq", "n": 2}))
    );
    assert!(holds(&zip, json!({"type": "archive_contains_name", "matcher": {"op": "ends_with", "value": ".md"}})));
}

#[test]
fn missing_information_makes_a_condition_false_not_an_error() {
    let bare = file("Dl/a.png");
    for test in [
        json!({"type": "image_width", "cmp": "gt", "px": 0}),
        json!({"type": "duration", "cmp": "gt", "seconds": 0.0}),
        json!({"type": "archive_is_readable"}),
        json!({"type": "text_contains", "matcher": {"op": "contains", "value": ""}}),
    ] {
        assert!(!holds(&bare, test));
    }
}

#[test]
fn composition_groups_tests_without_leaving_the_ctx() {
    let ctx = file("Dl/photo.png");
    let ext = json!({"test": {"type": "extension", "in_list": ["png"]}});
    let spaces = json!({"test": {"type": "name_has_spaces"}});
    assert!(holds(&ctx, json!({"type": "any_of", "tests": [spaces, ext]})) && !holds(&ctx, json!({"type": "all_of", "tests": [spaces, ext]})));
    assert!(holds(&ctx, json!({"type": "none_of", "tests": [spaces]})));
}
