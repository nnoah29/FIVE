"""Opérations de FIVE : renommage, transferts, analyse disque, doublons, Git, archives, recherche floue."""

import hashlib
import os
import shutil
import stat
import subprocess
import tarfile
import threading
import time
import uuid
import zipfile

from coeur import Config, Refus, clean_name, in_trash, relative, resolve, resolve_entry

CHUNK = 1 << 20
FINISHED_KEPT = 12


def fuzzy_score(query, name):
    """Retourne un score (plus petit = meilleur) si les lettres de query suivent l'ordre dans name, sinon None."""
    q, n = query.lower(), name.lower()
    if q in n:
        return n.index(q)
    pos, gaps, first = -1, 0, None
    for ch in q:
        nxt = n.find(ch, pos + 1)
        if nxt < 0:
            return None
        first = nxt if first is None else first
        gaps += nxt - pos - 1 if pos >= 0 else 0
        pos = nxt
    return 1000 + gaps * 4 + first


def rename_many(pairs):
    """Renomme plusieurs éléments d'un coup, sans écraser, en deux phases pour gérer les permutations."""
    plan = []
    for rel, new in pairs:
        full = resolve_entry(rel)
        if in_trash(full):
            raise Refus(400, "Impossible de renommer dans la corbeille")
        plan.append((full, os.path.join(os.path.dirname(full), clean_name(new))))
    sources = {src for src, _ in plan}
    targets = [dst for _, dst in plan]
    if len(set(targets)) != len(targets):
        raise Refus(409, "Deux éléments auraient le même nom")
    for src, dst in plan:
        if src != dst and os.path.lexists(dst) and dst not in sources:
            raise Refus(409, f"« {os.path.basename(dst)} » existe déjà")
    staged = []
    for src, dst in plan:
        if src == dst:
            continue
        tmp = os.path.join(os.path.dirname(src), f".five-{uuid.uuid4().hex}")
        os.rename(src, tmp)
        staged.append((tmp, dst))
    for tmp, dst in staged:
        os.rename(tmp, dst)
    return len(staged)


def unique_destination(folder, name):
    candidate, n = name, 1
    stem, dot, ext = name.rpartition(".")
    stem, ext = (stem, "." + ext) if dot and stem else (name, "")
    while os.path.lexists(os.path.join(folder, candidate)):
        n += 1
        candidate = f"{stem} ({n}){ext}"
    return os.path.join(folder, candidate)


class Job:
    def __init__(self, kind, sources, dest):
        self.id = uuid.uuid4().hex[:10]
        self.kind, self.sources, self.dest = kind, sources, dest
        self.state, self.error = "running", ""
        self.total = self.done = 0
        self.files = self.files_done = 0
        self.current = ""
        self.gate = threading.Event()
        self.gate.set()
        self.cancelled = False

    def view(self):
        return {"id": self.id, "kind": self.kind, "state": self.state, "error": self.error, "total": self.total,
                "done": self.done, "files": self.files, "filesDone": self.files_done, "current": self.current,
                "dest": relative(self.dest), "count": len(self.sources)}

    def checkpoint(self):
        self.gate.wait()
        if self.cancelled:
            raise InterruptedError


JOBS = {}
JOBS_LOCK = threading.Lock()


def measure(path):
    if os.path.islink(path) or not os.path.isdir(path):
        return os.lstat(path).st_size, 1
    size = count = 0
    for current, _dirs, files in os.walk(path, followlinks=False):
        for name in files:
            try:
                size += os.lstat(os.path.join(current, name)).st_size
            except OSError:
                pass
            count += 1
    return size, max(count, 1)


def copy_file(job, src, dst):
    job.current = os.path.basename(src)
    if os.path.islink(src):
        os.symlink(os.readlink(src), dst)
        job.files_done += 1
        return
    try:
        with open(src, "rb") as reader, open(dst, "xb") as writer:
            while True:
                job.checkpoint()
                block = reader.read(CHUNK)
                if not block:
                    break
                writer.write(block)
                job.done += len(block)
        shutil.copystat(src, dst, follow_symlinks=False)
    except InterruptedError:
        if os.path.lexists(dst):
            os.remove(dst)
        raise
    job.files_done += 1


