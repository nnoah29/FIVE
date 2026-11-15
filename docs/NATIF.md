# FIVE natif (GTK 4)

Lancer : `cargo run -p five-gtk --release -- [--root DOSSIER]` (racine par défaut : le dossier personnel).
Réglages : `~/.config/five/reglages.json` ; dernier emplacement : `~/.config/five/session.json`.

## Fonctions
Barre latérale, barre du haut, cartes de dossiers, liste/grille, panneau Espace et aperçu, raccourcis (voir Paramètres, onglet Raccourcis) : multi-sélection, renommage simple et par lot
(remplacement, expression régulière, modèle), copier/couper/coller avec file de transferts (pause, reprise, annulation),
palette de commandes (Ctrl P / Ctrl K), aperçu rapide (Espace), analyse d'occupation, doublons, indicateurs Git,
contenu des archives, disques montés, récents, favoris, réseau, corbeille, thème clair/sombre, couleur principale,
densité, session restaurée.

## Captures pilotées par script
`five-gtk --root /home/nnoah --shot sortie.png=go:_WORKS/DeV/FIVE,select:1 --shot autre.png=settings,tab:keys`
Commandes : `go:CHEMIN` `view:list|grid` `theme:light|dark|system` `accent:#rrggbb` `compact:1` `select:N` `range:A-B`
`settings` `tab:display|files|keys|system` `palette` `quicklook` `usage` `dups` `rename` `mkdir` `delete` `zip` `unzip`
`input:TEXTE` `confirm` `copy` `cut` `paste` `filter:ID` `search:TEXTE` `size:LxH` `side`.
Une étape toutes les 900 ms ; les animations CSS sont coupées pendant les captures.

## Normes
`python3 tools/check-norms` s'applique aussi à `native/` (fonction ≤ 30 lignes, ≤ 6 fonctions par fichier, une ligne `///`).
