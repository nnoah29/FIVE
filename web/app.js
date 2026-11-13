"use strict";

const KINDS = {
  image: ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "avif", "ico"],
  video: ["mp4", "mkv", "webm", "mov", "avi", "m4v"],
  audio: ["mp3", "flac", "ogg", "wav", "m4a", "opus", "aac"],
  archive: ["zip", "tar", "gz", "xz", "7z", "rar", "bz2", "zst", "deb", "iso"],
  code: ["rs", "js", "ts", "py", "dart", "c", "cpp", "h", "go", "java", "sh", "html", "css", "json", "toml", "yaml", "yml", "sql", "lua", "kt"],
  doc: ["pdf", "md", "txt", "doc", "docx", "odt", "rtf", "csv", "xls", "xlsx", "ods", "ppt", "pptx", "epub", "log"],
};
const KIND_LABEL = { dir: "Dossier", image: "Image", video: "Vidéo", audio: "Audio", doc: "Document",
  archive: "Archive", code: "Code", other: "Autre" };
const KIND_COLOR = { dir: "var(--accent)", image: "var(--accent)", video: "#5b8def", audio: "#9b7be0", doc: "#4fb286",
  archive: "#d9a441", code: "#3fa9b8", other: "#8b8a84" };
const FILTERS = [["all", "Tout"], ["image", "Images"], ["video", "Vidéos"], ["audio", "Audio"],
  ["doc", "Documents"], ["archive", "Archives"], ["code", "Code"]];
const ICON_BY_KIND = { dir: "folder", image: "image", video: "video", audio: "audio", archive: "archive",
  code: "code", doc: "text", other: "file" };
const MONTHS = new Intl.DateTimeFormat("fr", { day: "numeric", month: "short", year: "numeric" });
const CLOCK = new Intl.DateTimeFormat("fr", { hour: "2-digit", minute: "2-digit" });

const state = { path: "", items: [], shown: [], sel: -1, picked: new Set(), anchor: -1, clip: null, usage: null, git: {}, gitPath: null, pending: null, view: "list", sort: "n", asc: true, filter: "all",
  hidden: false, settings: false, virtual: null, query: "", places: [], root: "", user: "", free: 0, total: 0, token: 0 };
const $ = (id) => document.getElementById(id);
const DEFAULTS = { theme: "system", view: "list", sort: "n", asc: true, hidden: false, units: "si", dates: "short",
  thumbs: true, newTab: true, confirmTrash: true, accent: "", density: "comfortable", git: true, restore: true };
const S = { ...DEFAULTS };
const STORE = "five-reglages";

function el(tag, attrs = {}, ...kids) {
  const node = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (k === "class") node.className = v;
    else if (k.startsWith("on")) node.addEventListener(k.slice(2), v);
    else node.setAttribute(k, v);
  }
  node.append(...kids.filter((k) => k !== null && k !== undefined));
  return node;
}
function icon(name) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("class", "ic");
  svg.setAttribute("viewBox", "0 0 24 24");
  const use = document.createElementNS("http://www.w3.org/2000/svg", "use");
  use.setAttribute("href", "#i-" + name);
  svg.append(use);
  return svg;
}
const ext = (n) => (n.includes(".") ? n.split(".").pop().toLowerCase() : "");
function kindOf(item) {
  if (item.d) return "dir";
  const e = ext(item.n);
  return Object.keys(KINDS).find((k) => KINDS[k].includes(e)) || "other";
}
function formatSize(n) {
  const iec = S.units === "iec";
  const base = iec ? 1024 : 1000;
  if (n < base) return n + " o";
  const units = iec ? ["Kio", "Mio", "Gio", "Tio"] : ["Ko", "Mo", "Go", "To"];
  let i = -1;
  do { n /= base; i++; } while (n >= base && i < 3);
  return n.toLocaleString("fr", { maximumFractionDigits: n < 10 ? 1 : 0 }) + " " + units[i];
}
function formatDate(t) {
  const d = new Date(t * 1000);
  if (S.dates === "full") return MONTHS.format(d) + " " + CLOCK.format(d);
  return d.toDateString() === new Date().toDateString() ? CLOCK.format(d) : MONTHS.format(d);
}
function toast(text) {
  const t = $("toast");
  t.textContent = text;
  t.classList.add("on");
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => t.classList.remove("on"), 2200);
}
const rawUrl = (p, dl) => "/api/raw?p=" + encodeURIComponent(p) + (dl ? "&dl=1" : "");
const joinPath = (a, b) => (a ? a + "/" + b : b);
const parentOf = (p) => p.split("/").slice(0, -1).join("/");

async function api(route, params) {
  const res = await fetch("/api/" + route + "?" + new URLSearchParams(params));
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.erreur || "Erreur " + res.status);
  return data;
}