def copy_tree(job, src, dst):
    if os.path.islink(src) or not os.path.isdir(src):
        return copy_file(job, src, dst)
    os.mkdir(dst)
    for entry in sorted(os.listdir(src)):
        job.checkpoint()
        copy_tree(job, os.path.join(src, entry), os.path.join(dst, entry))
    shutil.copystat(src, dst, follow_symlinks=False)


def remove_source(path):
    if stat.S_ISDIR(os.lstat(path).st_mode):
        shutil.rmtree(path)
    else:
        os.remove(path)


def transfer_one(job, src):
    dst = unique_destination(job.dest, os.path.basename(src))
    if job.kind == "move":
        try:
            os.rename(src, dst)
            size, count = measure(dst)
            job.done += size
            job.files_done += count
            return
        except OSError:
            pass
    copy_tree(job, src, dst)
    if job.kind == "move":
        remove_source(src)


def run_job(job):
    try:
        for src in job.sources:
            job.checkpoint()
            transfer_one(job, src)
        job.state = "done"
    except InterruptedError:
        job.state = "cancelled"
    except OSError as error:
        job.state, job.error = "error", error.strerror or "Erreur système"
    job.current = ""


def prune_jobs():
    ended = [j for j in JOBS.values() if j.state != "running"]
    for job in ended[:-FINISHED_KEPT]:
        JOBS.pop(job.id, None)


def start_job(kind, rels, dest_rel):
    if kind not in ("copy", "move"):
        raise Refus(400, "Type de transfert inconnu")
    dest = resolve(dest_rel)
    if not os.path.isdir(dest) or in_trash(os.path.join(dest, "x")):
        raise Refus(400, "Destination invalide")
    sources = [resolve_entry(r) for r in rels]
    for src in sources:
        if os.path.isdir(src) and not os.path.islink(src) and (dest == src or dest.startswith(src + os.sep)):
            raise Refus(400, "Impossible de copier un dossier dans lui-même")
        if kind == "move" and os.path.dirname(src) == dest:
            raise Refus(400, "Les éléments sont déjà dans ce dossier")
    job = Job(kind, sources, dest)
    for src in sources:
        size, count = measure(src)
        job.total += size
        job.files += count
    with JOBS_LOCK:
        prune_jobs()
        JOBS[job.id] = job
    threading.Thread(target=run_job, args=(job,), daemon=True).start()
    return job.id


def control_job(job_id, action):
    job = JOBS.get(job_id)
    if not job:
        raise Refus(404, "Transfert introuvable")
    if action == "pause" and job.state == "running":
        job.gate.clear()
        job.state = "paused"
    elif action == "resume" and job.state == "paused":
        job.state = "running"
        job.gate.set()
    elif action == "cancel" and job.state in ("running", "paused"):
        job.cancelled = True
        job.gate.set()
    elif action not in ("pause", "resume", "cancel"):
        raise Refus(400, "Action inconnue")


def jobs_view():
    return [j.view() for j in list(JOBS.values())]


def usage_of(full, budget=10.0):
    """Taille cumulée de chaque enfant direct de full, avec limite de temps."""
    deadline, rows, partial = time.monotonic() + budget, [], False
    with os.scandir(full) as scan:
        children = sorted(scan, key=lambda e: e.name)
    for entry in children:
        size, is_dir = 0, entry.is_dir(follow_symlinks=False)
        if is_dir:
            size, partial = dir_size(entry.path, deadline, partial)
        else:
            try:
                size = entry.stat(follow_symlinks=False).st_size
            except OSError:
                pass
        rows.append({"n": entry.name, "d": is_dir, "s": size, "p": relative(entry.path)})
    rows.sort(key=lambda r: -r["s"])
    return {"items": rows[:200], "total": sum(r["s"] for r in rows), "partial": partial or time.monotonic() > deadline}


def dir_size(path, deadline, partial):
    size = 0
    for current, _dirs, files in os.walk(path, followlinks=False):
        if time.monotonic() > deadline:
            return size, True
        for name in files:
            try:
                size += os.lstat(os.path.join(current, name)).st_size
            except OSError:
                pass
    return size, partial


def file_hash(path, deadline):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        while True:
            if time.monotonic() > deadline:
                return None
            block = handle.read(CHUNK)
            if not block:
                return digest.hexdigest()
            digest.update(block)


