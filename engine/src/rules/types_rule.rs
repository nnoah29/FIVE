//! Une règle : dossiers surveillés, déclencheurs, conditions, actions (spécification `docs/REGLES.md`, §2).

use serde::{Deserialize, Serialize};

use super::types_action::RuleAction;
use super::types_condition::ConditionGroup;
use super::types_matcher::{DurationSpec, Weekday};

/// Version du format des règles écrit sur disque.
pub const SCHEMA_VERSION: u32 = 1;

/// Valeur par défaut des indicateurs à vrai.
fn yes() -> bool {
    true
}

/// Dossier surveillé, relatif à la racine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Watch {
    pub path: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub max_depth: Option<u32>,
    #[serde(default = "yes")]
    pub ignore_hidden: bool,
}

/// Ce qui fait agir une règle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    Created,
    Modified,
    MovedIn,
    Stable {
        #[serde(rename = "for")]
        wait: DurationSpec,
    },
    Schedule {
        every: DurationSpec,
    },
    Daily {
        at: String,
    },
    Weekly {
        day: Weekday,
        at: String,
    },
    OnStart,
    OnMount {
        disk: String,
    },
    Manual,
}

/// Que faire quand la destination existe déjà (jamais d'écrasement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    #[default]
    Rename,
    Skip,
    Ask,
}

/// Règle automatique : si une alternative est vraie, appliquer les actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default = "yes")]
    pub trial: bool,
    pub watch: Vec<Watch>,
    pub triggers: Vec<Trigger>,
    pub alternatives: Vec<ConditionGroup>,
    pub actions: Vec<RuleAction>,
    #[serde(default)]
    pub on_conflict: ConflictPolicy,
    #[serde(default)]
    pub schema_version: u32,
}
