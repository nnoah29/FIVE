mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use five_engine::jobs::registry::Jobs;
use five_engine::jobs::start::start_job;
use support::Sandbox;

fn run(sandbox: &Sandbox, jobs: &Jobs, kind: &str, src: &[&str], dest: &str) -> String {
    let rels: Vec<String> = src.iter().map(|s| s.to_string()).collect();
    let id = start_job(&sandbox.root(), jobs, kind, &rels, dest).unwrap();
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
fn zip_then_unzip_restores_content_folders_and_permissions() {
    let sandbox = Sandbox::new();
    let script = sandbox.write("src/run.sh", "#!/bin/sh\n");
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    sandbox.write("src/sous/é à.txt", &"x".repeat(3_000_000));
    fs::create_dir_all(sandbox.dir.join("src/vide")).unwrap();
    fs::create_dir_all(sandbox.dir.join("out")).unwrap();
    fs::create_dir_all(sandbox.dir.join("ext")).unwrap();
    let jobs = Jobs::new();
    assert_eq!(run(&sandbox, &jobs, "zip", &["src"], "out"), "done");
    assert_eq!(run(&sandbox, &jobs, "unzip", &["out/src.zip"], "ext"), "done");
    assert_eq!(fs::read_to_string(sandbox.dir.join("ext/src/src/sous/é à.txt")).unwrap().len(), 3_000_000);
    assert!(sandbox.dir.join("ext/src/src/vide").is_dir());
    let mode = fs::metadata(sandbox.dir.join("ext/src/src/run.sh")).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o755);
}

#[test]
fn unzip_never_overwrites_an_existing_folder() {
    let sandbox = Sandbox::new();
    sandbox.write("a.txt", "AAA");
    fs::create_dir_all(sandbox.dir.join("out")).unwrap();
    let jobs = Jobs::new();
    assert_eq!(run(&sandbox, &jobs, "zip", &["a.txt"], "out"), "done");
    assert_eq!(run(&sandbox, &jobs, "unzip", &["out/a.txt.zip"], "out"), "done");
    assert_eq!(run(&sandbox, &jobs, "unzip", &["out/a.txt.zip"], "out"), "done");
    assert!(sandbox.dir.join("out/a.txt/a.txt").is_file());
    assert!(sandbox.dir.join("out/a.txt (2)/a.txt").is_file());
}

#[test]
fn tar_and_tar_gz_are_extracted_into_a_folder_named_after_the_archive() {
    let sandbox = Sandbox::new();
    sandbox.write("pack/n.txt", "hello");
    sandbox.write("pack/sous/m.txt", "world");
    for (flag, name) in [("-cf", "pack.tar"), ("-czf", "pack.tgz")] {
        let status = Command::new("tar").current_dir(&sandbox.dir).args([flag, name, "pack"]).status().unwrap();
        assert!(status.success());
    }
    fs::create_dir_all(sandbox.dir.join("ext")).unwrap();
    let jobs = Jobs::new();
    assert_eq!(run(&sandbox, &jobs, "unzip", &["pack.tar", "pack.tgz"], "ext"), "done");
    assert_eq!(fs::read_to_string(sandbox.dir.join("ext/pack/pack/sous/m.txt")).unwrap(), "world");
    assert_eq!(fs::read_to_string(sandbox.dir.join("ext/pack (2)/pack/n.txt")).unwrap(), "hello");
}

#[test]
fn unzip_refuses_a_file_that_is_not_an_archive() {
    let sandbox = Sandbox::new();
    sandbox.write("notes.txt", "pas une archive");
    fs::create_dir_all(sandbox.dir.join("ext")).unwrap();
    let refus = start_job(&sandbox.root(), &Jobs::new(), "unzip", &["notes.txt".to_string()], "ext").unwrap_err();
    assert_eq!(refus.code, 400);
}

#[test]
fn archives_made_by_the_system_zip_tools_can_be_read() {
    let sandbox = Sandbox::new();
    sandbox.write("src/a.txt", "AAA");
    let made = Command::new("python3")
        .current_dir(&sandbox.dir)
        .args(["-c", "import zipfile; z = zipfile.ZipFile('ref.zip', 'w', zipfile.ZIP_DEFLATED); z.write('src/a.txt'); z.close()"])
        .status();
    if made.is_ok_and(|s| s.success()) {
        fs::create_dir_all(sandbox.dir.join("ext")).unwrap();
        assert_eq!(run(&sandbox, &Jobs::new(), "unzip", &["ref.zip"], "ext"), "done");
        assert_eq!(fs::read_to_string(sandbox.dir.join("ext/ref/src/a.txt")).unwrap(), "AAA");
    }
}
