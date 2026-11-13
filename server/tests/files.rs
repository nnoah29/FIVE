mod support;

use std::fs;

use support::{Answer, Server};

fn api(server: &Server, path: &str, extra: &str) -> Answer {
    let session = format!("Cookie: mk={}\r\n{extra}", server.token);
    server.send(&format!("GET {path} HTTP/1.1\r\nHost: {{host}}\r\nConnection: close\r\n{session}\r\n"))
}

fn post(server: &Server, path: &str, body: &str, extra: &str) -> Answer {
    let session = format!("Cookie: mk={}\r\n{extra}", server.token);
    let head = format!("POST {path} HTTP/1.1\r\nHost: {{host}}\r\nConnection: close\r\n{session}Content-Length: {}\r\n\r\n", body.len());
    server.send(&format!("{head}{body}"))
}

#[test]
fn writes_need_the_x_five_header_and_a_local_origin() {
    let server = Server::start();
    let body = r#"{"p":"","name":"Neuf"}"#;
    assert_eq!(post(&server, "/api/mkdir", body, "").status, 403);
    assert_eq!(post(&server, "/api/mkdir", body, "X-Five: 1\r\nOrigin: http://evil.example\r\n").status, 403);
    assert_eq!(post(&server, "/api/mkdir", body, "X-Five: 1\r\n").status, 201);
    assert!(server.dir.join("Neuf").is_dir());
}

#[test]
fn listing_and_traversal_are_confined_to_the_root() {
    let server = Server::start();
    fs::write(server.dir.join("a.txt"), "hello").unwrap();
    let listing = api(&server, "/api/list?p=", "");
    assert!(listing.body.contains("\"n\":\"a.txt\""), "{}", listing.body);
    assert_eq!(api(&server, "/api/list?p=../..", "").status, 403);
    assert_eq!(api(&server, "/api/raw?p=../../etc/passwd", "").status, 403);
}

#[test]
fn raw_serves_ranges_with_safety_headers() {
    let server = Server::start();
    fs::write(server.dir.join("r.txt"), "0123456789").unwrap();
    let part = api(&server, "/api/raw?p=r.txt", "Range: bytes=2-5\r\n");
    assert_eq!((part.status, part.body.as_str()), (206, "2345"));
    assert!(part.head.contains("Content-Range: bytes 2-5/10"));
    assert!(part.head.contains("Content-Security-Policy: sandbox") && part.head.contains("nosniff"));
}

#[test]
fn delete_goes_to_the_trash_and_oversized_bodies_are_refused() {
    let server = Server::start();
    fs::write(server.dir.join("x.txt"), "x").unwrap();
    let gone = post(&server, "/api/delete", r#"{"paths":["x.txt"]}"#, "X-Five: 1\r\n");
    assert!(gone.body.contains("\"mode\":\"trash\""), "{}", gone.body);
    assert!(server.dir.join(".local/share/Trash/files/x.txt").exists());
    let huge = format!(r#"{{"paths":["{}"]}}"#, "a".repeat(70_000));
    assert_eq!(post(&server, "/api/delete", &huge, "X-Five: 1\r\n").status, 413);
}
