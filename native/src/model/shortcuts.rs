//! Raccourcis clavier affichés dans les réglages.

/// Touches (séparées par deux espaces) et leur effet.
pub const SHORTCUTS: [(&str, &str); 15] = [
    ("/", "Chercher"),
    ("j  k", "Descendre / monter dans la liste (ou les flèches)"),
    ("Entrée", "Ouvrir l'élément sélectionné"),
    ("Retour arrière  h", "Dossier parent"),
    ("c", "Nouveau dossier"),
    ("Suppr  #", "Mettre à la corbeille"),
    ("Espace", "Aperçu rapide"),
    ("Ctrl P  Ctrl K", "Palette de commandes"),
    ("Ctrl A", "Tout sélectionner"),
    ("Ctrl C  Ctrl X  Ctrl V", "Copier, couper, coller"),
    ("F2", "Renommer (par lot si plusieurs éléments)"),
    ("Ctrl clic  Maj clic", "Sélection multiple"),
    (".", "Afficher ou masquer les fichiers cachés"),
    (",", "Ouvrir ou fermer les paramètres"),
    ("Échap", "Fermer l'élément au premier plan"),
];
