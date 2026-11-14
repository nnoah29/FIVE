mod support;

use std::fs;

use serde_json::json;
use support::Server;
use support::rules_calls::{read, rule, write};

#[test]
fn preview_activate_run_undo_and_history_work_end_to_end() {
    let server = Server::start();
    for folder in ["Dl", "Docs"] {
        fs::create_dir_all(server.dir.join(folder)).unwrap();
    }
    fs::write(server.dir.join("Dl/a.pdf"), "x").unwrap();
    let id = write(&server, "POST", "/api/rules", &rule("PDF")).1["rule"]["id"].as_str().unwrap().to_string();
    assert_eq!(write(&server, "POST", "/api/rules/enable", &json!({"id": id, "activate": true})).0, 409);
    assert_eq!(write(&server, "POST", "/api/rules/preview", &json!({"id": id})).1["count"], 1);
    assert_eq!(write(&server, "POST", "/api/rules/enable", &json!({"id": id, "activate": true})).1["rule"]["status"], "active");
    assert_eq!(write(&server, "POST", "/api/rules/run", &json!({"id": id})).1["results"][0]["outcome"], "done");
    assert!(server.dir.join("Docs/a.pdf").exists());
    assert_eq!(write(&server, "POST", "/api/rules/undo", &json!({})).1["undone"], 1);
    assert!(server.dir.join("Dl/a.pdf").exists());
    let history = read(&server, "/api/rules/history?limit=5").1;
    assert!(history["items"][0]["undo_of"].is_string() && history["items"][1]["action"] == "move_to");
}

#[test]
fn the_catalog_and_an_empty_list_are_served() {
    let server = Server::start();
    let (code, catalog) = read(&server, "/api/rules/catalog");
    assert_eq!((code, catalog["conditions"].as_array().map(Vec::len), catalog["actions"].as_array().map(Vec::len)), (200, Some(74), Some(40)));
    let list = read(&server, "/api/rules").1;
    assert_eq!((list["rules"].as_array().map(Vec::len), list["damaged"].as_bool()), (Some(0), Some(false)));
}
