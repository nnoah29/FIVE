//! Libellés français des déclencheurs, conditions, actions, statuts et issues des règles (le moteur n'en contient aucun).

const TRIGGERS: [(&str, &str); 10] = [
    ("created", "un fichier apparaît"),
    ("modified", "un fichier est modifié"),
    ("moved_in", "un élément arrive"),
    ("stable", "un fichier ne bouge plus"),
    ("schedule", "à intervalle régulier"),
    ("daily", "chaque jour"),
    ("weekly", "chaque semaine"),
    ("on_start", "au démarrage de FIVE"),
    ("on_mount", "quand un disque est monté"),
    ("manual", "à la demande seulement"),
];

const CONDITIONS: [(&str, &str); 74] = [
    ("name", "Le nom"),
    ("stem", "Le nom sans extension"),
    ("extension", "L'extension est parmi"),
    ("name_length", "La longueur du nom"),
    ("name_has_spaces", "Le nom contient des espaces"),
    ("name_has_uppercase", "Le nom contient des majuscules"),
    ("name_has_accents", "Le nom contient des accents"),
    ("name_has_date_like", "Le nom contient une date"),
    ("path", "Le chemin"),
    ("parent_name", "Le nom du dossier parent"),
    ("in_folder", "Est dans le dossier"),
    ("depth", "La profondeur"),
    ("inside_git_repo", "Est dans un dépôt Git"),
    ("inside_trash", "Est dans la corbeille"),
    ("is_file", "Est un fichier"),
    ("is_folder", "Est un dossier"),
    ("is_symlink", "Est un lien symbolique"),
    ("is_broken_symlink", "Est un lien cassé"),
    ("is_hidden", "Est caché"),
    ("is_executable", "Est exécutable"),
    ("is_read_only", "Est en lecture seule"),
    ("is_empty", "Est vide"),
    ("is_mount_point", "Est un point de montage"),
    ("kind", "La famille est parmi"),
    ("mime", "Le type MIME"),
    ("size", "La taille"),
    ("folder_total_size", "La taille totale du dossier"),
    ("child_count", "Le nombre d'enfants"),
    ("file_count", "Le nombre de fichiers"),
    ("modified_before", "Modifié avant"),
    ("modified_after", "Modifié après"),
    ("modified_within", "Modifié depuis moins de"),
    ("older_than", "Modifié depuis plus de"),
    ("created_before", "Créé avant"),
    ("created_after", "Créé après"),
    ("accessed_older_than", "Ouvert depuis plus de"),
    ("modified_on_weekday", "Modifié un de ces jours"),
    ("modified_in_month", "Modifié durant ces mois"),
    ("modified_in_year", "Modifié durant ces années"),
    ("never_opened", "N'a jamais été ouvert"),
    ("text_contains", "Le texte"),
    ("line_count", "Le nombre de lignes"),
    ("first_line", "La première ligne"),
    ("has_shebang", "Commence par #!"),
    ("encoding", "L'encodage"),
    ("image_width", "La largeur de l'image"),
    ("image_height", "La hauteur de l'image"),
    ("orientation", "L'orientation de l'image"),
    ("megapixels", "Les mégapixels"),
    ("has_alpha", "L'image a de la transparence"),
    ("duration", "La durée"),
    ("video_resolution", "La résolution vidéo"),
    ("has_audio_track", "A une piste son"),
    ("archive_is_readable", "L'archive est lisible"),
    ("archive_entry_count", "Le nombre d'entrées de l'archive"),
    ("archive_contains_name", "L'archive contient un nom"),
    ("archive_is_encrypted", "L'archive est chiffrée"),
    ("duplicate_of_another", "Est un doublon"),
    ("is_newest_of_duplicates", "Est le plus récent des doublons"),
    ("is_oldest_of_duplicates", "Est le plus ancien des doublons"),
    ("same_name_as", "Un autre fichier porte le même nom"),
    ("git_status", "L'état Git est parmi"),
    ("downloaded_from", "Téléchargé depuis"),
    ("from_disk", "Vient du disque"),
    ("free_space_below", "L'espace libre est inférieur à"),
    ("disk_is_mounted", "Le disque est monté"),
    ("owned_by_me", "M'appartient"),
    ("permissions", "Les droits valent"),
    ("is_favorite", "Est un favori"),
    ("in_recents", "Est dans les récents"),
    ("is_pinned_folder", "Est un dossier épinglé"),
    ("all_of", "Toutes ces conditions"),
    ("any_of", "Au moins une de ces conditions"),
    ("none_of", "Aucune de ces conditions"),
];

