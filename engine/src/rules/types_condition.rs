//! Catalogue des conditions d'une règle (spécification `docs/REGLES.md`, §2.2).

use serde::{Deserialize, Serialize};

use super::types_matcher::{Cmp, DurationSpec, TextMatcher, Weekday};

/// Condition : un test, éventuellement inversé.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    #[serde(default)]
    pub negated: bool,
    pub test: ConditionTest,
}

/// Suite de conditions toutes vraies (ET).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionGroup {
    pub conditions: Vec<Condition>,
}

/// Un test sur un fichier ou un dossier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConditionTest {
    Name {
        matcher: TextMatcher,
    },
    Stem {
        matcher: TextMatcher,
    },
    Extension {
        in_list: Vec<String>,
    },
    NameLength {
        cmp: Cmp,
        n: u64,
    },
    NameHasSpaces,
    NameHasUppercase,
    NameHasAccents,
    NameHasDateLike,
    Path {
        matcher: TextMatcher,
    },
    ParentName {
        matcher: TextMatcher,
    },
    InFolder {
        path: String,
        #[serde(default)]
        recursive: bool,
    },
    Depth {
        cmp: Cmp,
        n: u64,
    },
    InsideGitRepo,
    InsideTrash,
    IsFile,
    IsFolder,
    IsSymlink,
    IsBrokenSymlink,
    IsHidden,
    IsExecutable,
    IsReadOnly,
    IsEmpty,
    IsMountPoint,
    Kind {
        in_list: Vec<String>,
    },
    Mime {
        matcher: TextMatcher,
    },
    Size {
        cmp: Cmp,
        bytes: u64,
    },
    FolderTotalSize {
        cmp: Cmp,
        bytes: u64,
    },
    ChildCount {
        cmp: Cmp,
        n: u64,
    },
    FileCount {
        cmp: Cmp,
        n: u64,
    },
    ModifiedBefore {
        at: f64,
    },
    ModifiedAfter {
        at: f64,
    },
    ModifiedWithin {
        duration: DurationSpec,
    },
    OlderThan {
        duration: DurationSpec,
    },
    CreatedBefore {
        at: f64,
    },
    CreatedAfter {
        at: f64,
    },
    AccessedOlderThan {
        duration: DurationSpec,
    },
    ModifiedOnWeekday {
        days: Vec<Weekday>,
    },
    ModifiedInMonth {
        months: Vec<u8>,
    },
    ModifiedInYear {
        years: Vec<i32>,
    },
    NeverOpened,
    TextContains {
        matcher: TextMatcher,
    },
    LineCount {
        cmp: Cmp,
        n: u64,
    },
    FirstLine {
        matcher: TextMatcher,
    },
    HasShebang,
    Encoding {
        value: String,
    },
    ImageWidth {
        cmp: Cmp,
        px: u64,
    },
    ImageHeight {
        cmp: Cmp,
        px: u64,
    },
    Orientation {
        value: String,
    },
    Megapixels {
        cmp: Cmp,
        n: f64,
    },
    HasAlpha,
    Duration {
        cmp: Cmp,
        seconds: f64,
    },
    VideoResolution {
        cmp: Cmp,
        px: u64,
    },
    HasAudioTrack,
    ArchiveIsReadable,
    ArchiveEntryCount {
        cmp: Cmp,
        n: u64,
    },
    ArchiveContainsName {
        matcher: TextMatcher,
    },
    ArchiveIsEncrypted,
    DuplicateOfAnother {
        scope: String,
    },
    IsNewestOfDuplicates {
        scope: String,
    },
    IsOldestOfDuplicates {
        scope: String,
    },
    SameNameAs {
        scope: String,
    },
    GitStatus {
        in_list: Vec<String>,
    },
    DownloadedFrom {
        matcher: TextMatcher,
    },
    FromDisk {
        disk: String,
    },
    FreeSpaceBelow {
        bytes: u64,
        on: String,
    },
    DiskIsMounted {
        disk: String,
    },
    OwnedByMe,
    Permissions {
        mode: u32,
    },
    IsFavorite,
    InRecents,
    IsPinnedFolder,
    AllOf {
        tests: Vec<Condition>,
    },
    AnyOf {
        tests: Vec<Condition>,
    },
    NoneOf {
        tests: Vec<Condition>,
    },
}
