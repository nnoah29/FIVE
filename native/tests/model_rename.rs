use five_gtk::model::rename_plan::{RenameSpec, check_names, new_name};
use five_gtk::model::settings::Settings;

#[test]
fn replace_mode_supports_plain_text_and_regex() {
    let mut spec = RenameSpec { find: "IMG".into(), replacement: "photo".into(), ..RenameSpec::default() };
    assert_eq!(new_name(&spec, "img_001.jpg", 0).unwrap(), "photo_001.jpg");
    spec.regex = true;
    spec.find = r"(\d+)".into();
    spec.replacement = "[$1]".into();
    assert_eq!(new_name(&spec, "img_001.jpg", 0).unwrap(), "img_[001].jpg");
    spec.find = "(".into();
    assert!(new_name(&spec, "x", 0).is_err());
}

#[test]
fn template_mode_numbers_the_batch() {
    let spec = RenameSpec { template_mode: true, ..RenameSpec::default() };
    assert_eq!(new_name(&spec, "a.txt", 0).unwrap(), "a-01.txt");
    assert_eq!(new_name(&spec, "b.txt", 11).unwrap(), "b-12.txt");
}

#[test]
fn name_checks_report_conflicts() {
    let olds = vec!["a".to_string(), "b".to_string()];
    let siblings = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(check_names(&olds, &["x".into(), "x".into()], &siblings), ["", "Doublon dans le lot"]);
    assert_eq!(check_names(&olds, &["c".into(), "b".into()], &siblings), ["Existe déjà", ""]);
    assert_eq!(check_names(&olds, &["".into(), "a/b".into()], &siblings), ["Nom invalide", "Nom invalide"]);
}

#[test]
fn settings_round_trip_through_json() {
    let mut changed = Settings::default();
    changed.accent = "#3b7bea".into();
    changed.compact = true;
    changed.ascending = false;
    assert_eq!(Settings::from_json(&changed.to_json()), changed);
    assert_eq!(Settings::from_json(&serde_json::Value::Null), Settings::default());
}
