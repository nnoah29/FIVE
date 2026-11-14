# FIVE — Règles « condition → action »

Spécification du système de règles automatiques de FIVE : ce que FIVE fait seul, sans clic, sur les fichiers d'un ou plusieurs dossiers. Même modèle que les règles de MIRA (`docs/dev/specs/R1-regles.md` dans le dépôt MIRA) : alternatives (OU) de suites de conditions (ET), conditions inversables, actions ordonnées, évaluateur pur testable sans écran.
Référence : `API.md` (contrat), `docs/DECISIONS.md` (D-02 dépendances minimales, D-05 zip, D-06 barre latérale). Ce document est la **spécification** ; le lot 1 est implémenté (voir §12).

## 1. Objectif et décisions proposées

Une règle surveille des dossiers, repère les fichiers qui correspondent à ses conditions et leur applique des actions. L'utilisateur la crée et la modifie depuis l'interface ; toute la logique vit dans `five-engine`.

| Décision | Choix |
|---|---|
| Quand | À l'arrivée ou à la modification d'un fichier dans un dossier surveillé, à intervalle fixe, ou à la demande (« Appliquer maintenant » sur une sélection ou un dossier). |
| Combinaison | « OU de groupes ET » : une règle a une ou plusieurs alternatives (OU) ; chaque alternative est une suite de conditions reliées par ET ; chaque condition est inversable. Pas de parenthèses en v1. |
| Portée | Par règle : un ou plusieurs dossiers surveillés (récursif ou non), sous la racine exposée uniquement. |
| Sûreté | Aucune suppression définitive automatique ; jamais d'écrasement ; aperçu obligatoire avant d'activer ; journal et annulation de la dernière exécution. |
| Architecture | Évaluateur pur dans le moteur (`rules/`), veilleur de dossiers séparé, actions exécutées par la file de transferts existante (progression, pause, annulation). |
| Stockage | `~/.config/five/regles.json`, versionné (`schema_version`), jamais de secret. |
| Dépendances | Aucune nouvelle : veille par comparaison périodique des dates et tailles (D-02) ; `inotify` via `rustix` en option, derrière le même trait. |