const TRASH = ".local/share/Trash/files";
const inTrash = (p) => p === TRASH || p.startsWith(TRASH + "/");
const canCreate = () => !state.virtual && !searching() && !inTrash(state.path);
async function post(route, body) {
  const res = await fetch("/api/" + route, { method: "POST", body: JSON.stringify(body),
    headers: { "Content-Type": "application/json", "X-Five": "1" } });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.erreur || "Erreur " + res.status);
  return data;
}
function dialog({ title, text, names, input, confirm, submit }) {
  const previous = document.activeElement;
  const field = input === undefined ? null : el("input", { class: "field", type: "text", value: input, maxlength: "255",
    spellcheck: "false", autocomplete: "off", "aria-label": title });
  const error = el("div", { class: "form-error mono", role: "alert" });
  const ok = el("button", { class: "btn go", type: "button" }, confirm);
  const cancel = el("button", { class: "btn", type: "button" }, "Annuler");
  const box = el("div", { class: "dialog", role: "dialog", "aria-modal": "true", "aria-label": title },
    el("h2", {}, title), text ? el("p", {}, text) : null,
    names ? el("ul", {}, ...names.map((n) => el("li", {}, n))) : null, field, error,
    el("div", { class: "dialog-actions" }, cancel, ok));
  const overlay = el("div", { class: "overlay" }, box);
  const close = () => { overlay.remove(); previous?.focus?.(); };
  const run = async () => {
    ok.disabled = true;
    error.textContent = "";
    try { await submit(field ? field.value : undefined); close(); }
    catch (e) { error.textContent = e.message; ok.disabled = false; field?.focus(); }
  };
  ok.onclick = run;
  cancel.onclick = close;
  overlay.addEventListener("mousedown", (e) => { if (e.target === overlay) close(); });
  overlay.addEventListener("keydown", (e) => {
    if (e.key === "Escape") { e.stopPropagation(); close(); }
    else if (e.key === "Enter" && e.target.tagName !== "BUTTON") { e.preventDefault(); run(); }
  });
  document.body.append(overlay);
  (field || ok).focus();
  field?.select();
}
function createFolder() {
  if (!canCreate()) return toast("Ouvrez un dossier ordinaire pour y créer un dossier");
  dialog({ title: "Nouveau dossier", text: "Il sera créé dans /" + state.path, input: "", confirm: "Créer",
    submit: async (name) => { await post("mkdir", { p: state.path, name }); toast("Dossier créé"); reload(); } });
}
function deleteItems(items) {
  if (!items.length) return;
  const final = items.every((i) => inTrash(i.p));
  const submit = async () => {
    await post("delete", { paths: items.map((i) => i.p) });
    toast(final ? "Supprimé définitivement" : "Mis à la corbeille");
    state.sel = -1;
    reload();
  };
  if (!final && !S.confirmTrash) { submit().catch((e) => toast(e.message)); return; }
  dialog({ title: final ? "Supprimer définitivement ?" : "Mettre à la corbeille ?",
    text: final ? "Cette action est irréversible." : "Vous pourrez le récupérer depuis la corbeille de votre système.",
    names: items.map((i) => "/" + i.p), confirm: final ? "Supprimer définitivement" : "Mettre à la corbeille", submit });
}
const VIEW_LABEL = { recent: "Récents", favorites: "Favoris", network: "Réseau" };
const reload = () => load(state.path, state.virtual);
async function load(path, virtual = null) {
  const ticket = ++state.token;
  try {
    const q = state.query.trim();
    const h = state.hidden ? "1" : "0";
    const data = virtual ? await api("view", { v: virtual, h }) : await api("list", { p: path, h, ...(q ? { q } : {}) });
    if (ticket !== state.token) return;
    Object.assign(state, { path: data.path, items: data.items, free: data.free, total: data.total, sel: -1, picked: new Set(), virtual });
    render();
    showPreview();
    afterLoad();
  } catch (e) {
    if (ticket !== state.token) return;
    state.items = [];
    render(e.message);
    toast(e.message);
  }
}

function applyFilter(item) {
  return state.filter === "all" || kindOf(item) === state.filter;
}
function sortItems(list) {
  const cmp = {
    n: (a, b) => a.n.localeCompare(b.n, "fr", { numeric: true, sensitivity: "base" }),
    t: (a, b) => kindOf(a).localeCompare(kindOf(b)) || a.n.localeCompare(b.n, "fr"),
    m: (a, b) => a.m - b.m,
    s: (a, b) => a.s - b.s,
  }[state.sort];
  const sign = state.asc ? 1 : -1;
  return [...list].sort((a, b) => (b.d - a.d) || sign * cmp(a, b));
}
const searching = () => !!state.query.trim() && !state.virtual;
const listAll = () => searching() || !!state.virtual;

