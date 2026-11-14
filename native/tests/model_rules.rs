use five_gtk::model::rules_labels::{action_label, condition_label, outcome_label, status_label, trigger_label};
use five_gtk::model::rules_summary::summarize;
use serde_json::json;

#[test]
fn labels_are_french_and_unknown_names_are_kept_as_they_are() {
    assert_eq!(condition_label("extension"), "L'extension est parmi");
    assert_eq!(action_label("move_to"), "Déplacer vers");
    assert_eq!(trigger_label("stable"), "un fichier ne bouge plus");
    assert_eq!(action_label("inconnue"), "inconnue");
    assert_eq!(
        (status_label("trial"), status_label("invalid"), outcome_label("dry_run"), outcome_label("ask")),
        ("En essai", "Invalide", "Essai", "À décider")
    );
}

#[test]
fn every_catalog_entry_has_a_label() {
    for kind in five_engine::rules::catalog::catalog()["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .chain(five_engine::rules::catalog::catalog()["actions"].as_array().unwrap().iter())
    {
        let name = kind["type"].as_str().unwrap();
        let label =
            if kind["example"]["type"] == name && five_engine::rules::catalog::catalog()["actions"].as_array().unwrap().iter().any(|a| a["type"] == name) {
                action_label(name)
            } else {
                condition_label(name)
            };
        assert_ne!(label, name, "{name}");
    }
    for entry in five_engine::rules::catalog::catalog()["triggers"].as_array().unwrap() {
        assert_ne!(trigger_label(entry["type"].as_str().unwrap()), entry["type"].as_str().unwrap());
    }
}

#[test]
fn the_summary_names_folders_triggers_and_counts() {
    let rule = json!({"watch": [{"path": "Téléchargements", "recursive": true}, {"path": ""}], "triggers": [{"type": "created"}, {"type": "manual"}],
        "alternatives": [{"conditions": [{"test": {"type": "is_file"}}, {"test": {"type": "is_hidden"}}]}], "actions": [{"type": "trash"}]});
    assert_eq!(
        summarize(&rule),
        "Surveille Téléchargements (+ sous-dossiers), Dossier personnel · quand un fichier apparaît, à la demande seulement · 2 conditions → 1 action"
    );
    assert_eq!(summarize(&json!({})), "Surveille aucun dossier · quand aucun déclencheur · 0 condition → 0 action");
}

#[test]
fn bookmarks_are_added_once_removed_cleanly_and_other_lines_are_kept() {
    use five_gtk::model::bookmarks::toggle_bookmark;
    let base = "file:///home/a/Docs Docs\n";
    let added = toggle_bookmark(base, "/home/a/Mes fichiers", true);
    assert_eq!(added, "file:///home/a/Docs Docs\nfile:///home/a/Mes%20fichiers Mes fichiers\n");
    assert_eq!(toggle_bookmark(&added, "/home/a/Mes fichiers", true), added);
    assert_eq!(toggle_bookmark(&added, "/home/a/Mes fichiers", false), base);
    assert_eq!(toggle_bookmark(base, "/home/a/Docs", false), "");
}
