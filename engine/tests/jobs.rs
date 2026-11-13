mod support;

use std::fs;
use std::thread::sleep;
use std::time::{Duration, Instant};

use five_engine::jobs::registry::Jobs;
use five_engine::jobs::start::start_job;
use support::Sandbox;

fn paths(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn wait_until_done(jobs: &Jobs, id: &str) -> String {
    let started = Instant::now();
    loop {
        let view = jobs.view().into_iter().find(|j| j["id"] == id).unwrap();
        let state = view["state"].as_str().unwrap().to_string();
        if state != "running" || started.elapsed() > Duration::from_secs(10) {
            return state;
        }
        sleep(Duration::from_millis(10));
    }
}

#[test]
fn copy_duplicates_files_and_folders() {
    let sandbox = Sandbox::new();
    sandbox.write("src/a.txt", "AAA");
    sandbox.write("src/sous/b.txt", "BB");
    fs::create_dir(sandbox.dir.join("cible")).unwrap();
    let jobs = Jobs::new();
    let id = start_job(&sandbox.root(), &jobs, "copy", &paths(&["src"]), "cible").unwrap();
    assert_eq!(wait_until_done(&jobs, &id), "done");
    assert_eq!(fs::read_to_string(sandbox.dir.join("cible/src/sous/b.txt")).unwrap(), "BB");
    assert!(sandbox.dir.join("src/a.txt").exists());
    let view = jobs.view().remove(0);
    assert_eq!((view["total"].as_u64(), view["done"].as_u64()), (Some(5), Some(5)));
}

#[test]
fn copy_never_overwrites_and_numbers_the_copy() {
    let sandbox = Sandbox::new();
    sandbox.write("a.txt", "1");
    sandbox.write("d/a.txt", "2");
    let jobs = Jobs::new();
    let id = start_job(&sandbox.root(), &jobs, "copy", &paths(&["a.txt"]), "d").unwrap();
    assert_eq!(wait_until_done(&jobs, &id), "done");
    assert_eq!(fs::read_to_string(sandbox.dir.join("d/a.txt")).unwrap(), "2");
    assert_eq!(fs::read_to_string(sandbox.dir.join("d/a (2).txt")).unwrap(), "1");
}

#[test]
fn move_relocates_the_source() {
    let sandbox = Sandbox::new();
    sandbox.write("a.txt", "1");
    fs::create_dir(sandbox.dir.join("d")).unwrap();
    let jobs = Jobs::new();
    let id = start_job(&sandbox.root(), &jobs, "move", &paths(&["a.txt"]), "d").unwrap();
    assert_eq!(wait_until_done(&jobs, &id), "done");
    assert!(!sandbox.dir.join("a.txt").exists() && sandbox.dir.join("d/a.txt").exists());
}
