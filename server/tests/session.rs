mod support;

use support::Server;

fn get(server: &Server, path: &str, extra: &str) -> support::Answer {
    server.send(&format!("GET {path} HTTP/1.1\r\nHost: {{host}}\r\nConnection: close\r\n{extra}\r\n"))
}

fn cookie(server: &Server) -> String {
    format!("Cookie: mk={}\r\n", server.token)
}

#[test]
fn api_requires_a_session() {
    let server = Server::start();
    assert_eq!(get(&server, "/api/places", "").status, 401);
    assert_eq!(get(&server, "/api/places", &cookie(&server)).status, 200);
    assert_eq!(get(&server, "/api/places", "Cookie: mk=faux\r\n").status, 401);
}

#[test]
fn launch_token_sets_an_httponly_strict_cookie() {
    let server = Server::start();
    let good = get(&server, &format!("/?k={}", server.token), "");
    assert_eq!(good.status, 302);
    assert!(good.head.contains("HttpOnly; SameSite=Strict"), "{}", good.head);
    assert_eq!(get(&server, "/?k=mauvais", "").status, 401);
}

#[test]
fn foreign_host_headers_are_refused() {
    let server = Server::start();
    let raw = "GET /api/places HTTP/1.1\r\nHost: evil.example\r\nConnection: close\r\n\r\n";
    assert_eq!(server.send(raw).status, 403);
}

#[test]
fn the_web_interface_is_served_without_a_session() {
    let server = Server::start();
    let page = get(&server, "/", "");
    assert_eq!(page.status, 200);
    assert!(page.body.contains("<title>FIVE</title>"));
    assert_eq!(get(&server, "/..%2f..%2fetc/passwd", "").status, 404);
}
