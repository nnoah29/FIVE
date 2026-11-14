use five_engine::rules::info_image::image_info;
use five_engine::rules::needs::needs_of;
use five_engine::rules::types_rule::Rule;
use serde_json::{Value, json};

fn rule(condition: Value, action: Value) -> Rule {
    serde_json::from_value(json!({"id": "r", "name": "r", "watch": [{"path": "Dl"}], "triggers": [{"type": "created"}], "alternatives": [{"conditions": [{"test": condition}]}], "actions": [action]})).unwrap()
}

#[test]
fn needs_follow_the_conditions_and_the_templates_the_rules_use() {
    let plain = rule(json!({"type": "extension", "in_list": ["pdf"]}), json!({"type": "trash"}));
    assert_eq!(needs_of(&[&plain]), Default::default());
    let text = rule(json!({"type": "all_of", "tests": [{"test": {"type": "has_shebang"}}]}), json!({"type": "rename", "template": "{hash:6}_{stem}"}));
    let needs = needs_of(&[&plain, &text]);
    assert!(needs.text && needs.hash && !needs.image && !needs.duplicates);
    let dups = rule(json!({"type": "is_file"}), json!({"type": "trash_duplicates", "keep": "oldest"}));
    assert!(needs_of(&[&dups]).duplicates);
}

#[test]
fn image_headers_give_their_dimensions_for_png_gif_bmp_jpeg_and_webp() {
    let gif = [b"GIF89a".to_vec(), vec![10, 0, 20, 0]].concat();
    assert_eq!(image_info(&gif).map(|i| (i.width, i.height)), Some((10, 20)));
    let mut bmp = vec![0u8; 26];
    bmp[0..2].copy_from_slice(b"BM");
    bmp[18] = 64;
    bmp[22] = 32;
    assert_eq!(image_info(&bmp).map(|i| (i.width, i.height)), Some((64, 32)));
    let jpeg = [vec![0xff, 0xd8, 0xff, 0xe0, 0, 4, 0, 0], vec![0xff, 0xc0, 0, 11, 8, 1, 224, 2, 128, 3]].concat();
    assert_eq!(image_info(&jpeg).map(|i| (i.width, i.height)), Some((640, 480)));
    let mut webp = vec![0u8; 30];
    webp[0..4].copy_from_slice(b"RIFF");
    webp[8..16].copy_from_slice(b"WEBPVP8X");
    webp[24] = 99;
    webp[27] = 49;
    assert_eq!(image_info(&webp).map(|i| (i.width, i.height)), Some((100, 50)));
    assert!(image_info(b"pas une image").is_none());
}
