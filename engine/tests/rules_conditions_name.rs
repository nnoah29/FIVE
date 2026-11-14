mod support;

use five_engine::rules::eval::condition_holds;
use serde_json::json;
use support::rules_json::{condition, file};

fn holds(path: &str, test: serde_json::Value) -> bool {
    condition_holds(&file(path), &condition(json!({"test": test})))
}

#[test]
fn name_matchers_cover_every_operator_and_ignore_case() {
    let name = |op: &str, value: &str| holds("Dl/Facture-2024.PDF", json!({"type": "name", "matcher": {"op": op, "value": value}}));
    assert!(name("contains", "facture") && name("equals", "facture-2024.pdf") && name("starts_with", "FACT") && name("ends_with", ".pdf"));
    assert!(name("glob", "facture-*.pdf") && name("regex", r"^facture-\d{4}\.pdf$") && !name("contains", "devis"));
    assert!(holds("Dl/a.txt", json!({"type": "name", "matcher": {"op": "in_list", "values": ["A.TXT", "b.txt"]}})));
}

#[test]
fn case_sensitive_matchers_distinguish_case() {
    let test = json!({"type": "name", "matcher": {"op": "contains", "value": "facture", "case_sensitive": true}});
    assert!(!holds("Dl/Facture.pdf", test.clone()) && holds("Dl/facture.pdf", test));
}

#[test]
fn stem_and_extension_ignore_the_dot_and_case() {
    assert!(holds("Dl/photo.JPG", json!({"type": "extension", "in_list": [".png", "jpg"]})));
    assert!(holds("Dl/photo.JPG", json!({"type": "stem", "matcher": {"op": "equals", "value": "photo"}})));
    assert!(!holds("Dl/.bashrc", json!({"type": "extension", "in_list": ["bashrc"]})));
}

#[test]
fn name_shape_conditions_detect_spaces_uppercase_accents_and_dates() {
    assert!(holds("Dl/Mon fichier.txt", json!({"type": "name_has_spaces"})));
    assert!(holds("Dl/Rapport.txt", json!({"type": "name_has_uppercase"})) && !holds("Dl/rapport.txt", json!({"type": "name_has_uppercase"})));
    assert!(holds("Dl/été.txt", json!({"type": "name_has_accents"})) && !holds("Dl/ete.txt", json!({"type": "name_has_accents"})));
    assert!(
        ["scan-2024-03-09.pdf", "IMG_20240309.jpg", "rapport 09-03-2024.docx"].iter().all(|n| holds(&format!("Dl/{n}"), json!({"type": "name_has_date_like"})))
    );
    assert!(!holds("Dl/version-1234.txt", json!({"type": "name_has_date_like"})));
}
