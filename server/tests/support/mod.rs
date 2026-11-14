//! Banc d'essai : lance le vrai binaire `five` sur un dossier jetable et parle HTTP brut avec lui.

#![allow(dead_code)]

pub mod rules_calls;

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub struct Server {
    pub dir: PathBuf,
    pub port: u16,
    pub token: String,
    child: Child,
}

pub struct Answer {
    pub status: u16,
    pub head: String,
    pub body: String,
}

impl Server {
    pub fn start() -> Self {
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let dir = std::env::temp_dir().join(format!("five-http-{}-{port}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let dir = fs::canonicalize(dir).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_five"))
            .args(["--root", dir.to_str().unwrap(), "--port", &port.to_string()])
            .env("HOME", &dir)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let url = lines.nth(1).unwrap().unwrap();
        let token = url.split("?k=").nth(1).unwrap().to_string();
        Self { dir, port, token, child }
    }

    pub fn send(&self, raw: &str) -> Answer {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream.write_all(raw.replace("{host}", &format!("127.0.0.1:{}", self.port)).as_bytes()).unwrap();
        let mut text = Vec::new();
        stream.read_to_end(&mut text).unwrap();
        let text = String::from_utf8_lossy(&text).into_owned();
        let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
        let status = head.split_whitespace().nth(1).unwrap().parse().unwrap();
        Answer { status, head: head.to_string(), body: body.to_string() }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.dir);
    }
}