function render(error) {
  if (state.sel < 0) state.picked.clear();
  const files = state.items.filter((i) => listAll() || !i.d);
  state.shown = sortItems(files.filter(applyFilter));
  renderSide();
  renderHead();
  renderCards();
  renderTable();
  renderItems(error);
  updatePaste();
}
function renderSide() {
  $("count").textContent = state.items.length.toLocaleString("fr");
  $("countLabel").textContent = searching() ? "résultats" : state.items.length > 1 ? "éléments" : "élément";
  $("up").disabled = !state.path || !!state.virtual;
  $("newFolder").disabled = !canCreate();
  const trash = state.places.find((p) => p.id === "trash");
  const main = [{ id: "home", label: "Dossier personnel", path: "" }, { id: "recent", label: "Récents", v: "recent" },
    { id: "favorites", label: "Favoris", v: "favorites" }, { id: "network", label: "Réseau", v: "network" }, trash].filter(Boolean);
  const others = state.places.filter((p) => p.id !== "trash");
  const entry = (p) => {
    const here = p.v ? state.virtual === p.v : !state.virtual && !searching() &&
      (p.path === "" ? state.path === "" : state.path === p.path || state.path.startsWith(p.path + "/"));
    return el("button", { type: "button", "aria-current": String(!!here),
      onclick: () => (p.v ? goView(p.v) : go(p.path)) }, icon(p.id), p.label);
  };
  $("places").replaceChildren(...main.map(entry),
    ...(others.length ? [el("div", { class: "nav-title" }, "Dossiers"), ...others.map(entry)] : []));
  $("tags").replaceChildren(el("div", { class: "nav-title" }, "Disques"),
    ...(state.disks || []).map((d) => {
      const ratio = d.t ? (d.t - d.f) / d.t : 0;
      const here = d.p !== null && !state.virtual && (d.p === state.path || state.path.startsWith(d.p + "/"));
      return el("button", { type: "button", class: "disk", disabled: d.p === null, "aria-current": String(!!here),
        title: d.m + " · " + d.fs + (d.p === null ? " (hors de la racine)" : ""),
        onclick: () => go(d.p) }, icon("drive"), el("span", { class: "n" }, d.n),
      el("small", {}, formatSize(d.t - d.f) + " / " + formatSize(d.t)),
      el("i", { class: "bar", style: `width:${Math.round(ratio * 100)}%` }));
    }));
}
function setFilter(id) {
  state.filter = state.filter === id ? "all" : id;
  state.sel = -1;
  render();
  showPreview();
}
function renderHead() {
  const name = state.path === TRASH ? "Corbeille" : state.path ? state.path.split("/").pop() : "Dossier personnel";
  $("title").textContent = state.virtual ? VIEW_LABEL[state.virtual] : searching() ? "Recherche" : name;
  $("filesTitle").textContent = state.virtual ? "Éléments" : searching() ? "Résultats" : "Tous les fichiers";
  const parts = state.path && !state.virtual ? state.path.split("/") : [];
  const crumbs = [el("button", { type: "button", onclick: () => go("") }, state.root || "~")];
  if (state.virtual) crumbs.splice(0, crumbs.length, el("button", { type: "button" }, "Vue virtuelle"));
  const sep = (parent) => el("button", { class: "sep", type: "button", "aria-label": "Sous-dossiers de " + (parent || state.root),
    "aria-haspopup": "menu", onclick: (e) => crumbMenu(e.currentTarget, parent) }, "/");
  if (!state.virtual) crumbs.push(sep(""));
  parts.forEach((part, i) => {
    if (i > 0) crumbs.push(sep(parts.slice(0, i).join("/")));
    crumbs.push(el("button", { type: "button", onclick: () => go(parts.slice(0, i + 1).join("/")) }, part));
    if (i === parts.length - 1) crumbs.push(sep(parts.join("/")));
  });
  $("crumbs").replaceChildren(...crumbs);
  $("pills").replaceChildren(...FILTERS.map(([id, label]) =>
    el("button", { class: "pill", type: "button", "aria-pressed": String(state.filter === id),
      onclick: () => (id === "all" ? (state.filter = "all", state.sel = -1, render(), showPreview()) : setFilter(id)) }, label)));
  $("userName").textContent = state.user;
  $("userRoot").textContent = state.root;
  $("avatar").textContent = (state.user[0] || "?").toUpperCase();
}
function renderCards() {
  const dirs = listAll() ? [] : sortItems(state.items.filter((i) => i.d));
  $("cards").replaceChildren(...dirs.map((d) => {
    const n = d.c;
    const label = n === null || n === undefined ? "—" : n.toLocaleString("fr") + (n > 1 ? " éléments" : " élément");
    const open = el("button", { class: "card-main", type: "button", title: d.n, onclick: () => go(d.p) }, icon(d.l ? "link" : "folder"),
      el("div", {}, el("b", {}, d.n, gitBadge(d)), el("div", { class: "row2" }, el("span", {}, label), el("span", {}, formatDate(d.m)))));
    const dots = el("button", { class: "dots card-dots", type: "button", "aria-label": "Actions pour " + d.n, "aria-haspopup": "menu",
      onclick: (e) => openMenu(e.currentTarget, d) }, icon("more"));
    return el("div", { class: "card" }, open, dots);
  }));
}
function renderTable() {
  const cols = [["n", "Nom", ""], ["t", "Type", "c-tag"], ["s", "Taille", ""], ["m", "Modifié", "c-date"]];
  $("thead").hidden = state.view === "grid";
  $("thead").replaceChildren(el("span", { class: "sp" }),
    ...cols.map(([k, label, cls]) => {
      const on = state.sort === k;
      return el("button", { type: "button", class: cls, ...(on ? { "aria-sort": state.asc ? "ascending" : "descending" } : {}),
        onclick: () => sortBy(k) }, label, icon("sort"));
    }), el("span", { class: "sp" }));
}
function sortBy(key) {
  S.asc = S.sort === key ? !S.asc : key === "n" || key === "t";
  S.sort = key;
  saveSettings();
  state.asc = S.asc;
  state.sort = S.sort;
  state.sel = -1;
  render();
  showPreview();
}
function nameParts(n) {
  const i = n.lastIndexOf(".");
  return i > 0 ? [n.slice(0, i), n.slice(i)] : [n, ""];
}
function rowFor(item, i) {
  const k = kindOf(item);
  const sub = listAll() ? "/" + (parentOf(item.p) || "") : null;
  const base = { "data-i": i, "aria-selected": String(state.picked.has(item.p)), tabindex: "-1",
    onclick: (e) => select(i, e), ondblclick: () => open(i) };
  if (state.view === "grid") {
    const thumb = el("div", { class: "thumb" }, k === "image" && S.thumbs && item.s < 30e6
      ? el("img", { src: rawUrl(item.p), loading: "lazy", alt: "" }) : icon(item.l ? "link" : ICON_BY_KIND[k]));
    return el("div", { ...base, class: "tile" }, thumb, el("div", { class: "name", title: item.n }, item.n),
      el("div", { class: "meta" }, item.d ? "Dossier" : formatSize(item.s)));
  }
  const [stem, tail] = nameParts(item.n);
  const dots = el("button", { class: "dots", type: "button", "aria-label": "Actions pour " + item.n, "aria-haspopup": "menu",
    onclick: (e) => { e.stopPropagation(); openMenu(e.currentTarget, item, i); } }, icon("more"));
  return el("div", { ...base, class: "row", role: "row" },
    el("span", { class: "box", onclick: (e) => { e.stopPropagation(); select(i, { ctrlKey: true }); } }, icon("check")),
    el("span", { class: "name", title: item.n }, stem, tail ? el("em", {}, tail) : null, gitBadge(item), sub ? el("small", {}, sub) : null),
    el("span", { class: "tag" }, el("i", { style: "background:" + KIND_COLOR[k] }), KIND_LABEL[k]),
    el("span", { class: "meta" }, item.d ? "—" : formatSize(item.s)),
    el("span", { class: "meta date" }, formatDate(item.m)), dots);
}
function renderItems(error) {
  const box = $("items");
  box.className = "items" + (state.view === "grid" ? " grid" : "");
  if (!state.shown.length) {
    box.replaceChildren(emptyState(error));
    return;
  }
  const wrap = el("div", { class: "wrap" }, ...state.shown.map(rowFor));
  box.replaceChildren(wrap);
}
function emptyState(error) {
  const virtualText = { recent: "Aucun fichier récent dans cette racine.", favorites: "Aucun favori : ajoutez un dossier aux signets de votre gestionnaire de fichiers.",
    network: "Aucun partage réseau monté dans cette racine." };
  const text = error || (state.virtual && virtualText[state.virtual]) || (state.query.trim() ? "Aucun nom ne contient « " + state.query.trim() + " »."
    : state.items.some((i) => !i.d) ? "Aucun fichier de ce type dans ce dossier." : state.items.length ? "Ce dossier ne contient que des sous-dossiers." : "Ce dossier est vide.");
  const action = state.virtual ? null : state.query.trim() || state.filter !== "all"
    ? el("button", { class: "btn", type: "button", onclick: clearFilters }, "Effacer les filtres")
    : state.path ? el("button", { class: "btn", type: "button", onclick: goUp }, "Dossier parent") : null;
  return el("div", { class: "empty" }, icon("folder"), el("div", { class: "mono" }, error ? "Lecture impossible" : "Aucun fichier"),
    el("p", {}, text), action);
}

