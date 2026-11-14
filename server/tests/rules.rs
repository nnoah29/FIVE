mod support;

use std::fs;

use serde_json::json;
use support::Server;
use support::rules_calls::{call, read, rule, write};

#[test]
fn rules_routes_need_a_session_the_write_header_and_a_known_method() {
    let server = Server::start();
    let no_session = server.send("GET /api/rules HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    assert_eq!(no_session.status, 401);
    for method in ["POST", "PUT", "DELETE"] {
        assert_eq!(call(&server, method, "/api/rules/r-x", Some(&rule("a")), "").status, 403, "{method}");
    }
    assert_eq!(call(&server, "POST", "/api/rules", Some(&rule("a")), "X-Five: 1\r\nOrigin: http://evil.example\r\n").status, 403);
    assert_eq!(call(&server, "PATCH", "/api/rules", None, "X-Five: 1\r\n").status, 405);
}

#[test]
fn a_rule_is_created_listed_updated_and_deleted() {
    let server = Server::start();
    fs::create_dir_all(server.dir.join("Dl")).unwrap();
    fs::create_dir_all(server.dir.join("Docs")).unwrap();
    let (code, made) = write(&server, "POST", "/api/rules", &rule("PDF"));
    let id = made["rule"]["id"].as_str().unwrap().to_string();
    assert_eq!((code, made["rule"]["status"].as_str()), (201, Some("trial")));
    assert_eq!(read(&server, "/api/rules").1["rules"][0]["name"], "PDF");
    let (code, changed) = write(&server, "PUT", &format!("/api/rules/{id}"), &rule("PDF 2"));
    assert_eq!((code, changed["rule"]["name"].as_str()), (200, Some("PDF 2")));
    assert_eq!(write(&server, "DELETE", &format!("/api/rules/{id}"), &json!({})).0, 200);
    assert!(read(&server, "/api/rules").1["rules"].as_array().unwrap().is_empty());
}

#[test]
fn bad_requests_get_clear_refusals() {
    let server = Server::start();
    let mut bad = rule("");
    bad["watch"] = json!([{"path": "Absent"}]);
    let (code, answer) = write(&server, "POST", "/api/rules", &bad);
    assert_eq!(code, 400);
    assert!(answer["erreur"].as_str().unwrap().contains("/Absent"));
    assert_eq!(write(&server, "POST", "/api/rules/validate", &bad).1["valid"], false);
    assert_eq!(write(&server, "PUT", "/api/rules/r-inconnu", &rule("a")).0, 400);
    assert_eq!(write(&server, "DELETE", "/api/rules/../secret", &json!({})).0, 404);
    assert_eq!(write(&server, "POST", "/api/rules/inconnue", &json!({})).0, 404);
    assert_eq!(write(&server, "POST", "/api/rules/undo", &json!({})).0, 404);
}
