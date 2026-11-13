#!/usr/bin/env python3
"""FIVE (File Interface & View Explorer) : serveur local pour parcourir de vrais fichiers dans le navigateur."""

import argparse
import getpass
import http.cookies
import http.server
import json
import mimetypes
import os
import secrets
import datetime
import shutil
import stat
import sys
import time
import urllib.parse
import xml.etree.ElementTree as ET

import ops
from coeur import Config, Refus, clean_name, in_trash, inside_root, relative, resolve, resolve_entry, trash_dir

WEB = os.path.join(os.path.dirname(os.path.abspath(__file__)), "web")
STATIC_TYPES = {".html": "text/html; charset=utf-8", ".css": "text/css; charset=utf-8",
                ".js": "text/javascript; charset=utf-8", ".ttf": "font/ttf", ".svg": "image/svg+xml"}
PLACES = [("desktop", "Bureau", "DESKTOP", "Desktop"), ("downloads", "Téléchargements", "DOWNLOAD", "Downloads"),
          ("docs", "Documents", "DOCUMENTS", "Documents"), ("pictures", "Images", "PICTURES", "Pictures"),
          ("music", "Musique", "MUSIC", "Music"), ("videos", "Vidéos", "VIDEOS", "Videos")]
TEXT_LIMIT = 200_000










def describe(entry_path, name):
    try:
        info = os.lstat(entry_path)
    except OSError:
        return None
    is_link = os.path.islink(entry_path)
    try:
        target = os.stat(entry_path) if is_link else info
    except OSError:
        target = info
    is_dir = os.path.isdir(entry_path)
    mime = None if is_dir else (mimetypes.guess_type(name)[0] or "")
    return {"n": name, "d": is_dir, "s": 0 if is_dir else target.st_size, "m": target.st_mtime,
            "l": is_link, "t": mime, "p": None}


def count_children(path, hidden):
    try:
        with os.scandir(path) as scan:
            return sum(1 for e in scan if hidden or not e.name.startswith("."))
    except OSError:
        return None


def list_dir(full, hidden=False):
    items = []
    with os.scandir(full) as scan:
        for entry in scan:
            item = describe(entry.path, entry.name)
            if item:
                item["p"] = relative(os.path.join(full, entry.name))
                items.append(item)
    for item in [i for i in items if i["d"]][:300]:
        item["c"] = count_children(os.path.join(Config.root, item["p"]), hidden)
    return items


def search_dir(full, query, hidden, fuzzy=False):
    query, found, deadline = query.lower(), [], time.monotonic() + 4
    for current, dirs, files in os.walk(full, followlinks=False):
        if not hidden:
            dirs[:] = [d for d in dirs if not d.startswith(".")]
        for name in dirs + files:
            score = ops.fuzzy_score(query, name) if fuzzy else (0 if query in name.lower() else None)
            if score is not None and (hidden or not name.startswith(".")):
                item = describe(os.path.join(current, name), name)
                if item:
                    item["p"] = relative(os.path.join(current, name))
                    item["_s"] = score
                    found.append(item)
        if len(found) >= (2000 if fuzzy else 400) or time.monotonic() > deadline:
            break
    found.sort(key=lambda i: (i["_s"], len(i["p"])))
    for item in found:
        item.pop("_s")
    return found[:80] if fuzzy else found


def read_text(full):
    with open(full, "rb") as handle:
        chunk = handle.read(TEXT_LIMIT + 1)
    if b"\0" in chunk[:8192]:
        return {"binary": True, "text": "", "truncated": False}
    return {"binary": False, "text": chunk[:TEXT_LIMIT].decode("utf-8", "replace"),
            "truncated": len(chunk) > TEXT_LIMIT}


TRASH = ".local/share/Trash/files"
NETWORK_TYPES = {"nfs", "nfs4", "cifs", "smb3", "fuse.sshfs", "fuse.rclone", "davfs", "fuse.davfs"}




def item_at(full):
    item = describe(full, os.path.basename(full) or full)
    if item:
        item["p"] = relative(full)
    return item


def local_path(uri):
    parsed = urllib.parse.urlsplit(uri)
    return os.path.realpath(urllib.parse.unquote(parsed.path)) if parsed.scheme == "file" else None


