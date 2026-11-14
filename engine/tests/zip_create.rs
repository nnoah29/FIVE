mod support;

use std::fs;
use std::thread::sleep;
use std::time::{Duration, Instant};

use five_engine::archive::list_archive;
use five_engine::jobs::registry::Jobs;
use five_engine::jobs::start::start_job;
use support::Sandbox;

fn paths(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn run(sandbox: &Sandbox, jobs: &Jobs, kind: &str, src: &[&str], dest: &str) -> String {
    let id = start_job(&sandbox.root(), jobs, kind, &paths(src), dest).unwrap();
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
fn zip_contains_files_and_folders_under_the_item_name() {
    let sandbox = Sandbox::new();
    sandbox.write("src/a.txt", "AAA");
    sandbox.write("src/sous/b.txt", "BB");
    fs::create_dir_all(sandbox.dir.join("src/vide")).unwrap();
    fs::create_dir(sandbox.dir.join("out")).unwrap();
    assert_eq!(run(&sandbox, &Jobs::new(), "zip", &["src"], "out"), "done");
    let listing = list_archive(&sandbox.dir.join("out/src.zip")).unwrap();
    let names: Vec<&str> = listing["items"].as_array().unwrap().iter().map(|i| i["n"].as_str().unwrap()).collect();
    assert_eq!(names, ["src/", "src/a.txt", "src/sous/", "src/sous/b.txt", "src/vide/"]);
}

#[test]
fn zip_names_follow_the_selection_and_never_overwrite() {
    let sandbox = Sandbox::new();
    sandbox.write("a.txt", "1");
    sandbox.write("b.txt", "2");
    fs::create_dir(sandbox.dir.join("out")).unwrap();
    let jobs = Jobs::new();
    assert_eq!(run(&sandbox, &jobs, "zip", &["a.txt"], "out"), "done");
    assert_eq!(run(&sandbox, &jobs, "zip", &["a.txt"], "out"), "done");
    assert_eq!(run(&sandbox, &jobs, "zip", &["a.txt", "b.txt"], "out"), "done");
    for name in ["a.txt.zip", "a.txt (2).zip", "Archive.zip"] {
        assert!(sandbox.dir.join("out").join(name).is_file(), "{name}");
    }
}

#[test]
fn zip_of_nothing_is_refused() {
    let sandbox = Sandbox::new();
    fs::create_dir(sandbox.dir.join("out")).unwrap();
    let refus = start_job(&sandbox.root(), &Jobs::new(), "zip", &[], "out").unwrap_err();
    assert_eq!(refus.code, 400);
}

#[test]
fn zip_skips_symbolic_links_and_its_own_archive() {
    let sandbox = Sandbox::new();
    sandbox.write("src/a.txt", "AAA");
    std::os::unix::fs::symlink("/etc/passwd", sandbox.dir.join("src/lien")).unwrap();
    assert_eq!(run(&sandbox, &Jobs::new(), "zip", &["src"], "src"), "done");
    let listing = list_archive(&sandbox.dir.join("src/src.zip")).unwrap();
    let names: Vec<&str> = listing["items"].as_array().unwrap().iter().map(|i| i["n"].as_str().unwrap()).collect();
    assert_eq!(names, ["src/", "src/a.txt"]);
}
