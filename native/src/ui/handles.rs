//! Widgets de la fenêtre dont le reste de l'application a besoin.

use gtk4::{Box as GtkBox, Button, Entry, Label, Overlay, ScrolledWindow, Stack};

/// Barre latérale.
pub struct SideUi {
    pub root: GtkBox,
    pub count: Label,
    pub count_label: Label,
    pub new_folder: Button,
    pub up: Button,
    pub places: GtkBox,
    pub disks: GtkBox,
}

/// Barre du haut.
pub struct TopUi {
    pub root: GtkBox,
    pub menu: Button,
    pub search: Entry,
    pub hidden: Button,
    pub theme: Button,
    pub gear: Button,
    pub avatar: Label,
    pub user_name: Label,
    pub user_root: Label,
    pub user: GtkBox,
    pub who: GtkBox,
}

/// Section des dossiers : titre, fil d'Ariane, cartes.
pub struct FoldersUi {
    pub root: GtkBox,
    pub title: Label,
    pub crumbs: GtkBox,
    pub cards: GtkBox,
}

/// Section des fichiers : filtres, en-tête de colonnes, liste.
pub struct FilesUi {
    pub root: GtkBox,
    pub title: Label,
    pub pills: GtkBox,
    pub paste: Button,
    pub view_list: Button,
    pub view_grid: Button,
    pub thead: GtkBox,
    pub items: GtkBox,
    pub scroll: ScrolledWindow,
}

/// Panneau de droite : espace et aperçu.
pub struct WorkUi {
    pub root: Overlay,
    pub body: GtkBox,
    pub close: Button,
}

/// Tous les widgets de la fenêtre.
pub struct Ui {
    pub window: gtk4::ApplicationWindow,
    pub overlay: Overlay,
    pub side: SideUi,
    pub side_scroll: ScrolledWindow,
    pub page: GtkBox,
    pub scrim: GtkBox,
    pub top: TopUi,
    pub folders: FoldersUi,
    pub files: FilesUi,
    pub work: WorkUi,
    pub stack: Stack,
    pub content: GtkBox,
    pub settings: GtkBox,
    pub toast: Label,
    pub transfers: GtkBox,
    pub lower: Overlay,
    pub dock: GtkBox,
    pub dock_bar: GtkBox,
}