Hors périmètre v1 : exécuter une commande ou un script (surface d'attaque, voir `API.md`), conversion de formats, OCR, condition sur le contenu binaire, groupes imbriqués, règles entre machines, synchronisation cloud, étiquettes (tags) persistantes, suppression définitive automatique, envoi de fichiers par réseau.

## 2. Types (`five-engine`)

JSON étiqueté (`type`, snake_case).

- `Rule` : `id`, `name`, `enabled`, `watch: Vec<Watch>`, `triggers: Vec<Trigger>`, `alternatives: Vec<ConditionGroup>`, `actions: Vec<RuleAction>`, `on_conflict`, `schema_version`. L'ordre des règles n'est pas dans `Rule` (porté par le fichier).
- `Watch` : `path` (relatif à la racine), `recursive`, `max_depth`, `ignore_hidden`.
- `ConditionGroup` : `conditions: Vec<Condition>` (ET).
- `Condition` : `negated: bool`, `test: ConditionTest`.
- `TextMatcher` : opérateur (`contains`, `equals`, `starts_with`, `ends_with`, `glob`, `regex`, `in_list`) et valeur ou liste de valeurs ; casse ignorée par défaut, option `case_sensitive`.
- `Compare` : `lt`, `le`, `eq`, `ne`, `ge`, `gt`, `between`.
- `Duration` : nombre et unité (`seconds`, `minutes`, `hours`, `days`, `weeks`, `months`).

### 2.1 Déclencheurs (`Trigger`)

| Variante | Sens |
|---|---|
| `Created` | un nouveau fichier ou dossier apparaît dans un dossier surveillé |
| `Modified` | un fichier existant change de taille ou de date |
| `MovedIn` | un élément arrive d'ailleurs (déplacé ou copié) |
| `Stable { for }` | la taille n'a pas changé depuis `for` (fin d'un téléchargement ou d'une copie) ; évite d'agir sur un fichier en cours d'écriture |
| `Schedule { every }` | toutes les N minutes, heures ou jours, sur l'ensemble du dossier |
| `Daily { at }` / `Weekly { day, at }` | à heure fixe |
| `OnStart` | au démarrage de FIVE |
| `OnMount { disk }` | quand un disque ou un partage est monté |
| `Manual` | « Appliquer maintenant » uniquement |

Un fichier temporaire (`.part`, `.crdownload`, `.tmp`, `~`, `.swp`, verrous) n'est jamais traité tant qu'il porte ce nom ; la règle se déclenche quand il est renommé.

### 2.2 Catalogue de conditions (`ConditionTest`)

| Famille | Variantes |
|---|---|
| Nom | `Name { matcher }`, `Stem { matcher }` (nom sans extension), `Extension { in_list }`, `NameLength { cmp, n }`, `NameHasSpaces`, `NameHasUppercase`, `NameHasAccents`, `NameHasDateLike` (AAAA-MM-JJ, JJ/MM/AAAA, AAAAMMJJ) |
| Chemin | `Path { matcher }`, `ParentName { matcher }`, `InFolder { path, recursive }`, `Depth { cmp, n }`, `InsideGitRepo`, `InsideTrash` |
| Nature | `IsFile`, `IsFolder`, `IsSymlink`, `IsBrokenSymlink`, `IsHidden`, `IsExecutable`, `IsReadOnly`, `IsEmpty` (fichier de 0 octet, dossier vide), `IsMountPoint` |
| Famille de fichier | `Kind { in_list: image\|video\|audio\|document\|archive\|code\|other }`, `Mime { matcher }` (déterminé par l'extension, repli sur les premiers octets) |
| Taille | `Size { cmp, bytes }`, `FolderTotalSize { cmp, bytes }`, `ChildCount { cmp, n }` (dossiers), `FileCount { cmp, n }` |
| Dates | `ModifiedBefore`, `ModifiedAfter`, `ModifiedWithin { duration }`, `OlderThan { duration }`, `CreatedBefore`, `CreatedAfter`, `AccessedOlderThan { duration }`, `ModifiedOnWeekday { days }`, `ModifiedInMonth { months }`, `ModifiedInYear { years }`, `NeverOpened` (date d'accès = date de création) |
| Contenu texte | `TextContains { matcher }` (200 Ko au plus, fichiers texte), `LineCount { cmp, n }`, `FirstLine { matcher }`, `HasShebang`, `Encoding { utf8\|other }` |
| Images | `ImageWidth { cmp, px }`, `ImageHeight { cmp, px }`, `Orientation { landscape\|portrait\|square }`, `Megapixels { cmp, n }`, `HasAlpha` |
| Audio et vidéo | `Duration { cmp, seconds }`, `VideoResolution { cmp }`, `HasAudioTrack` |
| Archives | `ArchiveIsReadable`, `ArchiveEntryCount { cmp, n }`, `ArchiveContainsName { matcher }`, `ArchiveIsEncrypted` |
| Doublons | `DuplicateOfAnother { scope: folder\|watched\|root }` (taille puis SHA-256), `IsNewestOfDuplicates`, `IsOldestOfDuplicates`, `SameNameAs { scope }` |
| Git | `GitStatus { in_list: modified\|added\|deleted\|untracked\|ignored\|conflict }` |
| Origine | `DownloadedFrom { matcher }` (attribut étendu `user.xdg.origin.url` s'il existe), `FromDisk { disk }` |
| Système | `FreeSpaceBelow { bytes, on: folder\|disk }`, `DiskIsMounted { disk }`, `OwnedByMe`, `Permissions { mode }` |
| Interface | `IsFavorite`, `InRecents`, `IsPinnedFolder` |
| Composition | `AllOf`, `AnyOf`, `NoneOf` (un niveau, pour factoriser) |

Pas de condition sur le contenu binaire, sur des métadonnées EXIF ou ID3 en v1 (à étudier, voir §11).

### 2.3 Catalogue d'actions (`RuleAction`)

| Famille | Variantes | Traduction |
|---|---|---|
| Déplacer et copier | `MoveTo { folder }`, `CopyTo { folder }`, `MoveToDated { folder, pattern }` (sous-dossiers `{année}/{mois}`) , `MoveToByKind { base }` (Images, Vidéos, Audio, Documents, Archives, Code), `MoveToByExtension { base }` | `jobs::Kind::Move` / `Copy` |
| Renommer | `Rename { template }`, `ReplaceInName { find, replace, regex }`, `ChangeCase { lower\|upper\|title\|sentence }`, `Slugify` (sans accents ni espaces), `AddPrefix`, `AddSuffix`, `AddDate { from: modified\|created\|today, position, format }`, `AddCounter { start, width }`, `RemoveSpaces`, `NormalizeExtension` (`.JPG` → `.jpg`, `.jpeg` → `.jpg`) | `rename` (lot atomique, jamais d'écrasement) |
| Nettoyage | `Trash`, `TrashDuplicates { keep: newest\|oldest\|shortest_name }`, `EmptyTrashOlderThan { duration }`, `RemoveEmptyFolders` | corbeille XDG ; jamais de suppression définitive automatique |
| Archives | `Zip { mode: one_per_item\|one_for_all, name_template }`, `Unzip { then: keep\|trash_archive }`, `ZipFolderThenTrash` | D-05 (`Kind::Zip`, `Kind::Unzip`) |
| Dossiers | `CreateFolder { template }`, `FlattenFolder` (remonte le contenu d'un dossier à un seul enfant) | `mkdir` |
| Attributs | `SetReadOnly { value }`, `SetExecutable { value }`, `SetPermissions { mode }`, `SetModifiedTime { from: created\|today\|name }` | `chmod`, `utimes` |
| Organisation | `PinFolderInSidebar` (D-06), `AddToFavorites`, `RemoveFromFavorites`, `Hide { value }` (préfixe point) | réglages et signets GTK |
| Ouverture | `OpenWithDefault`, `RevealInFiveFolder` (ouvre le dossier dans FIVE) | `open_item` |
| Notification | `Notify { message_template }` (toast et entrée du journal), `NotifyOnlyOnce` | propre aux règles |
| Enchaînement | `StopProcessing`, `SkipRemainingActions`, `WaitStable { for }` | propre aux règles |
| Journal | `LogOnly` (ne fait rien, enregistre ce qui aurait été fait ; sert à tester une règle) | propre aux règles |

Modèles de noms et de chemins (`template`) : `{name}`, `{stem}`, `{ext}`, `{kind}`, `{size}`, `{parent}`, `{counter}`, `{counter:3}`, `{modified:AAAA-MM-JJ}`, `{created:AAAA}`, `{today:AAAA-MM-JJ}`, `{year}`, `{month}`, `{day}`, `{weekday}`, `{hash:8}` (début du SHA-256), `{width}`, `{height}`, `{duration}`. Un modèle inconnu est une erreur de validation, jamais du texte littéral.

### 2.4 Politique de conflit (`on_conflict`)

`rename` (défaut : `nom (2).ext`), `skip` (ne rien faire et le journaliser), `ask` (arrêter la règle et demander). `replace` n'existe pas : FIVE n'écrase jamais.

## 3. Stockage

- `regles.json` : tableau de règles dans l'ordre d'évaluation, `schema_version` global et par règle. Écriture atomique (fichier temporaire puis renommage). Un fichier abîmé est mis de côté (`regles.json.abime`) et les règles sont désactivées, jamais perdues.
- `journal-regles.jsonl` : une ligne par action (date, règle, chemin source, chemin résultat, action, issue). Rotation à 5 Mo. Sert à l'annulation et à l'affichage « Historique » ; aucun contenu de fichier n'y figure.
- `etat-regles.json` : dernier instantané connu des dossiers surveillés (nom, taille, date), pour ne pas retraiter les fichiers existants au démarrage.
- Les règles sont de la configuration : migrées sans perte. L'état et le journal sont transitoires.

## 4. Évaluation (`five-engine/src/rules`)

Fonction pure, sans I/O.

- Entrée : règles ordonnées et `FileContext` = `Entry` (nom, taille, dates, dossier, lien) + informations demandées à la demande (type MIME, dimensions, hachage, statut Git, doublons) + heure courante. Les informations coûteuses ne sont calculées que si une condition les utilise, et une seule fois par fichier.
- Sortie : `RuleOutcome { actions: Vec<PlannedAction>, matched: Vec<RuleId>, skipped: Vec<(RuleId, Reason)> }`.
- Parcours : règles activées, valides, dans l'ordre, dont un dossier surveillé contient le fichier. Une règle se déclenche si au moins une alternative a toutes ses conditions vraies (inversions appliquées). Ses actions s'ajoutent dans l'ordre. `StopProcessing` interrompt le parcours pour ce fichier.
- Un fichier déplacé ou renommé par une règle est réévalué au plus **une fois** par passe (anti-boucle) ; une règle ne peut pas traiter le résultat de sa propre action.
- Conflits entre règles : la première règle qui déplace ou supprime le fichier l'emporte ; les suivantes voient le nouveau chemin. Deux renommages contradictoires dans la même passe : le premier gagne et le second est journalisé comme ignoré.
- Expressions régulières : crate `regex` (temps linéaire), compilées et plafonnées en taille à l'enregistrement. Globs : `*`, `?`, `[abc]`, `**`.
- Ordre d'application d'une passe : tri par chemin, déplacements et renommages d'abord, archives ensuite, suppressions en dernier.

## 5. Validation (à l'enregistrement, erreur explicite en français)

- Au moins un dossier surveillé, un déclencheur, une alternative non vide et une action.
- Dossier surveillé ou destination hors de la racine, inexistant, ou dans la corbeille (sauf `EmptyTrashOlderThan`).
- Une destination ne peut pas être dans un dossier surveillé récursivement par la même règle si l'action peut la re-déclencher (boucle) : refus avec la raison, sauf si une condition l'exclut explicitement (`InFolder` inversé).
- Expression régulière ou glob invalide ; modèle de nom inconnu ; plafonds dépassés (règles : 200, alternatives : 10, conditions : 30, actions : 20, valeurs : 1 000 caractères).
- Actions contradictoires dans une même règle (`Trash` suivi de `MoveTo`, `Zip` puis `Unzip` du même fichier, deux `MoveTo`).
- Action `Trash` ou `TrashDuplicates` sans au moins une condition (une règle « tout mettre à la corbeille » est refusée).
- Intervalle de `Schedule` inférieur à 1 minute ; durée `Stable` inférieure à 2 secondes.
- Si un dossier visé disparaît ensuite, la règle est **invalide** (statut calculé à la lecture) : non exécutée, signalée dans l'interface avec la raison.
- Activation : une règle nouvelle ou modifiée reste **en essai** (équivalent `LogOnly`) tant que l'utilisateur n'a pas consulté son aperçu et choisi « Activer ».

## 6. Intégration

- **Veilleur** (`rules/watch`) : un thread par FIVE, analyse les dossiers surveillés toutes les 5 s (configurable) en comparant `(nom, taille, date)` à `etat-regles.json` ; budget de 200 ms par passe, reprise à la passe suivante au-delà ; ne descend pas dans `.git`, `node_modules`, `target`, ni dans les dossiers cachés sauf demande.
- **Exécution** : les actions de déplacement, copie, zip et extraction passent par `jobs::start::start_job` (progression, pause, annulation visibles dans la file de transferts) ; les renommages par `rename`, la corbeille par `trash_ops`. Aucune nouvelle voie d'écriture : les règles de sécurité de `API.md` (confinement à la racine, liens non suivis, corbeille XDG, jamais d'écrasement) s'appliquent telles quelles.
- **Fichiers en cours d'écriture** : jamais traités avant `Stable` ; un fichier ouvert en écriture par un autre processus (verrou `fcntl` ou taille changeante) est repoussé à la passe suivante.
- **Reprise sur incident** : le journal indique l'action commencée ; au redémarrage, une action interrompue est rejouée seulement si elle est idempotente (renommage, déplacement), sinon signalée.
- **Limites de débit** : 100 fichiers par passe et par règle, 1 000 par jour pour `Notify` ; au-delà, la règle se met en pause et le dit.
- **Arrêt** : fermer FIVE arrête le veilleur ; les règles `Schedule`, `Daily`, `Weekly` ne rattrapent pas une exécution manquée (une seule exécution au prochain démarrage si l'option « rattraper » est cochée).
- **Disques** : un disque absent suspend les règles qui le visent, sans erreur.

## 7. API (moteur, puis serveur)

Moteur : `rules::{list, get, create, update, delete, set_enabled, reorder, validate, preview, run_now, undo_last, history, catalog}`.
Serveur (à ajouter à `API.md`) :

| Route | Rôle |
|---|---|
| `GET /api/rules` | liste des règles avec statut (valide, en essai, active, invalide + raison) |
| `GET /api/rules/catalog` | catalogue de conditions, d'actions, de modèles et de déclencheurs (le moteur reste la source de vérité) |
| `GET /api/rules/history?limit` | journal récent |
| `POST /api/rules` | créer ; `PUT /api/rules/<id>` modifier ; `DELETE /api/rules/<id>` |
| `POST /api/rules/validate` | valider sans enregistrer |
| `POST /api/rules/preview` | liste des fichiers touchés et de ce qui leur arriverait, sans rien modifier |
| `POST /api/rules/run` | `{id, paths?}` appliquer maintenant |
| `POST /api/rules/undo` | annuler la dernière exécution (déplacements, renommages, archives créées, éléments mis à la corbeille) |
| `POST /api/rules/enable` | `{id, enabled}` ; `POST /api/rules/reorder` `{ids[]}` |

Les écritures portent l'en-tête `X-Five: 1` comme les autres. Aucun chemin absolu ni secret dans les erreurs.

## 8. Interface native (puis web)

- **Paramètres → Règles** : liste réordonnable (glisser ou monter/descendre), interrupteur d'activation, badge « en essai », « active », « invalide : <raison> », compteur de fichiers traités, dernière exécution.
- **Éditeur** : nom ; dossiers surveillés (sélecteur) ; déclencheurs ; blocs de conditions séparés par « OU », « ET » dans un bloc, case « inverser » par condition ; liste d'actions ordonnée ; politique de conflit ; bouton **Aperçu** (tableau « fichier → ce qui lui arrive »), **Essayer**, **Activer**.
- **Historique** : tableau (heure, règle, fichier, action, résultat), bouton **Annuler la dernière exécution**.
- **Depuis un fichier** : menu « Créer une règle à partir de ce fichier » (préremplit nom, extension, dossier, taille) ; depuis un dossier : « Surveiller ce dossier ».
- **Palette** : « Appliquer les règles à ce dossier », « Ouvrir l'historique des règles », « Mettre en pause toutes les règles ».
- **Indicateur** : pastille dans la barre du haut quand une règle est en cours ou en erreur ; interrupteur global « Règles actives ».
- **Libellés** : en français dans `native/src/model/` (une table unique conditions / actions / modèles), jamais dans le moteur.

## 9. Recettes (exemples attendus du catalogue)

1. Téléchargements : fichier stable, extension image → `Images/{année}/{mois}`.
2. Captures d'écran (`Screenshot*.png`) de plus de 30 jours → corbeille.
3. Archives `.zip` stables dans Téléchargements → `Unzip` puis archive à la corbeille.
4. Documents PDF dont le nom contient « facture » → `Documents/Factures/{année}` + préfixe `{modified:AAAA-MM-JJ}`.
5. Doublons exacts dans Photos → `TrashDuplicates` en gardant le plus ancien.
6. Dossier de plus de 90 jours, sans accès depuis 90 jours → `Zip` puis `Trash`.
7. Noms avec espaces ou accents dans un dossier de projet → `Slugify`.
8. Fichiers `.tmp`, `.part` de plus de 7 jours → corbeille.
9. Vidéos de plus de 2 Go dans Téléchargements → `Notify` (rien d'autre).
10. Disque de sauvegarde monté → `CopyTo` des documents modifiés ces 24 h.
11. Corbeille de plus de 30 jours → `EmptyTrashOlderThan` (la seule suppression définitive permise, via le mécanisme de corbeille XDG).

## 10. Tests (boîte noire, depuis ce document)

- Évaluateur : chaque condition et chaque action, ET, OU, inversion, portée par dossier, `StopProcessing`, anti-boucle, conflits entre règles, modèles de noms (chaque variable), globs et regex.
- Validation : chaque cas de refus du §5, règle devenue invalide, boucle destination/surveillance, règle « tout supprimer ».
- Veilleur : un fichier créé déclenche la règle ; un fichier en cours d'écriture ne la déclenche pas avant `Stable` ; les fichiers existants au démarrage ne sont pas retraités ; plafond de débit ; dossier surveillé supprimé.
- Sécurité : aucune action ne sort de la racine ; liens symboliques non suivis ; jamais d'écrasement ; aucune suppression définitive hors corbeille ; chemins absolus absents des erreurs.
- Reprise : arrêt au milieu d'une passe, rejeu sans doublon ; journal et annulation (chaque action est réversible ou explicitement marquée non réversible).
- Contrat : tests des routes du §7, y compris les refus (400, 403, 404, 409).
- Interface : tests de modèle (libellés, catalogue), de l'éditeur et de la liste ; captures `--shot` de la liste, de l'éditeur et de l'aperçu.

## 11. Livraison en lots et questions ouvertes

Lots : 1. types, stockage, validation, évaluateur (testables sans écran) ; 2. veilleur, exécution par la file de transferts, journal, annulation ; 3. API moteur et serveur ; 4. interface native (liste, éditeur, aperçu, historique) ; 5. interface web (optionnel).

Questions ouvertes (à trancher avant le lot 1) :
- Veille par `inotify` ou par balayage périodique (balayage retenu par défaut, D-02).
- Conditions EXIF et ID3 (dépendance supplémentaire).
- Annulation illimitée ou limitée à la dernière exécution (dernière exécution retenue).
- Règles par dossier ou règles globales ordonnées (règles globales retenues, comme MIRA).
- Exécution en arrière-plan quand l'interface est fermée (non retenue : FIVE n'est pas un service).

## 12. État d'avancement

**Lot 1 — fait** (`engine/src/rules/`, tests `engine/tests/rules_*.rs`) : types (`Rule`, déclencheurs, 74 conditions, 40 actions), évaluateur pur (`evaluate`, `rule_matches`), planification des actions avec noms calculés (`plan_rule`, modèles de noms), validation complète du §5 (`validate_rule`, `validate_set`), statut (`rule_status`), stockage atomique (`store`), noms temporaires, glob, regex en cache.

**Lot 2 — fait** : veilleur par balayage (`scan`, `changes`, `state`), règles planifiées (`schedule`), contexte construit depuis le disque avec seulement les informations demandées (`needs`, `context_build`, `info_*`), exécution (`exec_*`, `exec_pass`), journal et annulation (`journal`, `undo`), service qui orchestre une passe (`runner::RulesService::{tick, run_now, preview}`).
**Lot 3 — fait** : API du moteur (`rules::api_*`) et routes `/api/rules` du serveur (voir `API.md`), catalogue exposé, aperçu obligatoire avant activation (empreinte de la règle dans `apercus-regles.json`).
**Lot 4 — fait** (`native/src/rules/`, `native/src/settings/rules_*.rs`) : veille native, onglet Paramètres → Règles, éditeur complet, aperçu, historique, annulation.
Reste : lot 5 (interface web, optionnel).

Écarts du lot 1 avec le texte ci-dessus (à relire avant le lot 2) :
- `Compare::between` n'existe pas : deux conditions (`ge` et `le`) suffisent.
- Après une action qui déplace ou met à la corbeille, **l'évaluation de ce fichier s'arrête** (le §4 prévoyait que les règles suivantes voient le nouveau chemin) ; c'est plus sûr tant que l'exécuteur n'existe pas.
- Dans une règle, les renommages s'enchaînent sur le nom courant ; entre règles, le premier renommage gagne et les suivants sont journalisés comme ignorés.
- `AddDate` : `AAAA-MM-JJ Nom.ext` (préfixe) ou `Nom AAAA-MM-JJ.ext` (suffixe) ; `AddCounter` : `Nom_NNN.ext`.
- `evaluate` suppose que l'appelant a déjà retiré les règles invalides (`rule_status`) ; un état d'information absent (dimensions, hachage, texte…) rend la condition fausse, jamais une erreur. L'appelant remplit `FileContext` seulement pour les conditions utilisées.
- Une règle neuve est **en essai** par défaut (`trial: true`) ; `dry_run` marque les actions prévues d'une règle en essai ou contenant `log_only`.
- La table des familles de fichiers (`rules/kinds.rs`) double celle de l'interface native ; à fusionner au lot 4.

Écarts du lot 2 (à relire avant le lot 3) :
- **Première passe = mémorisation.** Les fichiers déjà présents ne sont jamais traités ; seuls les fichiers apparus ou modifiés ensuite le sont (`Created`, `Modified`, `Stable`). Le veilleur ne distingue pas `MovedIn` de `Created`.
- **Règles planifiées** (`Schedule`, `Daily`, `Weekly`) : une règle vue pour la première fois est seulement mémorisée, aucun rattrapage d'une exécution manquée. `OnStart` ne s'exécute qu'à la première passe du service ; `OnMount` à l'apparition d'un disque dans `mounted_disks`.
- **Actions confiées à l'interface** (résultat `deferred`, rien n'est exécuté par le moteur) : `notify`, `notify_only_once`, `add_to_favorites`, `remove_from_favorites`, `pin_folder_in_sidebar`, `open_with_default`, `reveal_in_five_folder`. Plafonds appliqués par le moteur : 1 000 notifications par jour, une `notify_only_once` par règle et par passe.
- **Conditions non calculées** (toujours fausses) : `duration`, `video_resolution`, `has_audio_track` (il faudrait un outil externe) ; `archive_is_encrypted` ; `downloaded_from` dépend de l'attribut étendu `user.xdg.origin.url`. Les favoris, récents et dossiers épinglés viennent de `Extras`, rempli par l'interface.
- **Annulation** : réversibles — renommer, masquer, déplacer, copier, compresser, extraire, mettre à la corbeille, créer ou supprimer un dossier vide, droits, date. **Non réversibles** : `empty_trash_older_than` (effacement définitif) et `flatten_folder`. L'annulation refuse de toucher à ce qui a changé depuis et à tout chemin hors de la racine et de la corbeille, même si le journal a été modifié.
- **Plafonds** : 100 fichiers par règle et par passe (le reste est journalisé « plafond atteint »), balayage limité à 200 ms par passe (reprise à la passe suivante), doublons calculés en 3 s au plus par fichier.
- Le journal contient aussi les lignes « essai » (`dry_run`) ; une règle neuve reste en essai tant qu'elle n'est pas activée.
- `Zip { one_for_all }` regroupe les fichiers d'une même passe et d'une même règle dans une seule archive, créée à côté du premier fichier.

Écarts du lot 3 :
- `POST /api/rules/<action>` remplace les routes `/api/rules/enable` etc. du §7 par un seul aiguillage ; les chemins sont identiques. Pas de `GET /api/rules/<id>` : la liste suffit.
- Une règle créée ou modifiée repasse **toujours en essai** (même si le client envoie `trial: false`) ; seule `enable` avec `activate: true` la sort de l'essai, après un aperçu de sa version actuelle.
- Le serveur n'exécute aucune passe automatique : `tick` reste à l'interface native (lot 4). Les routes `preview` et `run` créent un service éphémère sur les transferts partagés du serveur.

Écarts du lot 4 :
- **Veille** : un passage toutes les 5 secondes, dans un fil de travail, tant que la fenêtre native est ouverte. Interrupteur général dans l'onglet Règles ; commandes de palette « Mettre en pause ou reprendre les règles » et « Ouvrir les règles et leur historique ». **Pas de pastille dans la barre du haut** (prévue au §8) : un échec ou une question s'affiche en notification.
- **Éditeur** : tous les champs sont déduits du catalogue du moteur (`fields`), y compris les conditions imbriquées (toutes / au moins une / aucune). Une taille se saisit « 5 Mo », des droits en octal, une date « AAAA-MM-JJ hh:mm ». « Enregistrer » refuse une règle invalide en listant tous les problèmes ; « Aperçu » fonctionne sur le brouillon non enregistré.
- **Actions de l'interface** : notification (message en notification), favoris (fichier de signets GTK), épinglage, ouverture, « montrer dans FIVE ». La condition « dans les récents » reste toujours fausse (l'interface ne fournit pas les récents au moteur).
- Menu d'un fichier : « Créer une règle à partir de ce fichier » (dossier du fichier, même extension) ; menu d'un dossier : « Surveiller ce dossier avec une règle ».
- Non fait : tests d'interface automatisés de l'éditeur (vérifié par captures sous Xvfb et par un clic réel sur « Enregistrer »), raccourcis clavier propres aux règles.
