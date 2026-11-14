mod support;

use std::fs::{self, File};
use std::time::{Duration, UNIX_EPOCH};

use five_engine::rules::context_build::{BuildEnv, Extras, build_context};
use five_engine::rules::needs::Needs;
use five_engine::rules::types_rule::Watch;
use support::Sandbox;

fn env<'a>(sandbox: &'a Sandbox, root: &'a five_engine::root::Root, needs: Needs, extras: &'a Extras, watches: &'a [Watch]) -> BuildEnv<'a> {
    let _ = sandbox;
    BuildEnv { root, needs, extras, watches, disks: &[], now: 1_700_000_000.0 }
}

#[test]
fn metadata_fields_are_always_filled() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    sandbox.write("Dl/.cache.txt", "abc");
    fs::create_dir_all(sandbox.dir.join("Dl/vide")).unwrap();
    std::os::unix::fs::symlink("/nulle/part", sandbox.dir.join("Dl/lien")).unwrap();
    let extras = Extras::default();
    let e = env(&sandbox, &root, Needs::default(), &extras, &[]);
    let file = build_context(&e, "Dl/.cache.txt", None, 0).unwrap();
    assert_eq!((file.size, file.hidden(), file.kind.as_str(), file.owned_by_me), (3, true, "document", true));
    assert!(file.modified.is_some() && file.mime.as_deref() == Some("text/plain") && file.text.is_none());
    let folder = build_context(&e, "Dl/vide", None, 0).unwrap();
    assert!(folder.is_dir && folder.empty && folder.mime.is_none());
    assert!(build_context(&e, "Dl/lien", None, 0).unwrap().broken_symlink);
    assert!(build_context(&e, "Dl/absent", None, 0).is_none() && build_context(&e, "../hors", None, 0).is_none());
}

#[test]
fn costly_information_is_read_only_when_a_rule_needs_it() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    sandbox.write("Dl/note.md", "bonjour\nmonde\n");
    let extras = Extras::default();
    let needs = Needs { text: true, hash: true, ..Needs::default() };
    let ctx = build_context(&env(&sandbox, &root, needs, &extras, &[]), "Dl/note.md", Some(5.0), 3).unwrap();
    assert_eq!((ctx.text.as_deref(), ctx.encoding.as_deref(), ctx.stable_for, ctx.counter), (Some("bonjour\nmonde\n"), Some("utf8"), Some(5.0), 3));
    assert_eq!(ctx.hash.as_deref().map(str::len), Some(64));
}

#[test]
fn archives_and_images_are_described_from_their_content() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    let png = [b"\x89PNG\r\n\x1a\n".to_vec(), vec![0, 0, 0, 13], b"IHDR".to_vec(), vec![0, 0, 3, 32, 0, 0, 2, 88, 8, 6]].concat();
    fs::create_dir_all(sandbox.dir.join("Dl")).unwrap();
    fs::write(sandbox.dir.join("Dl/p.png"), png).unwrap();
    let extras = Extras::default();
    let needs = Needs { image: true, archive: true, ..Needs::default() };
    let image = build_context(&env(&sandbox, &root, needs, &extras, &[]), "Dl/p.png", None, 0).unwrap().image.unwrap();
    assert_eq!((image.width, image.height, image.has_alpha), (800, 600, true));
}

#[test]
fn duplicates_are_found_by_scope_with_newest_and_oldest() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    for (path, content) in [("Dl/a.bin", "SAME"), ("Dl/sous/b.bin", "SAME"), ("Dl/c.bin", "AUTRE"), ("Ailleurs/d.bin", "SAME")] {
        sandbox.write(path, content);
    }
    for (path, age) in [("Dl/a.bin", 1000), ("Dl/sous/b.bin", 2000), ("Ailleurs/d.bin", 3000)] {
        File::open(sandbox.dir.join(path)).unwrap().set_modified(UNIX_EPOCH + Duration::from_secs(age)).unwrap();
    }
    let watches = [Watch { path: "Dl".into(), recursive: true, max_depth: None, ignore_hidden: true }];
    let extras = Extras::default();
    let needs = Needs { duplicates: true, ..Needs::default() };
    let ctx = build_context(&env(&sandbox, &root, needs, &extras, &watches), "Dl/a.bin", None, 0).unwrap();
    let dups = ctx.duplicates.unwrap();
    assert_eq!((dups.folder.others, dups.watched.others, dups.root.others), (0, 1, 2));
    assert!(dups.watched.oldest && !dups.watched.newest && dups.root.oldest);
    assert_eq!(dups.root.same_name, 0);
}

#[test]
fn interface_knowledge_comes_from_the_extras() {
    let sandbox = Sandbox::new();
    let root = sandbox.root();
    sandbox.write("Dl/a.txt", "x");
    let mut extras = Extras::default();
    extras.favorites.insert("Dl/a.txt".into());
    let ctx = build_context(&env(&sandbox, &root, Needs::default(), &extras, &[]), "Dl/a.txt", None, 0).unwrap();
    assert!(ctx.favorite && !ctx.recent && !ctx.pinned);
}