def collect_by_size(full, minimum, deadline):
    sizes = {}
    for current, dirs, files in os.walk(full, followlinks=False):
        dirs[:] = [d for d in dirs if not d.startswith(".")]
        for name in files:
            path = os.path.join(current, name)
            try:
                info = os.lstat(path)
            except OSError:
                continue
            if stat.S_ISREG(info.st_mode) and info.st_size >= minimum:
                sizes.setdefault(info.st_size, []).append(path)
        if time.monotonic() > deadline:
            return sizes, True
    return sizes, False


def find_duplicates(full, minimum=1024, budget=20.0):
    """Groupes de fichiers identiques : même taille, puis même empreinte SHA-256."""
    deadline = time.monotonic() + budget
    sizes, partial = collect_by_size(full, minimum, deadline)
    groups = []
    for size, paths in sizes.items():
        if len(paths) < 2:
            continue
        by_hash = {}
        for path in paths:
            try:
                digest = file_hash(path, deadline)
            except OSError:
                continue
            if digest is None:
                partial = True
                break
            by_hash.setdefault(digest, []).append(path)
        groups += [{"size": size, "files": sorted(relative(p) for p in same)} for same in by_hash.values() if len(same) > 1]
    groups.sort(key=lambda g: -g["size"] * (len(g["files"]) - 1))
    return {"groups": groups[:150], "partial": partial,
            "wasted": sum(g["size"] * (len(g["files"]) - 1) for g in groups)}


GIT_FLAGS = ["-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null", "-c", "core.untrackedCache=false"]


def run_git(folder, args):
    env = {**os.environ, "GIT_OPTIONAL_LOCKS": "0", "GIT_TERMINAL_PROMPT": "0", "GIT_CONFIG_NOSYSTEM": "1"}
    try:
        done = subprocess.run(["git", *GIT_FLAGS, "-C", folder, *args], capture_output=True, timeout=4, env=env, check=False)
    except (OSError, subprocess.SubprocessError):
        return None
    return done.stdout if done.returncode == 0 else None


def git_marks(full):
    """État Git des enfants directs de full : {nom: code} (M modifié, A ajouté, D supprimé, ? non suivi, ! ignoré, U conflit)."""
    top = run_git(full, ["rev-parse", "--show-toplevel"])
    out = run_git(full, ["status", "--porcelain=v1", "-z", "--ignored=matching", "--untracked-files=normal"])
    if not top or out is None:
        return {}
    top = os.path.realpath(top.decode().strip())
    marks = {}
    for record in out.split(b"\0"):
        if len(record) < 4:
            continue
        code, path = record[:2].decode(), os.path.join(top, record[3:].decode().rstrip("/"))
        rel = os.path.relpath(path, full)
        if rel.startswith(".."):
            continue
        name = rel.split(os.sep)[0]
        marks[name] = merge_mark(marks.get(name), mark_for(code))
    return marks


def mark_for(code):
    if "U" in code or code in ("AA", "DD"):
        return "U"
    if code == "!!":
        return "!"
    if code == "??":
        return "?"
    return "D" if "D" in code else "A" if "A" in code else "M"


def merge_mark(old, new):
    order = "UMADL?!"
    return new if old is None or order.index(new) < order.index(old) else old


ARCHIVE_LIMIT = 500


def list_archive(full):
    """Contenu d'une archive zip ou tar (500 entrées au plus), sans rien extraire."""
    rows = []
    if zipfile.is_zipfile(full):
        with zipfile.ZipFile(full) as archive:
            for info in archive.infolist()[:ARCHIVE_LIMIT]:
                rows.append({"n": info.filename, "s": info.file_size, "d": info.is_dir()})
            return {"items": rows, "more": len(archive.infolist()) > ARCHIVE_LIMIT}
    try:
        with tarfile.open(full, "r:*") as archive:
            for member in archive:
                if len(rows) >= ARCHIVE_LIMIT:
                    return {"items": rows, "more": True}
                rows.append({"n": member.name, "s": member.size, "d": member.isdir()})
    except (tarfile.TarError, EOFError) as error:
        raise Refus(400, "Archive illisible : " + str(error))
    return {"items": rows, "more": False}
