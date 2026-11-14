//! Informations sur les règles et le système utilisées par le service : attente de stabilité, disques montés.

use super::time_calc::seconds;
use super::types_action::RuleAction;
use super::types_rule::{Rule, Trigger};
use crate::disks::mounted_disks;
use crate::root::Root;

/// Attente de stabilité la plus longue demandée par les règles (au moins 2 secondes).
pub fn max_wait(rules: &[Rule]) -> f64 {
    let waits = rules.iter().flat_map(|r| {
        let triggers = r.triggers.iter().filter_map(|t| if let Trigger::Stable { wait } = t { Some(seconds(wait)) } else { None });
        let actions = r.actions.iter().filter_map(|a| if let RuleAction::WaitStable { wait } = a { Some(seconds(wait)) } else { None });
        triggers.chain(actions).collect::<Vec<_>>()
    });
    waits.fold(2.0, f64::max)
}

/// Noms des disques montés.
pub fn mount_names(root: &Root) -> Vec<String> {
    mounted_disks(root).iter().filter_map(|d| d["n"].as_str().map(String::from)).collect()
}
