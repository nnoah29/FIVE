mod support;

use std::fs;

use five_engine::jobs::registry::Jobs;
use five_engine::jobs::start::{control_job, start_job};
use support::Sandbox;

fn paths(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn start_refuses_invalid_requests() {
    let sandbox = Sandbox::new();
    sandbox.write("d/x", "1");
    sandbox.write("f", "1");
    let (root, jobs) = (sandbox.root(), Jobs::new());
    let code = |kind: &str, src: &[&str], dest: &str| start_job(&root, &jobs, kind, &paths(src), dest).unwrap_err().code;
    assert_eq!(code("zip", &["f"], "d"), 400);
    assert_eq!(code("copy", &["d"], "d/sous"), 400);
    assert_eq!(code("copy", &["f"], "f"), 400);
    assert_eq!(code("move", &["d/x"], "d"), 400);
    assert_eq!(code("copy", &["f"], "../.."), 403);
    assert_eq!(code("copy", &["absent"], "d"), 404);
}

#[test]
fn control_rejects_unknown_jobs_and_actions() {
    let sandbox = Sandbox::new();
    sandbox.write("a", "1");
    fs::create_dir(sandbox.dir.join("d")).unwrap();
    let jobs = Jobs::new();
    assert_eq!(control_job(&jobs, "inconnu", "pause").unwrap_err().code, 404);
    let id = start_job(&sandbox.root(), &jobs, "copy", &paths(&["a"]), "d").unwrap();
    assert_eq!(control_job(&jobs, &id, "danse").unwrap_err().code, 400);
}
