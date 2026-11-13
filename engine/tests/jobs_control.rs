mod support;

use std::fs;
use std::thread::sleep;
use std::time::Duration;

use five_engine::jobs::registry::Jobs;
use five_engine::jobs::start::{control_job, start_job};
use support::Sandbox;

fn big_file(sandbox: &Sandbox) {
    let path = sandbox.dir.join("gros.bin");
    fs::write(path, vec![7u8; 64 * 1024 * 1024]).unwrap();
    fs::create_dir(sandbox.dir.join("d")).unwrap();
}

fn state_of(jobs: &Jobs) -> String {
    jobs.view()[0]["state"].as_str().unwrap().to_string()
}

#[test]
fn pause_then_resume_completes_the_copy() {
    let sandbox = Sandbox::new();
    big_file(&sandbox);
    let jobs = Jobs::new();
    let id = start_job(&sandbox.root(), &jobs, "copy", &["gros.bin".to_string()], "d").unwrap();
    control_job(&jobs, &id, "pause").unwrap();
    sleep(Duration::from_millis(100));
    let frozen = jobs.view()[0]["done"].as_u64().unwrap();
    sleep(Duration::from_millis(100));
    assert_eq!(jobs.view()[0]["done"].as_u64().unwrap(), frozen);
    control_job(&jobs, &id, "resume").unwrap();
    while state_of(&jobs) == "running" {
        sleep(Duration::from_millis(10));
    }
    assert_eq!(state_of(&jobs), "done");
    assert_eq!(fs::metadata(sandbox.dir.join("d/gros.bin")).unwrap().len(), 64 * 1024 * 1024);
}

#[test]
fn cancel_removes_the_partial_file() {
    let sandbox = Sandbox::new();
    big_file(&sandbox);
    let jobs = Jobs::new();
    let id = start_job(&sandbox.root(), &jobs, "copy", &["gros.bin".to_string()], "d").unwrap();
    control_job(&jobs, &id, "pause").unwrap();
    control_job(&jobs, &id, "cancel").unwrap();
    sleep(Duration::from_millis(300));
    assert_eq!(state_of(&jobs), "cancelled");
    assert!(!sandbox.dir.join("d/gros.bin").exists());
    assert!(sandbox.dir.join("gros.bin").exists());
}