function pickedItems() { return state.shown.filter((i) => state.picked.has(i.p)); }
function select(i, ev) {
  state.sel = i;
  const item = state.shown[i];
  if (!item) state.picked.clear();
  else if (ev && (ev.ctrlKey || ev.metaKey)) {
    if (state.picked.has(item.p)) state.picked.delete(item.p); else state.picked.add(item.p);
  } else if (ev && ev.shiftKey && state.anchor >= 0) {
    const [a, b] = [Math.min(state.anchor, i), Math.max(state.anchor, i)];
    state.picked = new Set(state.shown.slice(a, b + 1).map((x) => x.p));
  } else state.picked = new Set([item.p]);
  if (!(ev && ev.shiftKey)) state.anchor = i;
  paintSelection();
  const node = document.querySelector(`#items [data-i="${i}"]`);
  if (node) node.scrollIntoView({ block: "nearest" });
  showPreview();
  if (state.picked.size) $("work").classList.add("open");
}
function paintSelection() {
  document.querySelectorAll("#items [data-i]").forEach((n) =>
    n.setAttribute("aria-selected", String(state.picked.has(state.shown[+n.dataset.i]?.p))));
}
function selectAll() {
  state.picked = new Set(state.shown.map((i) => i.p));
  state.sel = state.shown.length ? 0 : -1;
  paintSelection();
  showPreview();
}
function open(i) { openItem(state.shown[i]); }
function openItem(item) {
  if (!item) return;
  if (item.d) { state.query = ""; $("search").value = ""; go(item.p); }
  else if (S.newTab) window.open(rawUrl(item.p), "_blank", "noopener");
  else location.href = rawUrl(item.p);
}
function go(path) {
  $("work").classList.remove("open");
  $("side").classList.remove("open");
  $("scrim").classList.remove("on");
  load(path);
}
function goView(v) {
  state.query = "";
  $("search").value = "";
  $("work").classList.remove("open");
  $("side").classList.remove("open");
  $("scrim").classList.remove("on");
  load("", v);
}
function goUp() { state.query = ""; $("search").value = ""; go(parentOf(state.path)); }
function clearFilters() { state.filter = "all"; state.query = ""; $("search").value = ""; load(state.path); }

