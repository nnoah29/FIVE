# FIVE — File Interface & View Explorer

Gestionnaire de fichiers Linux léger, en Rust, même direction artistique que MIRA. L'interface est native (GTK 4), branchée directement sur le moteur.

## Disposition
- `engine/` : crate `five-engine`, logique pure sur les fichiers, aucun réseau.
- `engine/src/rules/` : règles automatiques condition → action (`docs/REGLES.md`), évaluateur, exécution, journal, API.
- `native/` : crate `five-gtk`, interface native GTK 4, branchée directement sur `five-engine` (aucun réseau). `native/src/model/` = logique pure testée ; `app/` = comportements ; `render/`, `preview/`, `settings/` = régions de la fenêtre ; `ui/` = widgets, icônes cairo, thème (`ui/style.css`).
- `tools/check-norms` : normes de code (fonction ≤ 30 lignes, ≤ 6 fonctions par fichier, une ligne `///` par fonction, aucun commentaire dans un corps, zéro `unwrap`/`expect`/`panic!`/`unsafe` hors tests).

## Commandes
- `cargo run -p five-gtk --release -- [--root DOSSIER]` lance l'interface native ; `five-gtk --shot fichier.png=go:chemin,select:1,settings …` produit des captures pilotées par script (voir `docs/NATIF.md`).
- `cargo test -p five-engine --test <famille>` (tests ciblés), `cargo clippy --all-targets -- -D warnings`, `cargo fmt`, `python3 tools/check-norms`.
- `./ci.sh` regroupe le tout ; ne le lancer que sur accord de l'utilisateur.

## Règles
- Tests en boîte noire dans `engine/tests/`, depuis `docs/` (règles, décisions). Un test rouge se corrige dans le code.
- Un commit par fonctionnalité, message `type(domaine): résumé`, sans trailer d'attribution.
