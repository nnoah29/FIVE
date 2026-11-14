//! Catalogue des actions d'une règle (spécification `docs/REGLES.md`, §2.3).

use serde::{Deserialize, Serialize};

use super::types_matcher::DurationSpec;

/// Mise en forme de la casse d'un nom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseMode {
    Lower,
    Upper,
    Title,
    Sentence,
}

/// Où insérer un texte dans un nom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    Prefix,
    Suffix,
}

/// Date utilisée par une action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateSource {
    Modified,
    Created,
    Today,
    Name,
}

/// Quel doublon garder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Keep {
    Newest,
    Oldest,
    ShortestName,
}

/// Une archive par élément, ou une seule pour tous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZipMode {
    OnePerItem,
    OneForAll,
}

/// Sort de l'archive après extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AfterUnzip {
    Keep,
    TrashArchive,
}

/// Une action appliquée à un fichier qui satisfait les conditions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleAction {
    MoveTo {
        folder: String,
    },
    CopyTo {
        folder: String,
    },
    MoveToDated {
        folder: String,
        pattern: String,
    },
    MoveToByKind {
        base: String,
    },
    MoveToByExtension {
        base: String,
    },
    Rename {
        template: String,
    },
    ReplaceInName {
        find: String,
        replace: String,
        #[serde(default)]
        regex: bool,
    },
    ChangeCase {
        mode: CaseMode,
    },
    Slugify,
    AddPrefix {
        text: String,
    },
    AddSuffix {
        text: String,
    },
    AddDate {
        from: DateSource,
        position: Position,
        format: String,
    },
    AddCounter {
        start: u32,
        width: u8,
    },
    RemoveSpaces,
    NormalizeExtension,
    Trash,
    TrashDuplicates {
        keep: Keep,
    },
    EmptyTrashOlderThan {
        duration: DurationSpec,
    },
    RemoveEmptyFolders,
    Zip {
        mode: ZipMode,
        #[serde(default)]
        name_template: Option<String>,
    },
    Unzip {
        then: AfterUnzip,
    },
    ZipFolderThenTrash,
    CreateFolder {
        template: String,
    },
    FlattenFolder,
    SetReadOnly {
        value: bool,
    },
    SetExecutable {
        value: bool,
    },
    SetPermissions {
        mode: u32,
    },
    SetModifiedTime {
        from: DateSource,
    },
    PinFolderInSidebar,
    AddToFavorites,
    RemoveFromFavorites,
    Hide {
        value: bool,
    },
    OpenWithDefault,
    RevealInFiveFolder,
    Notify {
        message_template: String,
    },
    NotifyOnlyOnce,
    StopProcessing,
    SkipRemainingActions,
    WaitStable {
        #[serde(rename = "for")]
        wait: DurationSpec,
    },
    LogOnly,
}