function fact(label, value) { return [el("dt", {}, label), el("dd", {}, value)]; }
function gaugeArc(ratio) {
  const r = 100, cx = 120, cy = 118;
  const end = Math.PI * (1 - Math.min(Math.max(ratio, 0), 1));
  const x = cx + r * Math.cos(end), y = cy - r * Math.sin(end);
  return `M${cx - r} ${cy} A${r} ${r} 0 0 1 ${x.toFixed(2)} ${y.toFixed(2)}`;
}
function spacePanel() {
  const sizes = { image: 0, video: 0, audio: 0, doc: 0, rest: 0 };
  state.items.filter((i) => !i.d).forEach((i) => { const k = kindOf(i); sizes[k in sizes ? k : "rest"] += i.s; });
  const direct = Object.values(sizes).reduce((a, b) => a + b, 0);
  const u = state.usage && state.usage.path === state.path && state.usage.data;
  const here = u ? u.total : direct;
  const folders = state.items.filter((i) => i.d).length;
  const files = state.items.length - folders;
  const gauge = el("div", { class: "gauge" });
  gauge.innerHTML = `<svg viewBox="0 0 240 124" aria-hidden="true"><path class="track" d="M20 118A100 100 0 0 1 220 118" fill="none" stroke-width="26"/><path class="used" d="${gaugeArc(here / (state.total || 1))}" fill="none" stroke-width="26"/></svg>`;
  const caption = u ? "avec les sous-dossiers" : files + " fichier" + (files > 1 ? "s" : "") + " · " + folders + " dossier" + (folders > 1 ? "s" : "");
  gauge.append(el("div", { class: "read" }, el("b", {}, formatSize(here)), el("span", {}, caption)));
  const disk = el("div", { class: "disk-line" }, "Disque : " + formatSize(state.free) + " libres sur " + formatSize(state.total));
  const rows = [["image", "Images", "image"], ["video", "Vidéos", "video"], ["audio", "Audio", "audio"],
    ["doc", "Documents", "text"], ["rest", "Autres", "file"]];
  const kinds = el("div", { class: "kinds" }, el("div", { class: "kinds-title" }, "Dans ce dossier"),
    ...rows.map(([k, label, ic]) => el("div", { class: "kind" }, el("span", { class: "glyph" }, icon(ic)),
      el("span", {}, label), el("small", {}, formatSize(sizes[k])))));
  return [el("div", { class: "space-head" }, el("h2", {}, "Espace")), gauge, disk, ...spaceExtras(kinds)];
}
function showPreview() {
  const body = $("workBody");
  const chosen = pickedItems();
  if (chosen.length > 1) { body.replaceChildren(...batchPanel(chosen)); return; }
  const item = chosen[0];
  if (!item) { body.replaceChildren(...spacePanel()); return; }
  const k = kindOf(item);
  const date = new Date(item.m * 1000);
  const facts = el("dl", { class: "facts sheet" }, ...fact("Emplacement", "/" + item.p),
    ...fact("Type", item.d ? "Dossier" : item.t || ext(item.n).toUpperCase() || "Fichier"),
    ...(item.d ? [] : fact("Taille", formatSize(item.s) + " (" + item.s.toLocaleString("fr") + " octets)")),
    ...fact("Modifié", MONTHS.format(date) + " à " + CLOCK.format(date)),
    ...(item.l ? fact("Nature", "Lien symbolique") : []));
  const actions = el("div", { class: "actions" },
    item.d ? el("button", { class: "btn go", type: "button", onclick: () => open(state.sel) }, icon("open"), "Ouvrir le dossier")
      : el("a", { class: "btn go", href: rawUrl(item.p, true) }, icon("down"), "Télécharger"),
    el("button", { class: "btn", type: "button", onclick: () => copyPath(item) }, icon("copy"), "Copier le chemin"),
    el("button", { class: "btn", type: "button", onclick: () => renameItems([item]) }, icon("pen"), "Renommer"),
    el("button", { class: "btn", type: "button", onclick: () => deleteItems([item]) }, icon("trash2"),
      inTrash(item.p) ? "Supprimer définitivement" : "Mettre à la corbeille"));
  const box = el("div", { class: "preview none", title: "Aperçu rapide : Espace", onclick: () => quickLook() });
  body.replaceChildren(el("h2", {}, item.n), box, facts, actions);
  fillPreview(box, item, k, facts);
}
async function copyPath(item) {
  const full = state.root.replace(/\/$/, "") + "/" + item.p;
  try { await navigator.clipboard.writeText(full); toast("Chemin copié"); }
  catch { toast("Copie refusée par le navigateur"); }
}
async function fillPreview(box, item, k, facts) {
  const stamp = state.sel + ":" + item.p;
  const none = (t) => box.replaceChildren(icon(ICON_BY_KIND[k]), t);
  if (item.d) return none("Dossier");
  const url = rawUrl(item.p);
  const media = { image: () => el("img", { src: url, alt: item.n, onload: (e) => addFacts(facts, [["Dimensions", e.target.naturalWidth + " × " + e.target.naturalHeight + " px"]]) }),
    video: () => el("video", { src: url, controls: "", onloadedmetadata: (e) => addFacts(facts, [["Durée", clock(e.target.duration)],
      ["Dimensions", e.target.videoWidth + " × " + e.target.videoHeight + " px"]]) }),
    audio: () => el("audio", { src: url, controls: "", onloadedmetadata: (e) => addFacts(facts, [["Durée", clock(e.target.duration)]]) }) }[k];
  if (media) { box.className = "preview"; return box.replaceChildren(media()); }
  if (isArchive(item.n)) return fillArchive(box, item, none);
  if (ext(item.n) === "pdf") { box.className = "preview"; return box.replaceChildren(el("iframe", { src: url, title: item.n })); }
  if (item.s > 50e6) return none("Aperçu indisponible : fichier volumineux");
  try {
    const data = await api("text", { p: item.p });
    if (stamp !== state.sel + ":" + item.p) return;
    if (data.binary) return none("Aperçu indisponible : fichier binaire");
    box.className = "preview text";
    box.replaceChildren(el("pre", {}, data.text + (data.truncated ? "\n…" : "")));
  } catch (e) { none(e.message); }
}