def recent_items():
    store = os.path.expanduser("~/.local/share/recently-used.xbel")
    try:
        marks = ET.parse(store).getroot().iter("bookmark")
    except (OSError, ET.ParseError):
        return []
    rows = sorted(((m.get("modified", ""), m.get("href", "")) for m in marks), reverse=True)
    seen, out = set(), []
    for _, href in rows:
        full = local_path(href)
        if full and full not in seen and inside_root(full) and os.path.exists(full):
            seen.add(full)
            out.append(item_at(full))
        if len(out) >= 200:
            break
    return [i for i in out if i]


def favorite_items():
    out = []
    for name in ("~/.config/gtk-3.0/bookmarks", "~/.gtk-bookmarks"):
        try:
            lines = open(os.path.expanduser(name), encoding="utf-8").read().splitlines()
        except OSError:
            continue
        for line in lines:
            full = local_path(line.split(" ", 1)[0]) if line.strip() else None
            if full and inside_root(full) and os.path.exists(full) and full not in [o["_f"] for o in out]:
                item = item_at(full)
                if item:
                    item["_f"] = full
                    out.append(item)
    for item in out:
        item.pop("_f")
    return out


def network_items():
    out = []
    try:
        lines = open("/proc/mounts", encoding="utf-8").read().splitlines()
    except OSError:
        return out
    for line in lines:
        parts = line.split()
        if len(parts) > 2 and parts[2] in NETWORK_TYPES:
            full = parts[1].replace("\\040", " ")
            item = item_at(full) if inside_root(full) and os.path.isdir(full) else None
            if item:
                out.append(item)
    return out


def mounted_disks():
    out, seen = [], set()
    try:
        lines = open("/proc/mounts", encoding="utf-8").read().splitlines()
    except OSError:
        return out
    for line in lines:
        parts = line.split()
        if len(parts) < 3 or not parts[0].startswith("/dev/") or parts[0].startswith("/dev/loop"):
            continue
        mount = parts[1].replace("\\040", " ")
        if parts[0] in seen or mount.startswith(("/boot", "/var/lib", "/snap")):
            continue
        seen.add(parts[0])
        try:
            st = os.statvfs(mount)
        except OSError:
            continue
        name = "Système" if mount == "/" else os.path.basename(mount.rstrip("/")) or mount
        out.append({"n": name, "m": mount, "dev": parts[0], "fs": parts[2], "t": st.f_blocks * st.f_frsize,
                    "f": st.f_bavail * st.f_frsize, "p": relative(mount) if inside_root(mount) else None})
    return out


VIEWS = {"recent": recent_items, "favorites": favorite_items, "network": network_items}


MAX_BODY = 65536










def make_folder(parent_rel, name):
    parent = resolve(parent_rel)
    if in_trash(os.path.join(parent, "x")):
        raise Refus(400, "Impossible de créer un dossier dans la corbeille")
    target = os.path.join(parent, clean_name(name))
    os.mkdir(target)
    return relative(target)


def unique_trash_name(files, name):
    candidate, n = name, 1
    while os.path.lexists(os.path.join(files, candidate)):
        n += 1
        candidate = f"{name}.{n}"
    return candidate


def move_to_trash(full):
    base = trash_dir()
    files, infos = os.path.join(base, "files"), os.path.join(base, "info")
    os.makedirs(files, mode=0o700, exist_ok=True)
    os.makedirs(infos, mode=0o700, exist_ok=True)
    name = unique_trash_name(files, os.path.basename(full))
    stamp = datetime.datetime.now().strftime("%Y-%m-%dT%H:%M:%S")
    info = f"[Trash Info]\nPath={urllib.parse.quote(full)}\nDeletionDate={stamp}\n"
    with open(os.path.join(infos, name + ".trashinfo"), "w", encoding="utf-8") as handle:
        handle.write(info)
    try:
        os.rename(full, os.path.join(files, name))
    except OSError:
        os.remove(os.path.join(infos, name + ".trashinfo"))
        raise Refus(409, "Cet élément est sur un autre disque que la corbeille : suppression refusée")


