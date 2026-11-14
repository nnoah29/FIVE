use five_engine::rules::catalog::catalog;

#[test]
fn catalog_entries_name_their_family_and_match_their_example() {
    let all = catalog();
    for list in ["conditions", "actions"] {
        for entry in all[list].as_array().unwrap() {
            assert!(!entry["family"].as_str().unwrap().is_empty() && entry["example"]["type"] == entry["type"], "{entry}");
        }
    }
    assert!(all["template_variables"].as_array().unwrap().len() >= 18 && all["limits"]["rules"] == 200);
}

#[test]
fn entries_describe_their_form_fields() {
    let all = catalog();
    let field = |list: &str, kind: &str, name: &str| {
        let entry = all[list].as_array().unwrap().iter().find(|e| e["type"] == kind).unwrap();
        entry["fields"].as_array().unwrap().iter().find(|f| f["name"] == name).unwrap().clone()
    };
    assert_eq!(field("conditions", "name", "matcher")["kind"], "matcher");
    assert_eq!(field("conditions", "size", "cmp")["choices"].as_array().unwrap().len(), 6);
    assert_eq!(field("conditions", "size", "bytes")["kind"], "bytes");
    assert_eq!(field("conditions", "modified_before", "at")["kind"], "date");
    assert_eq!(field("conditions", "permissions", "mode")["kind"], "octal");
    assert_eq!(field("conditions", "any_of", "tests")["kind"], "conditions");
    assert_eq!(field("actions", "move_to", "folder")["kind"], "folder");
    assert_eq!(field("actions", "zip", "name_template")["optional"], true);
    assert_eq!(field("actions", "trash_duplicates", "keep")["choices"][0], "newest");
    assert_eq!(field("triggers", "stable", "for")["kind"], "duration");
}