function openMenu(button, item, i) {
  if (i !== undefined && !state.picked.has(item.p)) select(i);
  const targets = state.picked.has(item.p) && state.picked.size > 1 ? pickedItems() : [item];
  const final = targets.every((t) => inTrash(t.p));
  const single = targets.length === 1;
  const entries = [single ? { icon: item.d ? "open" : "file", label: item.d ? "Ouvrir le dossier" : "Ouvrir dans un onglet", run: () => openItem(item) } : null,
    single && !item.d ? { icon: "down", label: "Télécharger", href: rawUrl(item.p, true) } : null,
    single && !item.d ? { icon: "eye", label: "Aperçu rapide", hint: "Espace", run: () => quickLook(item) } : null,
    single ? { icon: "copy", label: "Copier le chemin", run: () => copyPath(item) } : null,
    { icon: "copy", label: "Copier", hint: "Ctrl C", run: () => setClip("copy", targets) },
    { icon: "cut", label: "Couper", hint: "Ctrl X", run: () => setClip("move", targets) },
    { icon: "pen", label: "Renommer", hint: "F2", run: () => renameItems(targets) },
    { icon: "trash2", label: final ? "Supprimer définitivement" : "Mettre à la corbeille", run: () => deleteItems(targets) }];
  popover(button, entries.filter(Boolean), item.p);
}
function popover(button, entries, key) {
  const pop = $("menuPop");
  if (!pop.hidden && pop.dataset.p === key) return closeMenu();
  closeMenu();
  const nodes = entries.map((e) => {
    const inner = [icon(e.icon), e.label, e.hint ? el("small", { class: "hint" }, e.hint) : null];
    if (e.href) return el("a", { role: "menuitem", href: e.href, onclick: closeMenu }, ...inner);
    return el("button", { type: "button", role: "menuitem", onclick: () => { closeMenu(); e.run(); } }, ...inner);
  });
  pop.replaceChildren(...nodes);
  pop.dataset.p = key;
  pop.hidden = false;
  const r = button.getBoundingClientRect();
  const h = pop.offsetHeight;
  const top = r.bottom + 6 + h > innerHeight - 8 ? r.top - 6 - h : r.bottom + 6;
  pop.style.top = Math.max(8, top) + "px";
  pop.style.left = Math.max(8, Math.min(r.left, innerWidth - pop.offsetWidth - 8)) + "px";
  button.setAttribute("aria-expanded", "true");
}
function closeMenu() {
  $("menuPop").hidden = true;
  document.querySelectorAll("[aria-expanded]").forEach((b) => b.removeAttribute("aria-expanded"));
}
document.addEventListener("click", (e) => { if (!e.target.closest(".menu, .dots, .sep")) closeMenu(); });
function move(delta) {
  if (!state.shown.length) return;
  select(Math.min(Math.max(state.sel + delta, 0), state.shown.length - 1));
}
function typing(e) { return e.target.matches("input, textarea"); }
document.addEventListener("keydown", (e) => {
  if (document.querySelector(".overlay")) return;
  if (state.settings && e.key === "Escape") { closeSettings(); return; }
  if (e.key === "Escape") {
    if (!$("menuPop").hidden) { closeMenu(); return; }
    if ($("search").value) { $("search").value = ""; state.query = ""; load(state.path); }
    else if ($("work").classList.contains("open")) $("work").classList.remove("open");
    else if (state.sel >= 0) select(-1);
    document.activeElement.blur?.();
    return;
  }
  const mod = (e.ctrlKey || e.metaKey) && !e.altKey;
  const low = e.key.toLowerCase();
  if (mod && (low === "p" || low === "k")) { e.preventDefault(); openPalette(); return; }
  const clip = { a: selectAll, c: () => setClip("copy", pickedItems()), x: () => setClip("move", pickedItems()), v: pasteClip };
  if (mod && !typing(e) && !state.settings && clip[low]) { e.preventDefault(); clip[low](); return; }
  if (typing(e) || e.ctrlKey || e.metaKey || e.altKey) return;
  if (state.settings && e.key !== ",") return;
  const actions = { "/": () => $("search").focus(), j: () => move(1), ArrowDown: () => move(1), k: () => move(-1),
    ArrowUp: () => move(-1), Enter: () => open(state.sel), Backspace: goUp, h: goUp, ".": toggleHidden, c: createFolder, ",": toggleSettings,
    Delete: () => deleteItems(pickedItems()), "#": () => deleteItems(pickedItems()), " ": () => quickLook(), F2: () => renameItems(pickedItems()) };
  if (actions[e.key]) { e.preventDefault(); actions[e.key](); }
});

function toggleHidden() {
  setSetting("hidden", !S.hidden);
  toast(S.hidden ? "Fichiers cachés affichés" : "Fichiers cachés masqués");
}
function setPressed(group, attr, value) {
  group.querySelectorAll("button").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset[attr] === value)));
}
function saveSettings() {
  try { localStorage.setItem(STORE, JSON.stringify(S)); } catch {}
}
function loadSettings() {
  try { Object.assign(S, JSON.parse(localStorage.getItem(STORE) || "{}")); } catch {}
  Object.assign(state, { view: S.view, sort: S.sort, asc: S.asc, hidden: S.hidden });
  applySetting("theme", true);
  applySetting("dates", true);
  applySetting("accent", true);
  applySetting("density", true);
  setPressed($("view"), "v", state.view);
  $("hidden").setAttribute("aria-pressed", String(S.hidden));
}
const HEX = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i;
const SWATCHES = ["#e8542b", "#e0434b", "#d6407a", "#8b5ce6", "#3b7bea", "#1fb5c9", "#22a667", "#e0a60e"];
function luminance(hex) {
  const full = hex.length === 4 ? "#" + [...hex.slice(1)].map((c) => c + c).join("") : hex;
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(full.slice(i, i + 2), 16) / 255)
    .map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}