def erase_for_good(full):
    if stat.S_ISDIR(os.lstat(full).st_mode):
        shutil.rmtree(full)
    else:
        os.remove(full)
    info = os.path.join(trash_dir(), "info", os.path.basename(full) + ".trashinfo")
    if os.path.dirname(full) == os.path.join(os.path.realpath(trash_dir()), "files") and os.path.exists(info):
        os.remove(info)


def delete_entry(rel):
    full = resolve_entry(rel)
    base = os.path.realpath(trash_dir())
    if full == base or base.startswith(full + os.sep) or full == os.path.join(base, "files"):
        raise Refus(400, "Ce dossier est protégé : il contient la corbeille")
    if in_trash(full):
        erase_for_good(full)
        return "definitive"
    move_to_trash(full)
    return "trash"


def existing_places():
    out = []
    if os.path.isdir(os.path.join(Config.root, TRASH)):
        out.append({"id": "trash", "label": "Corbeille", "path": TRASH})
    xdg = xdg_user_dirs()
    for key, label, name, default in PLACES:
        full = os.path.realpath(xdg.get(name) or os.path.join(Config.root, default))
        taken = [p["path"] for p in out]
        if inside_root(full) and os.path.isdir(full) and full != Config.root and relative(full) not in taken:
            out.append({"id": key, "label": label, "path": relative(full)})
    return out


def xdg_user_dirs():
    found = {}
    try:
        lines = open(os.path.expanduser("~/.config/user-dirs.dirs"), encoding="utf-8").read().splitlines()
    except OSError:
        return found
    for line in lines:
        key, _, value = line.partition("=")
        if key.startswith("XDG_") and key.endswith("_DIR"):
            found[key[4:-4]] = os.path.expanduser(value.strip().strip('"').replace("$HOME", "~"))
    return found


def parse_range(header, size):
    if not header or not header.startswith("bytes="):
        return None
    start, _, end = header[6:].partition("-")
    try:
        if start == "":
            first, last = max(size - int(end), 0), size - 1
        else:
            first, last = int(start), int(end) if end else size - 1
    except ValueError:
        return None
    last = min(last, size - 1)
    return (first, last) if 0 <= first <= last else None


