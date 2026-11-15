mod support;

use five_engine::rules::state::{RulesState, Tracked, load_state, save_state};
use support::Sandbox;

#[test]
fn file_times_survive_the_state_file_exactly() {
    let sandbox = Sandbox::new();
    let path = sandbox.dir.join("etat-regles.json");
    let mut state = RulesState::default();
    for i in 0..2000u32 {
        let mtime = 1_794_750_290.0 + f64::from(i) * 0.7354367 + f64::from(i % 7) * 1e-7;
        state.tracked.insert(format!("Dl/{i}.pdf"), Tracked { size: 1, mtime, since: 1.0, done: false });
    }
    save_state(&path, &state).unwrap();
    let loaded = load_state(&path);
    let wrong = state.tracked.iter().filter(|(k, v)| loaded.tracked[*k].mtime != v.mtime).count();
    assert_eq!(wrong, 0);
}