function applyAccent(root) {
  if (!HEX.test(S.accent) || S.accent.toLowerCase() === SWATCHES[0]) {
    root.style.removeProperty("--ua");
    root.style.removeProperty("--on-accent");
    return;
  }
  const l = luminance(S.accent);
  root.style.setProperty("--ua", S.accent);
  root.style.setProperty("--on-accent", (l + 0.05) / 0.061 > 1.05 / (l + 0.05) ? "#1b1b1b" : "#ffffff");
}
function setSetting(key, value) {
  S[key] = value;
  saveSettings();
  applySetting(key);
}
function applySetting(key, quiet) {
  const root = document.documentElement;
  if (key === "theme") {
    if (S.theme === "system") delete root.dataset.theme; else root.dataset.theme = S.theme;
  } else if (key === "dates") {
    root.style.setProperty("--date-col", S.dates === "full" ? "150px" : "104px");
  } else if (key === "accent") {
    applyAccent(root);
    return;
  } else if (key === "density") {
    root.dataset.density = S.density;
    return;
  } else if (key === "hidden") {
    state.hidden = S.hidden;
    $("hidden").setAttribute("aria-pressed", String(S.hidden));
    reload();
    return;
  } else if (key === "view" || key === "sort" || key === "asc") {
    Object.assign(state, { view: S.view, sort: S.sort, asc: S.asc });
    setPressed($("view"), "v", state.view);
  }
  if (quiet) return;
  state.sel = -1;
  render();
  showPreview();
}

const TABS = [["display", "Affichage"], ["files", "Fichiers"], ["keys", "Raccourcis"], ["server", "Serveur"]];
const SHORTCUTS = [["/", "Chercher"], ["j  k", "Descendre / monter dans la liste (ou les flèches)"], ["Entrée", "Ouvrir l'élément sélectionné"],
  ["Retour arrière  h", "Dossier parent"], ["c", "Nouveau dossier"], ["Suppr  #", "Mettre à la corbeille"],
  ["Espace", "Aperçu rapide"], ["Ctrl P  Ctrl K", "Palette de commandes"], ["Ctrl A", "Tout sélectionner"],
  ["Ctrl C  Ctrl X  Ctrl V", "Copier, couper, coller"], ["F2", "Renommer (par lot si plusieurs éléments)"],
  ["Ctrl clic  Maj clic", "Sélection multiple"],
  [".", "Afficher ou masquer les fichiers cachés"], [",", "Ouvrir ou fermer les paramètres"], ["Échap", "Fermer l'élément au premier plan"]];
let tab = "display";