const ACTIONS: [(&str, &str); 40] = [
    ("move_to", "Déplacer vers"),
    ("copy_to", "Copier vers"),
    ("move_to_dated", "Déplacer vers un dossier daté"),
    ("move_to_by_kind", "Déplacer vers un dossier par famille"),
    ("move_to_by_extension", "Déplacer vers un dossier par extension"),
    ("rename", "Renommer avec un modèle"),
    ("replace_in_name", "Remplacer du texte dans le nom"),
    ("change_case", "Changer la casse"),
    ("slugify", "Nom sans accents ni espaces"),
    ("add_prefix", "Ajouter un préfixe"),
    ("add_suffix", "Ajouter un suffixe"),
    ("add_date", "Ajouter une date au nom"),
    ("add_counter", "Ajouter un compteur"),
    ("remove_spaces", "Retirer les espaces"),
    ("normalize_extension", "Normaliser l'extension"),
    ("trash", "Mettre à la corbeille"),
    ("trash_duplicates", "Mettre les doublons à la corbeille"),
    ("empty_trash_older_than", "Vider la corbeille des éléments plus vieux que"),
    ("remove_empty_folders", "Supprimer les dossiers vides"),
    ("zip", "Compresser en zip"),
    ("unzip", "Extraire l'archive"),
    ("zip_folder_then_trash", "Compresser puis mettre l'original à la corbeille"),
    ("create_folder", "Créer un dossier"),
    ("flatten_folder", "Aplatir le dossier"),
    ("set_read_only", "Lecture seule"),
    ("set_executable", "Exécutable"),
    ("set_permissions", "Fixer les droits"),
    ("set_modified_time", "Fixer la date de modification"),
    ("pin_folder_in_sidebar", "Épingler dans la barre latérale"),
    ("add_to_favorites", "Ajouter aux favoris"),
    ("remove_from_favorites", "Retirer des favoris"),
    ("hide", "Masquer ou afficher"),
    ("open_with_default", "Ouvrir avec l'application par défaut"),
    ("reveal_in_five_folder", "Montrer dans FIVE"),
    ("notify", "Notifier"),
    ("notify_only_once", "Notifier une seule fois"),
    ("stop_processing", "Arrêter le traitement"),
    ("skip_remaining_actions", "Ignorer les actions suivantes"),
    ("wait_stable", "Attendre que le fichier soit stable"),
    ("log_only", "Seulement journaliser"),
];

/// Libellé trouvé dans une table, ou le nom technique si inconnu.
fn lookup(table: &[(&str, &str)], kind: &str) -> String {
    table.iter().find(|(name, _)| *name == kind).map_or_else(|| kind.to_string(), |(_, text)| text.to_string())
}

/// Libellé d'un déclencheur : « un fichier apparaît ».
pub fn trigger_label(kind: &str) -> String {
    lookup(&TRIGGERS, kind)
}

/// Libellé d'une condition : « L'extension est parmi ».
pub fn condition_label(kind: &str) -> String {
    lookup(&CONDITIONS, kind)
}

/// Libellé d'une action : « Déplacer vers ».
pub fn action_label(kind: &str) -> String {
    lookup(&ACTIONS, kind)
}

/// Libellé du statut d'une règle.
pub fn status_label(status: &str) -> &'static str {
    match status {
        "active" => "Active",
        "trial" => "En essai",
        "disabled" => "Désactivée",
        _ => "Invalide",
    }
}

/// Libellé de l'issue d'une action dans l'historique.
pub fn outcome_label(outcome: &str) -> &'static str {
    match outcome {
        "done" => "Fait",
        "dry_run" => "Essai",
        "skipped" => "Ignoré",
        "failed" => "Échec",
        "deferred" => "Interface",
        _ => "À décider",
    }
}
