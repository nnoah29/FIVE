"""Noyau partagé : racine exposée, refus HTTP, résolution sûre des chemins."""

import os
import secrets


class Config:
    port = 8765
    root = os.path.realpath(os.path.expanduser("~"))
    token = secrets.token_urlsafe(24)
    hosts = set()


class Refus(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code


def resolve(rel):
    base = Config.root
    full = os.path.realpath(os.path.join(base, rel.strip("/")))
    prefix = base if base.endswith("/") else base + "/"
    if full != base and not full.startswith(prefix):
        raise Refus(403, "Chemin hors de la racine autorisée")
    return full


def relative(full):
    rel = os.path.relpath(full, Config.root)
    return "" if rel == "." else rel


def inside_root(full):
    prefix = Config.root if Config.root.endswith("/") else Config.root + "/"
    return full == Config.root or full.startswith(prefix)


def clean_name(name):
    name = (name or "").strip()
    if not name or name in (".", "..") or "/" in name or "\0" in name or len(name.encode()) > 255:
        raise Refus(400, "Nom invalide : sans « / », ni vide, 255 octets au plus")
    return name


def resolve_entry(rel):
    parent, name = os.path.split(rel.strip("/"))
    if not name or name in (".", ".."):
        raise Refus(400, "Chemin invalide")
    full = os.path.join(resolve(parent), name)
    if full == Config.root or not os.path.lexists(full):
        raise Refus(404, "Élément introuvable")
    return full


def trash_dir():
    return os.path.join(os.path.expanduser("~"), ".local/share/Trash")


def in_trash(full):
    base = os.path.join(os.path.realpath(trash_dir()), "files")
    return os.path.dirname(full) == base or full.startswith(base + os.sep)
