//! Appels HTTP des tests de règles : requête avec cookie, écriture avec en-tête `X-Five`, lecture, règle type.

use serde_json::{Value, json};

use super::{Answer, Server};

pub fn call(server: &Server, method: &str, path: &str, body: Option<&Value>, extra: &str) -> Answer {
    let text = body.map_or(String::new(), Value::to_string);
    let head = format!(
        "{method} {path} HTTP/1.1\r\nHost: {{host}}\r\nConnection: close\r\nCookie: mk={}\r\n{extra}Content-Length: {}\r\n\r\n",
        server.token,
        text.len()
    );
    server.send(&format!("{head}{text}"))
}

pub fn write(server: &Server, method: &str, path: &str, body: &Value) -> (u16, Value) {
    let answer = call(server, method, path, Some(body), "X-Five: 1\r\n");
    (answer.status, serde_json::from_str(&answer.body).unwrap_or(Value::Null))
}

pub fn read(server: &Server, path: &str) -> (u16, Value) {
    let answer = call(server, "GET", path, None, "");
    (answer.status, serde_json::from_str(&answer.body).unwrap_or(Value::Null))
}

pub fn rule(name: &str) -> Value {
    json!({"name": name, "watch": [{"path": "Dl"}], "triggers": [{"type": "manual"}],
        "alternatives": [{"conditions": [{"test": {"type": "extension", "in_list": ["pdf"]}}]}], "actions": [{"type": "move_to", "folder": "Docs"}]})
}
