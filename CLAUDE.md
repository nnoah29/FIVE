# FIVE — File Interface & View Explorer

Gestionnaire de fichiers Linux léger, en Rust, même direction artistique que MIRA. Le contrat de comportement est `API.md` ; l'interface web (`web/`) est la maquette de référence ; `legacy-python/` est le prototype jetable d'origine.

## Disposition
- `engine/` : crate `five-engine`, logique pure sur les fichiers, aucun réseau.
- `server/` : crate `five-server`, binaire `five`, coque HTTP locale qui sert `web/` et expose l'API.
- `web/` : interface (HTML/CSS/JS sans dépendance).
- `tools/check-norms` : normes de code (fonction ≤ 30 lignes, ≤ 6 fonctions par fichier, une ligne `///` par fonction, aucun commentaire dans un corps, zéro `unwrap`/`expect`/`panic!`/`unsafe` hors tests).

## Commandes
- `cargo test -p five-engine --test <famille>` (tests ciblés), `cargo clippy --all-targets -- -D warnings`, `cargo fmt`, `python3 tools/check-norms`.
- `./ci.sh` regroupe le tout ; ne le lancer que sur accord de l'utilisateur.

## Règles
- Tests en boîte noire dans `engine/tests/`, depuis `API.md`. Un test rouge se corrige dans le code.
- Un commit par fonctionnalité, message `type(domaine): résumé`, sans trailer d'attribution.
