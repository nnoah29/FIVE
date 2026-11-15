# Décisions de FIVE

## D-01 — Moteur Rust et interface native GTK (2026-11-14)
- **Contexte** : FIVE est une application de bureau légère, en Rust, pour Linux.
- **Choix** : `five-engine` (logique pure sur les fichiers, aucun réseau) et `five-gtk` (interface native GTK 4, qui appelle le moteur directement).
- **Raison** : le moteur reste réutilisable par un autre client (FFI), comme pour MIRA, et l'application n'ouvre aucun port.

## D-02 — Dépendances minimales
`serde_json`, `sha2`, `rustix` (statvfs, attributs étendus), `time` (heure locale du `.trashinfo`, décalage lu avant les threads), `flate2` (tar.gz, zip). Pas de runtime asynchrone. Zip et tar lus et écrits à la main. Ajouts documentés plus bas : `serde` et `regex` (D-07).

## D-03 — Comportements assumés du moteur
- Archives : zip (sans zip64), tar et tar.gz ; pas de bz2/xz.
- Liens symboliques vers un dossier : comptés comme fichiers dans l'occupation et les doublons.

## D-04 — Interface native GTK 4 (2026-11-14)
- **Contexte** : l'objectif est une vraie application légère, fidèle à la direction artistique de MIRA.
- **Choix** : crate `five-gtk` (gtk4-rs 0.11), appel direct de `five-engine` (travail bloquant dans `gio::spawn_blocking`), feuille de style GTK à jetons de couleur (thème clair/sombre, couleur principale, densité), icônes SVG dessinées avec cairo (recolorables par CSS), polices Anton / Hanken Grotesk / JetBrains Mono embarquées (`native/assets/fonts`) et déclarées à Pango.
- **Écarts assumés** : l'aperçu PDF utilise `pdftoppm` s'il est installé ; l'audio et la vidéo demandent GStreamer ; le tiroir de la barre latérale apparaît à 760 px ; entre 760 et 900 px le libellé « Paramètres » est masqué et les marges de la barre du haut réduites pour que la fenêtre puisse descendre à 760 px.
- **Vérification visuelle** : `five-gtk --shot` rend la fenêtre en PNG (sous Xvfb : `xvfb-run -a env GDK_BACKEND=x11 GSK_RENDERER=cairo five-gtk --shot …`), sans dépendre d'un outil externe.

## D-05 — Compresser en zip et extraire (2026-11-14)
- **Contexte** : le prototype ne faisait que lister les archives ; la compression et l'extraction sont attendues d'un gestionnaire de fichiers.
- **Choix** : `zip` et `unzip` sont deux natures de transfert de plus (`Kind`), donc progression, pause et annulation sont celles des copies. Écriture et lecture zip à la main (D-02) : deflate par `flate2`, somme de contrôle `flate2::Crc`, en-tête local corrigé après les données (pas de descripteur de données). L'archive créée porte le nom de l'élément seul + `.zip`, sinon `Archive.zip`, dans le dossier du premier élément ; l'extraction crée toujours un dossier neuf (nom de l'archive sans extension, suffixe ` (2)` en fin de nom) et le supprime en cas d'échec. tar et tar.gz s'extraient avec le même lecteur d'en-têtes que le listage.
- **Sécurité** : `safe_join` refuse `..` et l'octet nul ; tailles annoncées bornées pendant la copie (`copy_limited`) ; CRC vérifié ; plafond de 100 Go ; aucun lien ni fichier spécial recréé ; droits `& 0o777 | 0o600`, jamais de setuid.
- **Écarts assumés** : pas de zip64, de chiffrement, de bz2 ni de xz ; les dates des fichiers extraits ne sont pas restaurées ; menu contextuel, panneaux (archive seule : « Extraire ici » ; sinon « Compresser en .zip »), palette de commandes.
- **Retour arrière** : retirer `zip`/`unzip` de `Kind::parse` ; le reste du moteur est inchangé.

