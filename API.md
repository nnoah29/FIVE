# FIVE — contrat d'API du prototype

Contrat de comportement du serveur, implémenté en Rust (`engine/`, `server/`). Le prototype Python d'origine est conservé dans `legacy-python/` comme référence.
Toutes les routes sont locales (`127.0.0.1`), sous une racine exposée (`--root`, défaut : dossier personnel). Les chemins `p` sont **relatifs à la racine**.

## Sécurité (à conserver)
- Écoute sur `127.0.0.1` uniquement ; en-tête `Host` vérifié (anti-DNS-rebinding).
- Session : jeton aléatoire dans l'URL de lancement → cookie `HttpOnly; SameSite=Strict`.
- Écritures (POST) : en-tête `X-Five: 1` obligatoire, `Origin` local, corps JSON ≤ 64 Ko.
- Chemins : `realpath` + confinement à la racine ; pour les écritures, le dernier composant n'est jamais suivi (liens symboliques supprimés/renommés eux-mêmes).
- Fichiers servis avec `Content-Security-Policy: sandbox` (sauf PDF) et `nosniff`.
- Suppression = corbeille XDG (`~/.local/share/Trash`, fichier `.trashinfo`) ; définitive uniquement depuis la corbeille.

## Lecture (GET)
| Route | Rôle |
|---|---|
| `/api/places` | racine, utilisateur, dossiers XDG (`user-dirs.dirs`), corbeille |
| `/api/info` | racine, port, corbeille, `engine` (nom et version du moteur), utilisateur |
| `/api/list?p&h&q&f` | contenu d'un dossier ; `q` = recherche récursive (4 s max), `f=1` = recherche floue (sous-séquence, score) |
| `/api/view?v=recent\|favorites\|network` | vues virtuelles : `recently-used.xbel`, signets GTK, montages réseau (`/proc/mounts`) |
| `/api/disks` | disques montés (`/proc/mounts`, périphériques `/dev/*` hors loop) : nom, point de montage, taille, libre ; `p` = chemin relatif si sous la racine, sinon `null` (non cliquable) |
| `/api/raw?p&dl` | contenu brut, plages `Range` (206), `dl=1` → téléchargement |
| `/api/text?p` | 200 Ko de texte, détection binaire (octet nul) |
| `/api/archive?p` | contenu zip, tar, tar.gz (500 entrées), sans extraction ; bz2/xz et zip64 non pris en charge |
| `/api/usage?p` | taille cumulée des enfants directs, budget 10 s, indicateur `partial` |
| `/api/duplicates?p&min` | groupes de doublons : taille puis SHA-256, budget 20 s |
| `/api/git?p` | `{nom: M/A/D/?/!/U}` via `git status --porcelain=v1 -z` (`core.fsmonitor=false`, `hooksPath=/dev/null`, timeout 4 s) |
| `/api/jobs` | file de transferts |

## Écriture (POST JSON)
| Route | Corps | Comportement |
|---|---|---|
| `/api/mkdir` | `{p, name}` | 201 ; 409 si le nom existe |
| `/api/delete` | `{paths[]}` | corbeille, ou définitif si déjà dans la corbeille ; refuse racine et dossier de corbeille |
| `/api/rename` | `{items:[{p,name}]}` | lot atomique : validation complète, puis deux phases (nom temporaire) pour les permutations ; jamais d'écrasement |
| `/api/job` | `{kind: copy\|move\|zip\|unzip, paths[], dest}` | tâche en arrière-plan ; conflit → « nom (2) » ; déplacement : `rename`, sinon copie puis suppression ; `zip` : une archive `dest/<nom>.zip` (ou `Archive.zip`) pour tous les `paths`, liens symboliques ignorés ; `unzip` : un dossier neuf `dest/<archive sans extension>` par archive (zip, tar, tar.gz), supprimé si l'extraction échoue |
| `/api/job/control` | `{id, action: pause\|resume\|cancel}` | pause entre deux blocs de 1 Mo ; annulation supprime le fichier partiel |
| `/api/logout` | `{}` | efface le cookie |

## Archives : limites et sécurité
- Création : zip standard (deflate, noms en UTF-8, droits Unix conservés) ; refus au-delà de 4 Go par fichier ou de 65 535 entrées (413). Une archive partielle est supprimée en cas d'échec ou d'annulation.
- Extraction : zip (stocké ou deflate), tar, tar.gz ; jamais d'écrasement (dossier `nom (2)`). Un nom qui remonte (`..`) ou contient un octet nul refuse toute l'archive (400) ; les noms absolus sont ramenés sous le dossier de destination ; liens et fichiers spéciaux d'un tar sont ignorés. Taille et somme de contrôle de chaque entrée zip sont vérifiées, la taille extraite est plafonnée à 100 Go. Chiffrement, zip64, bz2 et xz : refusés (400).
- La progression d'une extraction suit les octets lus de l'archive.

## Idées non retenues dans le prototype
Terminal intégré et scripts personnalisés (exécution de commandes depuis une page web : surface d'attaque), coffre-fort chiffré, montage SFTP/cloud, onglets et multi-panneaux, étiquettes, recherches enregistrées, coloration syntaxique, EXIF.
