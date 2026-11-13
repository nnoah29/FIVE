mod support;

use std::path::Path;
use std::process::Command;

use five_engine::git_marks::git_marks;
use support::Sandbox;

fn git(folder: &Path, args: &[&str]) {
    let status = Command::new("git").args(["-c", "user.name=t", "-c", "user.email=t@t"]).args(args).current_dir(folder).output().unwrap().status;
    assert!(status.success(), "git {args:?}");
}

#[test]
fn marks_modified_untracked_and_ignored_children() {
    let sandbox = Sandbox::new();
    sandbox.write("suivi.txt", "1");
    sandbox.write("modifie.txt", "1");
    sandbox.write(".gitignore", "ignore.log\n");
    git(&sandbox.dir, &["init", "-q"]);
    git(&sandbox.dir, &["add", "."]);
    git(&sandbox.dir, &["commit", "-q", "-m", "init"]);
    sandbox.write("modifie.txt", "2");
    sandbox.write("neuf.txt", "1");
    sandbox.write("ignore.log", "1");
    let marks = git_marks(&sandbox.dir);
    assert_eq!(marks.get("modifie.txt"), Some(&'M'));
    assert_eq!(marks.get("neuf.txt"), Some(&'?'));
    assert_eq!(marks.get("ignore.log"), Some(&'!'));
    assert_eq!(marks.get("suivi.txt"), None);
}

#[test]
fn folders_take_the_strongest_mark_of_their_content() {
    let sandbox = Sandbox::new();
    sandbox.write("d/a.txt", "1");
    git(&sandbox.dir, &["init", "-q"]);
    git(&sandbox.dir, &["add", "."]);
    git(&sandbox.dir, &["commit", "-q", "-m", "init"]);
    sandbox.write("d/a.txt", "2");
    sandbox.write("d/nouveau.txt", "1");
    assert_eq!(git_marks(&sandbox.dir).get("d"), Some(&'M'));
}

#[test]
fn outside_a_repository_there_are_no_marks() {
    let sandbox = Sandbox::new();
    sandbox.write("a.txt", "1");
    assert!(git_marks(&sandbox.dir).is_empty());
}