## D-06 — Dossiers de la barre latérale configurables (2026-11-14)
- **Contexte** : la section « Dossiers » était figée sur les dossiers XDG (Téléchargements, Images, Musique, Vidéos).
- **Choix** : réglage `side_folders` dans `reglages.json` (liste de `{label, path absolu, icon}`) ; absent = mode **automatique** (comportement d'origine). Onglet Paramètres → **Dossiers** : bascule Automatique / Personnalisés (la liste part des dossiers détectés), nom et chemin modifiables (le chemin invalide est entouré), parcourir, monter / descendre / retirer, « Choisir un dossier… » (sélecteur système) et « Ajouter le dossier ouvert ». Menu d'un dossier : « Épingler dans la barre latérale » (bascule en mode personnalisé si besoin). La palette de commandes propose les mêmes dossiers.
- **Règles** : seuls les dossiers existants sous la racine apparaissent dans la barre (les autres restent listés dans les paramètres, marqués « introuvable ») ; un nom vide affiche le nom du dossier ; icône : celle du dossier détecté, sinon un dossier.
- **Retour arrière** : choisir « Automatique » (ou supprimer `side_folders`).

## D-07 — Règles automatiques : lot 1 et dépendances (2026-11-14)
- **Contexte** : `docs/REGLES.md` décrit un système de règles ; le lot 1 (types, stockage, validation, évaluateur) vit dans `five-engine`.
- **Choix** : `serde` (avec dérivation) et `regex` ajoutés au moteur. Le catalogue compte plus de 110 variantes étiquetées en JSON : les écrire à la main serait long et fragile ; les deux crates étaient déjà dans le graphe (`serde_json`, interface native), aucun téléchargement nouveau. Évaluateur pur, une fonction par famille de conditions, informations coûteuses fournies par l'appelant.
- **Raison** : refuser sans deviner (une variante inconnue est une erreur de lecture, la règle est conservée telle quelle dans `regles.json`) ; aucune écriture ni exécution dans ce lot, donc rien à annuler.
- **Retour arrière** : retirer `engine/src/rules/` et les deux dépendances ; aucun autre module ne s'en sert.

## D-08 — Règles automatiques : lot 2, exécution et journal (2026-11-14)
- **Contexte** : le lot 1 évalue des règles sans rien modifier ; le lot 2 les applique.
- **Choix** : aucune nouvelle voie d'écriture. Déplacement, copie, compression et extraction passent par `jobs::start::start_job` (progression, annulation), les renommages par `rename_many`, la corbeille par `trash_ops` (qui rend maintenant le chemin dans la corbeille pour la restaurer). Le veilleur balaie les dossiers (pas d'`inotify`, D-02) et compare à `etat-regles.json`. Le journal (`journal-regles.jsonl`) contient, pour chaque action, de quoi la défaire (chemins absolus, revérifiés contre la racine à l'annulation).
- **Sûreté** : jamais d'écrasement (politique `rename`, `skip` ou `ask`), seul effacement définitif = fichier de la corbeille plus ancien que la durée demandée (revérifié à l'exécution), droits spéciaux refusés, destination qui remonte (`..`) refusée, règle en essai = aucune écriture.
- **Raison** : une règle automatique doit pouvoir être arrêtée, relue et défaite ; l'exécution ne refait jamais confiance au plan seul (elle revérifie l'état du disque).
- **Retour arrière** : n'appeler ni `tick` ni `run_now` ; rien d'autre ne dépend de ces modules.

## D-09 — Règles automatiques : lot 3, API du moteur (2026-11-14)
- **Contexte** : l'interface doit lire, écrire, prévisualiser et appliquer les règles sans connaître les détails du moteur.
- **Choix** : l'API vit dans `five-engine` (`rules::api_*`, fonctions pures sur la racine et le dossier de configuration) ; l'interface native les appelle directement. Le catalogue (conditions, actions, déclencheurs, champs) est servi par le moteur.
- **Sûreté** : « essai d'abord » est une règle du moteur, pas de l'interface : une règle n'est active qu'après un aperçu de sa version exacte.

## D-10 — Règles automatiques : lot 4, interface native (2026-11-14)
- **Contexte** : les règles (moteur, API) doivent être utilisables sans écrire de JSON.
- **Choix** : la veille tourne dans le processus natif (`RulesService::tick` toutes les 5 s dans un fil de travail, résultats traités sur le fil de l'interface). L'éditeur ne connaît aucun type de condition ou d'action : il lit le catalogue du moteur (famille, champs, genres, valeurs permises) et construit les contrôles ; seuls les libellés français vivent côté interface (`model/rules_labels`, `rules_fields_labels`). Le brouillon est un JSON modifié par chemin ; une modification de structure reconstruit la fenêtre en gardant la position de défilement.
- **Sûreté** : enregistrer repasse toujours en essai ; activer exige l'aperçu de la version exacte (règle du moteur, pas de l'interface) ; « Appliquer maintenant » demande confirmation.
- **Piège évité** : les boutons de l'éditeur ne référencent l'éditeur que faiblement ; l'application le garde en vie tant que sa fenêtre est ouverte.
- **Retour arrière** : l'interrupteur « Appliquer les règles automatiquement » coupe la veille sans rien supprimer.

## D-11 — Sélecteur de fichiers du système (2026-11-15)
- **Contexte** : ouvrir ou enregistrer un fichier depuis une autre application (navigateur…) doit pouvoir passer par FIVE, dans une fenêtre simple, pas dans la fenêtre complète.
- **Choix** : FIVE devient un backend du portail de bureau (`org.freedesktop.impl.portal.FileChooser`) : `five-gtk --portal` exporte `OpenFile`, `SaveFile`, `SaveFiles` et l'objet `Request.Close` avec GDBus (déjà dans gio, aucune dépendance de plus). La fenêtre (`native/src/chooser/`) est indépendante de l'application : liste des dossiers et fichiers, chemin modifiable, emplacements habituels et dossiers épinglés, filtres, nom du fichier à enregistrer, confirmation avant d'écraser. `five-gtk --chooser open|folder|save` la lance seule (chemins sur la sortie standard) pour l'essayer et la tester.
- **Sûreté** : activation volontaire (`portals.conf`) ; le portail `gtk` reste en repli (`five;gtk`). Le sélecteur n'écrit jamais : il ne renvoie que des chemins.
- **Retour arrière** : retirer la ligne `FileChooser=five` de `~/.config/xdg-desktop-portal/portals.conf`.