function setting(title, text, control, stack) {
  return el("div", { class: "setting" + (stack ? " stack" : "") }, el("div", { class: "txt" }, el("b", {}, title), text ? el("p", {}, text) : null),
    el("div", { class: "ctl" }, control));
}
function choice(key, options) {
  return el("div", { class: "seg", role: "group", "aria-label": key },
    ...options.map(([value, label]) => el("button", { type: "button", "aria-pressed": String(S[key] === value),
      onclick: () => { setSetting(key, value); renderSettings(); } }, label)));
}
function checkbox(key, label) {
  return el("button", { class: "tick", type: "button", role: "checkbox", "aria-checked": String(!!S[key]), "aria-label": label,
    onclick: () => { setSetting(key, !S[key]); renderSettings(); } }, el("i", {}, icon("check")));
}
function accentControl() {
  const current = (S.accent || SWATCHES[0]).toLowerCase();
  const input = el("input", { class: "hex", type: "text", placeholder: "#personnalisée", maxlength: "7", spellcheck: "false",
    "aria-label": "Couleur personnalisée", value: SWATCHES.includes(current) ? "" : S.accent });
  input.oninput = () => {
    const v = input.value.trim();
    input.setAttribute("aria-invalid", String(v !== "" && !HEX.test(v)));
    if (HEX.test(v)) { setSetting("accent", v.toLowerCase()); markSwatches(); }
  };
  const group = el("div", { class: "swatches", role: "radiogroup", "aria-label": "Couleur principale" },
    ...SWATCHES.map((color) => el("button", { class: "sw", type: "button", role: "radio", style: "background:" + color,
      "aria-checked": String(color === current), "aria-label": color,
      onclick: () => { setSetting("accent", color === SWATCHES[0] ? "" : color); input.value = ""; markSwatches(); } }, icon("check"))), input);
  return group;
}
function markSwatches() {
  const current = (S.accent || SWATCHES[0]).toLowerCase();
  document.querySelectorAll(".sw").forEach((b) => b.setAttribute("aria-checked", String(b.getAttribute("aria-label") === current)));
}
function displayPanel() {
  return [setting("Thème", "« Système » suit le réglage de votre ordinateur.",
      choice("theme", [["system", "Système"], ["light", "Clair"], ["dark", "Sombre"]])),
    setting("Couleur principale", "Remplace l'orange : boutons, accents, sélection et libellés actifs.", accentControl(), true),
    setting("Densité de la liste", "Confortable : lignes aérées. Compacte : plus de fichiers à l'écran.",
      choice("density", [["comfortable", "Confortable"], ["compact", "Compacte"]])),
    setting("Affichage des fichiers", "Disposition utilisée à chaque ouverture.", choice("view", [["list", "Liste"], ["grid", "Grille"]])),
    setting("Miniatures des images", "En grille, affiche l'image plutôt qu'une icône.", checkbox("thumbs", "Miniatures des images")),
    setting("Unité des tailles", "1 Ko = 1 000 octets, ou 1 Kio = 1 024 octets.", choice("units", [["si", "Ko"], ["iec", "Kio"]])),
    setting("Format des dates", "Courte : l'heure pour aujourd'hui, sinon la date.", choice("dates", [["short", "Courte"], ["full", "Complète"]]))];
}
function filesPanel() {
  return [setting("Afficher les fichiers cachés", "Ceux dont le nom commence par un point.", checkbox("hidden", "Afficher les fichiers cachés")),
    setting("Tri par défaut", "Les dossiers restent toujours avant les fichiers.",
      choice("sort", [["n", "Nom"], ["t", "Type"], ["s", "Taille"], ["m", "Date"]])),
    setting("Ordre du tri", null, choice("asc", [[true, "Croissant"], [false, "Décroissant"]])),
    setting("Indicateurs Git", "Affiche M, ?, A, D ou ! à côté des fichiers d'un dépôt Git.", checkbox("git", "Indicateurs Git")),
    setting("Reprendre où je me suis arrêté", "Rouvre le dernier dossier consulté au démarrage.", checkbox("restore", "Reprendre la session")),
    setting("Ouvrir les fichiers dans un nouvel onglet", "Sinon, l'onglet actuel quitte le gestionnaire.", checkbox("newTab", "Nouvel onglet")),
    setting("Confirmer la mise à la corbeille", "La suppression définitive demande toujours une confirmation.",
      checkbox("confirmTrash", "Confirmer la mise à la corbeille"))];
}
function keysPanel() {
  return [el("dl", { class: "keys" }, ...SHORTCUTS.flatMap(([keys, text]) =>
    [el("dt", {}, ...keys.split("  ").map((k) => el("kbd", {}, k))), el("dd", {}, text)]))];
}
function serverPanel() {
  const holder = el("div", {}, el("div", { class: "setting" }, "Lecture des informations…"));
  api("info", {}).then((info) => holder.replaceChildren(
    el("dl", { class: "facts" }, ...fact("Racine exposée", info.root), ...fact("Adresse", "http://127.0.0.1:" + info.port),
      ...fact("Accès", "Cette machine uniquement. Lecture, création de dossiers et mise à la corbeille, dans la racine."),
      ...fact("Corbeille", "/" + info.trash), ...fact("Python", info.python)),
    setting("Changer de racine", "Relancez le serveur avec --root /chemin (par exemple --root / pour tout le disque).", el("span")),
    setting("Terminer la session", "Efface le cookie de ce navigateur. Rouvrez l'adresse du terminal pour revenir.",
      el("button", { class: "btn", type: "button", onclick: endSession }, "Terminer"))))
    .catch((e) => holder.replaceChildren(setting("Informations indisponibles", e.message, el("span"))));
  return [holder];
}
async function endSession() {
  try { await post("logout", {}); } catch (e) { return toast(e.message); }
  document.body.replaceChildren(el("div", { class: "empty", style: "height:100vh" }, icon("file"),
    el("div", { class: "mono" }, "Session terminée"), el("p", {}, "Rouvrez l'adresse affichée dans le terminal pour reprendre.")));
}
function renderSettings() {
  const panels = { display: displayPanel, files: filesPanel, keys: keysPanel, server: serverPanel };
  $("settings").replaceChildren(
    el("div", { class: "settings-top" }, el("h1", {}, "Paramètres"),
      el("button", { class: "btn icon", type: "button", onclick: closeSettings, "aria-label": "Fermer les paramètres",
        title: "Fermer ( Échap )" }, icon("x"))),
    el("div", { class: "tabs", role: "tablist" }, ...TABS.map(([id, label]) =>
      el("button", { type: "button", role: "tab", "aria-selected": String(tab === id), onclick: () => { tab = id; renderSettings(); } }, label))),
    el("div", { class: "group-card" }, ...panels[tab]()));
}
function openSettings() {
  state.settings = true;
  $("settings").hidden = false;
  $("content").hidden = true;
  $("gear").setAttribute("aria-pressed", "true");
  renderSettings();
}
function closeSettings() {
  state.settings = false;
  $("settings").hidden = true;
  $("content").hidden = false;
  $("gear").setAttribute("aria-pressed", "false");
}
function toggleSettings() { if (state.settings) closeSettings(); else openSettings(); }
function wire() {
  $("up").onclick = goUp;
  $("newFolder").onclick = createFolder;
  $("hidden").onclick = toggleHidden;
  $("close").onclick = () => $("work").classList.remove("open");
  $("menu").onclick = () => { $("side").classList.add("open"); $("scrim").classList.add("on"); };
  $("scrim").onclick = () => { $("side").classList.remove("open"); $("scrim").classList.remove("on"); };
  $("gear").onclick = toggleSettings;
  $("theme").onclick = () => {
    const dark = getComputedStyle(document.body).backgroundColor === "rgb(27, 27, 27)";
    setSetting("theme", dark ? "light" : "dark");
    if (state.settings) renderSettings();
  };
  $("view").onclick = (e) => {
    const b = e.target.closest("button");
    if (!b) return;
    setSetting("view", b.dataset.v);
  };
  let timer;
  $("search").oninput = (e) => {
    clearTimeout(timer);
    timer = setTimeout(() => { state.query = e.target.value; load(state.path); }, 250);
  };
}

async function start() {
  loadSettings();
  wire();
  try {
    const data = await api("places", {});
    state.places = data.places;
    state.root = data.root;
    state.user = data.user;
    await load("");
  } catch (e) {
    $("items").replaceChildren(emptyState(e.message));
  }
}
