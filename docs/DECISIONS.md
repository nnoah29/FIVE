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

## D-04 — Interface native GTK 4 à parité avec l'interface web (2026-11-14)
- **Contexte** : l'objectif est une vraie application légère ; l'interface web est la maquette validée.
- **Choix** : crate `five-gtk` (gtk4-rs 0.11), appel direct de `five-engine` (travail bloquant dans `gio::spawn_blocking`), feuille de style GTK adaptée de `web/style.css` (mêmes jetons, thème clair/sombre, couleur principale, densité), icônes SVG dessinées avec cairo (recolorables par CSS), polices Anton / Hanken Grotesk / JetBrains Mono embarquées et déclarées à Pango.
- **Écarts assumés** : « Télécharger » devient « Enregistrer une copie… » ; le réglage « ouvrir dans un nouvel onglet » et la session HTTP disparaissent (onglet Système à la place de Serveur) ; l'aperçu PDF utilise `pdftoppm` s'il est installé ; le tiroir de la barre latérale apparaît à 760 px comme sur la web ; entre 760 et 900 px le libellé « Paramètres » est masqué et les marges de la barre du haut réduites pour que la fenêtre puisse descendre à 760 px.
- **Vérification visuelle** : `five-gtk --shot` rend la fenêtre en PNG (sous Xvfb : `xvfb-run -a env GDK_BACKEND=x11 GSK_RENDERER=cairo five-gtk --shot …`), sans dépendre d'un outil externe.
- **Retour arrière** : `server/` + `web/` restent complets et servent la même interface dans un navigateur.

## D-05 — Compresser en zip et extraire (2026-11-14)
- **Contexte** : le prototype ne faisait que lister les archives ; la compression et l'extraction sont attendues d'un gestionnaire de fichiers.
- **Choix** : `zip` et `unzip` sont deux natures de transfert de plus (`Kind`), donc progression, pause et annulation sont celles des copies. Écriture et lecture zip à la main (D-02) : deflate par `flate2`, somme de contrôle `flate2::Crc`, en-tête local corrigé après les données (pas de descripteur de données). L'archive créée porte le nom de l'élément seul + `.zip`, sinon `Archive.zip`, dans le dossier du premier élément ; l'extraction crée toujours un dossier neuf (nom de l'archive sans extension, suffixe ` (2)` en fin de nom) et le supprime en cas d'échec. tar et tar.gz s'extraient avec le même lecteur d'en-têtes que le listage.
- **Sécurité** : `safe_join` refuse `..` et l'octet nul ; tailles annoncées bornées pendant la copie (`copy_limited`) ; CRC vérifié ; plafond de 100 Go ; aucun lien ni fichier spécial recréé ; droits `& 0o777 | 0o600`, jamais de setuid.
- **Écarts assumés** : pas de zip64, de chiffrement, de bz2 ni de xz ; les dates des fichiers extraits ne sont pas restaurées ; l'interface web n'expose pas encore ces actions (l'API les accepte). Native : menu contextuel, panneaux (archive seule : « Extraire ici » ; sinon « Compresser en .zip »), palette de commandes.
- **Retour arrière** : retirer `zip`/`unzip` de `Kind::parse` ; le reste du moteur est inchangé.

## D-06 — Dossiers de la barre latérale configurables (2026-11-14)
- **Contexte** : la section « Dossiers » était figée sur les dossiers XDG (Téléchargements, Images, Musique, Vidéos).
- **Choix** : réglage `side_folders` dans `reglages.json` (liste de `{label, path absolu, icon}`) ; absent = mode **automatique** (comportement d'origine). Onglet Paramètres → **Dossiers** : bascule Automatique / Personnalisés (la liste part des dossiers détectés), nom et chemin modifiables (le chemin invalide est entouré), parcourir, monter / descendre / retirer, « Choisir un dossier… » (sélecteur système) et « Ajouter le dossier ouvert ». Menu d'un dossier : « Épingler dans la barre latérale » (bascule en mode personnalisé si besoin). La palette de commandes propose les mêmes dossiers.
- **Règles** : seuls les dossiers existants sous la racine apparaissent dans la barre (les autres restent listés dans les paramètres, marqués « introuvable ») ; un nom vide affiche le nom du dossier ; icône : celle du dossier détecté, sinon un dossier.
- **Écart** : propre à l'interface native, la web garde la liste automatique.
- **Retour arrière** : choisir « Automatique » (ou supprimer `side_folders`).

## D-07 — Règles automatiques : lot 1 et dépendances (2026-11-14)
- **Contexte** : `docs/REGLES.md` décrit un système de règles ; le lot 1 (types, stockage, validation, évaluateur) vit dans `five-engine`.
- **Choix** : `serde` (avec dérivation) et `regex` ajoutés au moteur. Le catalogue compte plus de 110 variantes étiquetées en JSON : les écrire à la main serait long et fragile ; les deux crates étaient déjà dans le graphe (`serde_json`, interface native), aucun téléchargement nouveau. Évaluateur pur, une fonction par famille de conditions, informations coûteuses fournies par l'appelant.
- **Raison** : refuser sans deviner (une variante inconnue est une erreur de lecture, la règle est conservée telle quelle dans `regles.json`) ; aucune écriture ni exécution dans ce lot, donc rien à annuler.
- **Retour arrière** : retirer `engine/src/rules/` et les deux dépendances ; aucun autre module ne s'en sert.
