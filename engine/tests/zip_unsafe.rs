mod support;

use std::fs;
use std::thread::sleep;
use std::time::{Duration, Instant};

use five_engine::jobs::registry::Jobs;
use five_engine::jobs::start::start_job;
use flate2::Crc;
use support::Sandbox;
use support::zip_bytes::stored_zip;

fn extract(sandbox: &Sandbox, bytes: &[u8]) -> (String, String) {
    fs::write(sandbox.dir.join("piege.zip"), bytes).unwrap();
    fs::create_dir_all(sandbox.dir.join("ext")).unwrap();
    let jobs = Jobs::new();
    let id = start_job(&sandbox.root(), &jobs, "unzip", &["piege.zip".to_string()], "ext").unwrap();
    let started = Instant::now();
    loop {
        let view = jobs.view().into_iter().find(|j| j["id"] == id).unwrap();
        if view["state"] != "running" || started.elapsed() > Duration::from_secs(10) {
            return (view["state"].as_str().unwrap().to_string(), view["error"].as_str().unwrap().to_string());
        }
        sleep(Duration::from_millis(10));
    }
}

fn crc(data: &[u8]) -> u32 {
    let mut sum = Crc::new();
    sum.update(data);
    sum.sum()
}

#[test]
fn an_entry_that_climbs_out_of_the_folder_is_refused() {
    let sandbox = Sandbox::new();
    let (state, error) = extract(&sandbox, &stored_zip("../evil.txt", b"mal", crc(b"mal"), 3));
    assert_eq!(state, "error");
    assert!(error.contains("dangereuse"));
    assert!(!sandbox.dir.join("evil.txt").exists() && !sandbox.dir.join("ext/evil.txt").exists());
    assert!(!sandbox.dir.join("ext/piege").exists());
}

#[test]
fn a_wrong_checksum_removes_the_partial_folder() {
    let sandbox = Sandbox::new();
    let (state, _) = extract(&sandbox, &stored_zip("a.txt", b"abc", 12345, 3));
    assert_eq!(state, "error");
    assert!(!sandbox.dir.join("ext/piege").exists());
}

#[test]
fn data_larger_than_the_announced_size_is_refused() {
    let sandbox = Sandbox::new();
    let (state, _) = extract(&sandbox, &stored_zip("a.txt", b"abcdef", crc(b"abcdef"), 3));
    assert_eq!(state, "error");
    assert!(!sandbox.dir.join("ext/piege").exists());
}

#[test]
fn a_well_formed_stored_entry_is_extracted() {
    let sandbox = Sandbox::new();
    let (state, _) = extract(&sandbox, &stored_zip("/abs/./a.txt", b"abc", crc(b"abc"), 3));
    assert_eq!(state, "done");
    assert_eq!(fs::read_to_string(sandbox.dir.join("ext/piege/abs/a.txt")).unwrap(), "abc");
}