class Handler(http.server.BaseHTTPRequestHandler):
    server_version = "Five"
    protocol_version = "HTTP/1.1"

    def log_message(self, *args):
        pass

    def send_json(self, payload, code=200):
        body = json.dumps(payload).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def authorised(self):
        jar = http.cookies.SimpleCookie(self.headers.get("Cookie", ""))
        morsel = jar.get("mk")
        return bool(morsel) and secrets.compare_digest(morsel.value, Config.token)

    def do_GET(self):
        try:
            if self.headers.get("Host", "") not in Config.hosts:
                raise Refus(403, "Hôte non autorisé")
            url = urllib.parse.urlsplit(self.path)
            query = {k: v[0] for k, v in urllib.parse.parse_qs(url.query).items()}
            self.route(url.path, query)
        except Refus as refus:
            self.send_json({"erreur": str(refus)}, refus.code)
        except PermissionError:
            self.send_json({"erreur": "Accès refusé par le système"}, 403)
        except (FileNotFoundError, NotADirectoryError):
            self.send_json({"erreur": "Élément introuvable"}, 404)
        except (BrokenPipeError, ConnectionResetError):
            pass
        except OSError as error:
            self.send_json({"erreur": error.strerror or "Erreur système"}, 500)

    def do_POST(self):
        try:
            if self.headers.get("Host", "") not in Config.hosts:
                raise Refus(403, "Hôte non autorisé")
            origin = self.headers.get("Origin")
            if self.headers.get("X-Five") != "1" or (origin and urllib.parse.urlsplit(origin).netloc not in Config.hosts):
                raise Refus(403, "Requête refusée")
            if not self.authorised():
                raise Refus(401, "Session absente : ouvrez l'adresse affichée dans le terminal")
            self.write_api(urllib.parse.urlsplit(self.path).path, self.read_body())
        except Refus as refus:
            self.send_json({"erreur": str(refus)}, refus.code)
        except FileExistsError:
            self.send_json({"erreur": "Un élément de ce nom existe déjà"}, 409)
        except PermissionError:
            self.send_json({"erreur": "Accès refusé par le système"}, 403)
        except (FileNotFoundError, NotADirectoryError):
            self.send_json({"erreur": "Élément introuvable"}, 404)
        except (BrokenPipeError, ConnectionResetError):
            pass
        except OSError as error:
            self.send_json({"erreur": error.strerror or "Erreur système"}, 500)

    def read_body(self):
        size = int(self.headers.get("Content-Length") or 0)
        if size > MAX_BODY:
            raise Refus(413, "Requête trop volumineuse")
        try:
            data = json.loads(self.rfile.read(size) or b"{}")
        except ValueError:
            raise Refus(400, "Corps JSON invalide")
        if not isinstance(data, dict):
            raise Refus(400, "Corps JSON invalide")
        return data

    def write_api(self, path, body):
        if path == "/api/mkdir":
            made = make_folder(str(body.get("p", "")), str(body.get("name", "")))
            return self.send_json({"p": made}, 201)
        if path == "/api/delete":
            paths = body.get("paths")
            if not isinstance(paths, list) or not paths or len(paths) > 100:
                raise Refus(400, "Liste de chemins invalide")
            modes = [delete_entry(str(p)) for p in paths]
            return self.send_json({"mode": modes[0], "count": len(modes)})
        if path == "/api/rename":
            pairs = [(str(i.get("p", "")), str(i.get("name", ""))) for i in body.get("items") or [] if isinstance(i, dict)]
            if not pairs or len(pairs) > 500:
                raise Refus(400, "Liste de renommages invalide")
            return self.send_json({"count": ops.rename_many(pairs)})
        if path == "/api/job":
            rels = [str(p) for p in body.get("paths") or []][:500]
            if not rels:
                raise Refus(400, "Aucun élément")
            return self.send_json({"id": ops.start_job(str(body.get("kind", "")), rels, str(body.get("dest", "")))}, 201)
        if path == "/api/job/control":
            ops.control_job(str(body.get("id", "")), str(body.get("action", "")))
            return self.send_json({"ok": True})
        if path == "/api/logout":
            return self.end_session()
        raise Refus(404, "Route inconnue")

    def end_session(self):
        body = b'{"ok": true}'
        self.send_response(200)
        self.send_header("Set-Cookie", "mk=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0")
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def route(self, path, query):
        if path == "/" and "k" in query:
            return self.start_session(query["k"])
        if path.startswith("/api/"):
            if not self.authorised():
                raise Refus(401, "Session absente : ouvrez l'adresse affichée dans le terminal")
            return self.api(path[5:], query)
        return self.static(path)

    def start_session(self, token):
        if not secrets.compare_digest(token, Config.token):
            raise Refus(401, "Jeton invalide")
        self.send_response(302)
        self.send_header("Set-Cookie", f"mk={token}; HttpOnly; SameSite=Strict; Path=/")
        self.send_header("Location", "/")
        self.send_header("Content-Length", "0")
        self.end_headers()

    def static(self, path):
        name = "index.html" if path == "/" else path.lstrip("/")
        full = os.path.realpath(os.path.join(WEB, name))
        if not full.startswith(WEB + os.sep) or not os.path.isfile(full):
            raise Refus(404, "Introuvable")
        ctype = STATIC_TYPES.get(os.path.splitext(full)[1], "application/octet-stream")
        self.send_file(full, ctype, None, extra={"Cache-Control": "no-cache"})

    def api(self, name, query):
        rel = query.get("p", "")
        if name == "places":
            return self.send_json({"places": existing_places(), "root": Config.root, "user": getpass.getuser()})
        if name == "info":
            return self.send_json({"root": Config.root, "port": Config.port, "trash": TRASH,
                                   "python": sys.version.split()[0], "user": getpass.getuser()})
        if name == "disks":
            return self.send_json({"disks": mounted_disks()})
        if name == "view":
            return self.api_view(query)
        if name == "jobs":
            return self.send_json({"jobs": ops.jobs_view()})
        full = resolve(rel)
        if name in ("usage", "duplicates", "git", "archive"):
            return self.api_analysis(name, full, query)
        if name == "list":
            return self.api_list(full, query)
        if name == "text":
            return self.send_json(read_text(self.regular(full)))
        if name == "raw":
            return self.api_raw(full, query)
        raise Refus(404, "Route inconnue")

    def api_view(self, query):
        build = VIEWS.get(query.get("v", ""))
        if not build:
            raise Refus(404, "Vue inconnue")
        hidden = query.get("h") == "1"
        items = [i for i in build() if hidden or not i["n"].startswith(".")]
        usage = shutil.disk_usage(Config.root)
        self.send_json({"path": "", "items": items, "free": usage.free, "total": usage.total})

    def api_analysis(self, name, full, query):
        if name == "git":
            return self.send_json({"marks": ops.git_marks(self.folder(full))})
        if name == "archive":
            return self.send_json(ops.list_archive(self.regular(full)))
        if name == "usage":
            return self.send_json(ops.usage_of(self.folder(full)))
        minimum = max(int(query.get("min", "1024") or 1024), 1)
        return self.send_json(ops.find_duplicates(self.folder(full), minimum))

    def folder(self, full):
        if not os.path.isdir(full):
            raise Refus(400, "Ce n'est pas un dossier")
        return full

    def regular(self, full):
        if not os.path.isfile(full):
            raise Refus(400, "Ce n'est pas un fichier")
        return full

    def api_list(self, full, query):
        hidden = query.get("h") == "1"
        if "q" in query and query["q"].strip():
            items = search_dir(full, query["q"].strip(), hidden, query.get("f") == "1")
        else:
            items = [i for i in list_dir(full, hidden) if hidden or not i["n"].startswith(".")]
        usage = shutil.disk_usage(full)
        self.send_json({"path": relative(full), "items": items, "free": usage.free, "total": usage.total})

    def api_raw(self, full, query):
        self.regular(full)
        ctype = mimetypes.guess_type(full)[0] or "application/octet-stream"
        disposition = None
        if query.get("dl") == "1":
            quoted = urllib.parse.quote(os.path.basename(full))
            disposition = f"attachment; filename*=UTF-8''{quoted}"
        self.send_file(full, ctype, disposition, sandbox=ctype != "application/pdf")

    def send_file(self, full, ctype, disposition, extra=None, sandbox=False):
        size = os.path.getsize(full)
        span = parse_range(self.headers.get("Range"), size)
        first, last = span if span else (0, size - 1)
        self.send_response(206 if span else 200)
        headers = {"Content-Type": ctype, "Accept-Ranges": "bytes", "X-Content-Type-Options": "nosniff",
                   "Content-Length": str(max(last - first + 1, 0))}
        if span:
            headers["Content-Range"] = f"bytes {first}-{last}/{size}"
        if disposition:
            headers["Content-Disposition"] = disposition
        if sandbox:
            headers["Content-Security-Policy"] = "sandbox"
        headers.update(extra or {})
        for key, value in headers.items():
            self.send_header(key, value)
        self.end_headers()
        self.stream(full, first, last)

    def stream(self, full, first, last):
        remaining = last - first + 1
        with open(full, "rb") as handle:
            handle.seek(first)
            while remaining > 0:
                chunk = handle.read(min(65536, remaining))
                if not chunk:
                    break
                self.wfile.write(chunk)
                remaining -= len(chunk)


def main():
    parser = argparse.ArgumentParser(description="FIVE, lecture seule")
    parser.add_argument("--root", default="~", help="dossier racine exposé (défaut : votre dossier personnel)")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--open", action="store_true", help="ouvre le navigateur")
    args = parser.parse_args()
    Config.root = os.path.realpath(os.path.expanduser(args.root))
    Config.port = args.port
    Config.hosts = {f"127.0.0.1:{args.port}", f"localhost:{args.port}"}
    server = http.server.ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    server.daemon_threads = True
    address = f"http://127.0.0.1:{args.port}/?k={Config.token}"
    print(f"FIVE — racine : {Config.root}\nOuvrez : {address}", flush=True)
    if args.open:
        import webbrowser
        webbrowser.open(address)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        sys.exit(0)


if __name__ == "__main__":
    main()
