mod support;

use std::time::{Duration, Instant};

use five_engine::rules::scan::scan_watch;
use five_engine::rules::types_rule::Watch;
use support::Sandbox;

fn names(sandbox: &Sandbox, watch: Watch) -> Vec<String> {
    let mut out = Vec::new();
    assert!(scan_watch(&sandbox.root(), &watch, Instant::now() + Duration::from_secs(5), &mut out));
    let mut paths: Vec<String> = out.into_iter().map(|s| s.path).collect();
    paths.sort();
    paths
}

#[test]
fn the_scan_skips_hidden_items_git_and_dependency_folders() {
    let sandbox = Sandbox::new();
    for file in ["Dl/a.txt", "Dl/sous/b.txt", "Dl/.cache/c.txt", "Dl/.git/HEAD", "Dl/node_modules/m.js", "Dl/.cache2"] {
        sandbox.write(file, "x");
    }
    let watch = Watch { path: "Dl".into(), recursive: true, max_depth: None, ignore_hidden: true };
    assert_eq!(names(&sandbox, watch.clone()), ["Dl/a.txt", "Dl/node_modules", "Dl/sous", "Dl/sous/b.txt"]);
    let flat = Watch { recursive: false, ..watch };
    assert_eq!(names(&sandbox, flat), ["Dl/a.txt", "Dl/node_modules", "Dl/sous"]);
}
