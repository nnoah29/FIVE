use five_gtk::model::rules_draft::{Seg, at_mut, key, move_item, new_rule_draft, remove_item};
use five_gtk::model::rules_values::{date_text, format_bytes, parse_bytes, parse_date, parse_list, parse_octal};
use serde_json::json;

#[test]
fn paths_reach_nested_values_and_missing_ones_are_none() {
    let mut rule = new_rule_draft();
    *at_mut(&mut rule, &[key("watch"), Seg::Idx(0), key("path")]).unwrap() = json!("Téléchargements");
    assert_eq!(rule["watch"][0]["path"], "Téléchargements");
    assert!(at_mut(&mut rule, &[key("actions"), Seg::Idx(3)]).is_none() && at_mut(&mut rule, &[key("absent")]).is_none());
}

#[test]
fn items_move_within_their_list_and_can_be_removed() {
    let mut list = json!(["a", "b", "c"]);
    move_item(&mut list, 0, -1);
    move_item(&mut list, 0, 1);
    move_item(&mut list, 2, 1);
    assert_eq!(list, json!(["b", "a", "c"]));
    remove_item(&mut list, 1);
    remove_item(&mut list, 9);
    assert_eq!(list, json!(["b", "c"]));
}

#[test]
fn a_new_draft_is_a_trial_rule_the_engine_can_read() {
    let rule: five_engine::rules::types_rule::Rule = serde_json::from_value({
        let mut draft = new_rule_draft();
        draft["id"] = json!("r");
        draft
    })
    .unwrap();
    assert!(rule.trial && rule.enabled && rule.actions.is_empty() && rule.alternatives.len() == 1);
}

#[test]
fn sizes_are_read_and_written_in_decimal_units() {
    assert_eq!(
        (parse_bytes("5 Mo"), parse_bytes("1,5 Go"), parse_bytes("300"), parse_bytes("2ko"), parse_bytes("12 Xo")),
        (Some(5_000_000), Some(1_500_000_000), Some(300), Some(2_000), None)
    );
    assert_eq!(
        (format_bytes(5_000_000), format_bytes(1_500_000_000), format_bytes(300), format_bytes(1_234_000)),
        ("5 Mo".to_string(), "1,5 Go".to_string(), "300 o".to_string(), "1,23 Mo".to_string())
    );
}

#[test]
fn octal_lists_and_dates_are_parsed_leniently_but_strictly_on_range() {
    assert_eq!((parse_octal("644"), parse_octal(" 0755 "), parse_octal("4755"), parse_octal("89")), (Some(0o644), Some(0o755), None, None));
    assert_eq!(parse_list(" pdf, docx ,, odt "), ["pdf", "docx", "odt"]);
    let seconds = parse_date("2024-01-15 12:30").unwrap();
    assert_eq!(date_text(Some(seconds)), "2024-01-15 12:30");
    assert_eq!((parse_date("n'importe quoi"), date_text(None)), (None, String::new()));
}

#[test]
fn every_field_and_choice_of_the_catalog_has_a_french_label() {
    use five_gtk::model::rules_fields_labels::{choice_label, field_label};
    let all = five_engine::rules::catalog::catalog();
    for list in ["triggers", "conditions", "actions"] {
        for entry in all[list].as_array().unwrap() {
            for field in entry["fields"].as_array().unwrap() {
                let (name, kind) = (field["name"].as_str().unwrap(), field["kind"].as_str().unwrap());
                assert_ne!(field_label(name, kind), name, "champ {name} de {}", entry["type"]);
                let same_word = ["image", "audio", "document", "archive", "code", "portrait"];
                for choice in field["choices"].as_array().into_iter().flatten().filter(|c| !same_word.contains(&c.as_str().unwrap())) {
                    assert_ne!(choice_label(choice.as_str().unwrap()), choice.as_str().unwrap(), "valeur {choice}");
                }
            }
        }
    }
}
