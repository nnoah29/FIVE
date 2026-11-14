mod support;

use five_engine::rules::changes::{ChangeKind, detect};
use five_engine::rules::scan::Seen;
use five_engine::rules::state::RulesState;

fn seen(path: &str, size: u64, mtime: f64) -> Seen {
    Seen { path: path.into(), size, mtime, is_dir: false }
}

#[test]
fn the_first_pass_only_remembers_what_is_already_there() {
    let mut state = RulesState::default();
    let files = [seen("Dl/a.txt", 1, 10.0)];
    assert!(detect(&mut state, &files, 100.0, 2.0, true).is_empty());
    assert!(state.baselined && state.tracked.contains_key("Dl/a.txt"));
}

#[test]
fn new_and_modified_files_are_reported_once() {
    let mut state = RulesState::default();
    detect(&mut state, &[], 0.0, 2.0, true);
    let created = detect(&mut state, &[seen("Dl/a.txt", 1, 10.0)], 100.0, 2.0, true);
    assert_eq!((created.len(), created[0].kind), (1, ChangeKind::Created));
    let modified = detect(&mut state, &[seen("Dl/a.txt", 5, 11.0)], 101.0, 2.0, true);
    assert_eq!(modified[0].kind, ChangeKind::Modified);
}

#[test]
fn a_file_is_reported_stable_after_two_seconds_then_no_longer_after_the_longest_wait() {
    let mut state = RulesState::default();
    detect(&mut state, &[], 0.0, 30.0, true);
    let file = [seen("Dl/a.txt", 1, 10.0)];
    detect(&mut state, &file, 100.0, 30.0, true);
    assert!(detect(&mut state, &file, 101.0, 30.0, true).is_empty());
    let stable = detect(&mut state, &file, 103.0, 30.0, true);
    assert_eq!((stable[0].kind, stable[0].stable_for), (ChangeKind::Stable, 3.0));
    assert_eq!(detect(&mut state, &file, 131.0, 30.0, true).len(), 1);
    assert!(detect(&mut state, &file, 140.0, 30.0, true).is_empty());
}

#[test]
fn vanished_files_are_forgotten_only_after_a_complete_scan() {
    let mut state = RulesState::default();
    detect(&mut state, &[seen("Dl/a.txt", 1, 1.0)], 0.0, 2.0, true);
    detect(&mut state, &[], 10.0, 2.0, false);
    assert!(state.tracked.contains_key("Dl/a.txt"));
    detect(&mut state, &[], 20.0, 2.0, true);
    assert!(state.tracked.is_empty());
}
