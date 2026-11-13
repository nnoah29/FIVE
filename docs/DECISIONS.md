# Décisions de FIVE

## D-01 — Moteur Rust + coque HTTP locale qui sert l'interface web (2026-11-14)
- **Contexte** : le prototype Python a validé l'interface ; la vraie application doit être légère, en Rust.
- **Choix** : `five-engine` (logique pure, sans réseau) et `five-server` (binaire `five`, `tiny_http`, interface embarquée dans le binaire). L'interface web reste la maquette de référence.
- **Raison** : on garde l'interface validée et on remplace le backend sans la toucher ; le moteur reste réutilisable par un autre client (natif, FFI) comme pour MIRA.
- **Retour arrière** : `legacy-python/serveur.py` fonctionne toujours avec `web/` (lancer avec `python3 legacy-python/serveur.py`).

## D-02 — Dépendances minimales
`serde_json`, `sha2`, `rustix` (statvfs), `time` (heure locale du `.trashinfo`, décalage lu avant les threads), `flate2` (tar.gz), `tiny_http`. Pas de runtime asynchrone. Zip et tar lus à la main (en-têtes seulement). Binaire release : ~1,5 Mo, ~3 Mo de mémoire au repos.

## D-03 — Écarts assumés avec le prototype Python
- Archives : zip (sans zip64), tar et tar.gz ; pas de bz2/xz.
- Liens symboliques vers un dossier : comptés comme fichiers dans l'occupation et les doublons.
- `/api/info` renvoie `engine` à la place de `python`.
- La chaîne de session, le cookie et les en-têtes de protection sont identiques (voir `API.md`).
