"use strict";

const SESSION = "five-session";
const GIT_LABEL = { M: "Modifié", A: "Ajouté", D: "Supprimé", "?": "Non suivi", "!": "Ignoré", U: "Conflit" };

function clock(sec) {
  if (!Number.isFinite(sec)) return "—";
  const s = Math.round(sec);
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), r = s % 60;
  return (h ? h + ":" + String(m).padStart(2, "0") : m) + ":" + String(r).padStart(2, "0");
}
function addFacts(facts, rows) {
  if (!facts) return;
  const known = [...facts.querySelectorAll("dt")].map((d) => d.textContent);
  rows.filter(([label]) => !known.includes(label)).forEach(([label, value]) => facts.append(...fact(label, value)));
}
function modal(title, wide) {
  const previous = document.activeElement;
  const box = el("div", { class: "dialog" + (wide ? " wide" : ""), role: "dialog", "aria-modal": "true", "aria-label": title }, el("h2", {}, title));
  const overlay = el("div", { class: "overlay", tabindex: "-1" }, box);
  const close = () => { overlay.remove(); previous?.focus?.(); };
  overlay.addEventListener("mousedown", (e) => { if (e.target === overlay) close(); });
  overlay.addEventListener("keydown", (e) => { if (e.key === "Escape") { e.stopPropagation(); close(); } });
  document.body.append(overlay);
  overlay.focus();
  return { box, close, overlay };
}
const button = (label, cls, onclick, ic) => el("button", { class: "btn " + (cls || ""), type: "button", onclick }, ic ? icon(ic) : null, label);

/* ---------- Git, session, sélection différée ---------- */
function gitBadge(item) {
  const code = state.gitPath === state.path ? state.git[item.n] : null;
  return code ? el("span", { class: "git", "data-g": code, title: GIT_LABEL[code] || code }, code) : null;
}
async function loadGit() {
  state.git = {};
  state.gitPath = null;
  if (!S.git || state.virtual || searching()) return;
  const path = state.path;
  try {
    const data = await api("git", { p: path });
    if (path !== state.path || !Object.keys(data.marks).length) return;
    state.git = data.marks;
    state.gitPath = path;
    render();
  } catch { /* pas de dépôt : rien à afficher */ }
}
function afterLoad() {
  if (state.pending) {
    const i = state.shown.findIndex((x) => x.p === state.pending);
    state.pending = null;
    if (i >= 0) select(i);
  }
  try { localStorage.setItem(SESSION, JSON.stringify({ path: state.path, virtual: state.virtual })); } catch {}
  loadGit();
}

/* ---------- fil d'Ariane déroulant ---------- */
async function crumbMenu(btn, parent) {
  try {
    const data = await api("list", { p: parent, h: state.hidden ? "1" : "0" });
    const dirs = data.items.filter((i) => i.d).sort((a, b) => a.n.localeCompare(b.n, "fr", { sensitivity: "base" })).slice(0, 40);
    const entries = dirs.map((d) => ({ icon: "folder", label: d.n, run: () => go(d.p) }));
    popover(btn, entries.length ? entries : [{ icon: "folder", label: "Aucun sous-dossier", run: () => {} }], "crumb:" + parent);
  } catch (e) { toast(e.message); }
}

/* ---------- archives ---------- */
const isArchive = (n) => /\.(zip|jar|tar|tgz|txz|tbz2|tar\.gz|tar\.xz|tar\.bz2|tar\.zst)$/i.test(n);
async function fillArchive(box, item, none) {
  try {
    const data = await api("archive", { p: item.p });
    box.className = "preview text";
    const lines = data.items.map((r) => (r.d ? "".padStart(9) : formatSize(r.s).padStart(9)) + "  " + r.n);
    box.replaceChildren(el("pre", {}, lines.join("\n") + (data.more ? "\n…" : "")));
  } catch (e) { none(e.message); }
}

/* ---------- panneau Espace : occupation ---------- */
function spaceExtras(kinds) {
  const off = !!state.virtual;
  const tools = el("div", { class: "tools" }, button("Analyser l'occupation", "", analyseUsage, "archive"),
    button("Chercher les doublons", "", findDuplicates, "copy"));
  tools.querySelectorAll("button").forEach((b) => { b.disabled = off; });
  const u = state.usage;
  return [tools, u && u.path === state.path ? usageList(u) : kinds];
}
async function analyseUsage() {
  if (state.virtual) return;
  const path = state.path;
  state.usage = { path, loading: true };
  showPreview();
  try {
    state.usage = { path, data: await api("usage", { p: path }) };
  } catch (e) { state.usage = null; toast(e.message); }
  if (state.path === path) showPreview();
}
function usageList(u) {
  if (u.loading) return el("div", { class: "kinds-title" }, "Analyse en cours…");
  const max = u.data.items[0]?.s || 1;
  const rows = u.data.items.slice(0, 14).map((r) => el("button", { class: "use", type: "button", disabled: !r.d,
    onclick: () => go(r.p), title: r.n }, icon(r.d ? "folder" : "file"),
    el("span", { class: "n" }, r.n), el("small", {}, formatSize(r.s)),
    el("i", { class: "bar", style: `width:${Math.max(2, (r.s / max) * 100)}%` })));
  return el("div", { class: "kinds" }, el("div", { class: "kinds-title" },
    "Occupation : " + formatSize(u.data.total) + (u.data.partial ? " (partielle)" : "")), ...rows,
  button("Revenir aux catégories", "", () => { state.usage = null; showPreview(); }));
}

/* ---------- doublons ---------- */
async function findDuplicates() {
  if (state.virtual) return;
  const m = modal("Doublons dans /" + state.path, true);
  m.box.append(el("p", {}, "Analyse en cours : comparaison des tailles puis des empreintes SHA-256…"));
  try {
    const data = await api("duplicates", { p: state.path });
    renderDuplicates(m, data);
  } catch (e) { m.box.replaceChildren(el("h2", {}, "Doublons"), el("p", {}, e.message)); }
}
function renderDuplicates(m, data) {
  const marked = new Set();
  data.groups.forEach((g) => g.files.slice(1).forEach((f) => marked.add(f)));
  const summary = el("p", {});
  const confirm = button("", "go", async () => {
    confirm.disabled = true;
    try { await post("delete", { paths: [...marked] }); toast("Mis à la corbeille"); m.close(); reload(); }
    catch (e) { summary.textContent = e.message; confirm.disabled = false; }
  }, "trash2");
  const refresh = () => {
    const freed = data.groups.reduce((n, g) => n + g.size * g.files.filter((f) => marked.has(f)).length, 0);
    summary.textContent = data.groups.length
      ? `${data.groups.length} groupe(s). Gaspillage : ${formatSize(data.wasted)}. Le premier fichier de chaque groupe est conservé.` + (data.partial ? " Analyse partielle." : "")
      : "Aucun doublon trouvé dans ce dossier." + (data.partial ? " Analyse partielle." : "");
    confirm.lastChild.textContent = `Mettre ${marked.size} fichier(s) à la corbeille (${formatSize(freed)})`;
    confirm.disabled = !marked.size;
  };
  const list = el("div", { class: "dups" }, ...data.groups.map((g) => el("div", { class: "dup" },
    el("div", { class: "kinds-title" }, `${g.files.length} copies · ${formatSize(g.size)} chacune`),
    ...g.files.map((f) => {
      const tick = el("button", { class: "tick", type: "button", role: "checkbox", "aria-checked": String(marked.has(f)),
        "aria-label": f, onclick: () => {
          if (marked.has(f)) marked.delete(f); else marked.add(f);
          tick.setAttribute("aria-checked", String(marked.has(f)));
          refresh();
        } }, el("i", {}, icon("check")));
      return el("div", { class: "dup-row" }, tick, el("span", { class: "mono-path" }, "/" + f));
    }))));
  m.box.replaceChildren(el("h2", {}, "Doublons"), summary, data.groups.length ? list : null,
    el("div", { class: "dialog-actions" }, button("Fermer", "", m.close), data.groups.length ? confirm : null));
  refresh();
}

/* ---------- sélection multiple ---------- */
function batchPanel(items) {
  const files = items.filter((i) => !i.d);
  const facts = el("dl", { class: "facts sheet" }, ...fact("Sélection", items.length + " éléments"),
    ...fact("Fichiers", String(files.length)), ...fact("Dossiers", String(items.length - files.length)),
    ...fact("Taille", formatSize(files.reduce((n, i) => n + i.s, 0))));
  const final = items.every((i) => inTrash(i.p));
  const actions = el("div", { class: "actions" }, button("Renommer", "go", () => renameItems(items), "pen"),
    button("Copier", "", () => setClip("copy", items), "copy"), button("Couper", "", () => setClip("move", items), "cut"),
    button(final ? "Supprimer définitivement" : "Mettre à la corbeille", "", () => deleteItems(items), "trash2"));
  return [el("h2", {}, items.length + " éléments"), facts, actions];
}

/* ---------- renommage ---------- */
function renameItems(items) {
  if (!items.length) return toast("Sélectionnez au moins un élément");
  if (items.some((i) => inTrash(i.p))) return toast("Impossible de renommer dans la corbeille");
  if (items.length > 1) return batchRename(items);
  const item = items[0];
  dialog({ title: "Renommer", text: "Dans /" + parentOf(item.p), input: item.n, confirm: "Renommer",
    submit: async (name) => { await post("rename", { items: [{ p: item.p, name }] }); toast("Renommé"); reload(); } });
}
function newNameFor(m, item, index) {
  if (m.mode === "template") {
    const [stem, tail] = nameParts(item.n);
    const n = String(m.start + index).padStart(m.pad, "0");
    return m.tpl.split("{name}").join(stem).split("{ext}").join(tail).split("{n}").join(n);
  }
  if (!m.find) return item.n;
  const source = m.regex ? m.find : m.find.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return item.n.replace(new RegExp(source, m.icase ? "gi" : "g"), m.rep);
}
function checkNames(items, names) {
  const own = new Set(items.map((i) => i.n));
  const others = new Set(state.items.map((i) => i.n).filter((n) => !own.has(n)));
  return names.map((n, i) => {
    if (!n.trim() || n.includes("/") || n === "." || n === "..") return "Nom invalide";
    if (names.indexOf(n) !== i && n !== items[i].n) return "Doublon dans le lot";
    if (others.has(n)) return "Existe déjà";
    return "";
  });
}
function batchRename(items) {
  const m = { mode: "replace", find: "", rep: "", regex: false, icase: true, tpl: "{name}-{n}{ext}", start: 1, pad: 2 };
  const win = modal("Renommer " + items.length + " éléments", true);
  const fields = el("div", { class: "rn-fields" });
  const list = el("div", { class: "rn-list" });
  const note = el("div", { class: "form-error mono", role: "alert" });
  const ok = button("Renommer", "go", () => apply());
  const field = (label, key, type = "text") => {
    const input = el("input", { class: "field", type, value: String(m[key]), "aria-label": label, placeholder: label, spellcheck: "false" });
    input.oninput = () => { m[key] = type === "number" ? Math.max(0, +input.value || 0) : input.value; refresh(); };
    return input;
  };
  const toggle = (label, key) => {
    const box = el("input", { type: "checkbox", onchange: (e) => { m[key] = e.target.checked; refresh(); } });
    box.checked = !!m[key];
    return el("label", { class: "rn-check" }, box, label);
  };
  const drawFields = () => {
    const modes = el("div", { class: "seg" }, ...[["replace", "Remplacer"], ["template", "Modèle"]].map(([id, label]) =>
      el("button", { type: "button", "aria-pressed": String(m.mode === id), onclick: () => { m.mode = id; drawFields(); refresh(); } }, label)));
    const body = m.mode === "replace"
      ? [field("Rechercher", "find"), field("Remplacer par", "rep"), el("div", { class: "rn-row" }, toggle("Expression régulière", "regex"), toggle("Ignorer la casse", "icase"))]
      : [field("Modèle : {name} {n} {ext}", "tpl"), el("div", { class: "rn-row" }, field("Début", "start", "number"), field("Chiffres", "pad", "number"))];
    fields.replaceChildren(modes, ...body);
  };
  let plan = [];
  const refresh = () => {
    let names;
    try { names = items.map((it, i) => newNameFor(m, it, i)); note.textContent = ""; }
    catch (e) { note.textContent = "Expression invalide : " + e.message; names = items.map((i) => i.n); }
    const errors = checkNames(items, names);
    plan = items.map((it, i) => ({ it, name: names[i], err: errors[i] })).filter((p) => p.name !== p.it.n || p.err);
    list.replaceChildren(...items.map((it, i) => el("div", { class: "rn-line" + (errors[i] ? " bad" : "") },
      el("span", {}, it.n), icon("back"), el("b", {}, names[i]), errors[i] ? el("small", {}, errors[i]) : null)));
    ok.disabled = !plan.length || errors.some(Boolean) || !!note.textContent;
    ok.lastChild.textContent = plan.length ? `Renommer ${plan.length} élément(s)` : "Aucun changement";
  };
  const apply = async () => {
    ok.disabled = true;
    try {
      await post("rename", { items: plan.map((p) => ({ p: p.it.p, name: p.name })) });
      toast("Renommé"); win.close(); reload();
    } catch (e) { note.textContent = e.message; ok.disabled = false; }
  };
  win.box.append(fields, list, note, el("div", { class: "dialog-actions" }, button("Annuler", "", win.close), ok));
  drawFields();
  refresh();
}

/* ---------- presse-papiers et transferts ---------- */
function setClip(mode, items) {
  if (!items.length) return toast("Sélectionnez au moins un élément");
  state.clip = { mode, paths: items.map((i) => i.p) };
  toast(items.length + (mode === "copy" ? " copié(s)" : " coupé(s)"));
  updatePaste();
}
function updatePaste() {
  const b = $("paste");
  if (!b) return;
  b.hidden = !state.clip || !canCreate();
  if (state.clip) b.lastChild.textContent = "Coller (" + state.clip.paths.length + ")";
}
async function pasteClip() {
  if (!state.clip) return toast("Rien à coller : copiez ou coupez d'abord");
  if (!canCreate()) return toast("Ouvrez un dossier ordinaire pour y coller");
  try {
    const started = await post("job", { kind: state.clip.mode, paths: state.clip.paths, dest: state.path });
    jobs.seen.set(started.id, "running");
    if (state.clip.mode === "move") state.clip = null;
    updatePaste();
    pollJobs();
  } catch (e) { toast(e.message); }
}
const jobs = { seen: new Map(), hidden: new Set(), timer: null };
async function pollJobs() {
  clearTimeout(jobs.timer);
  let list = [];
  try { list = (await api("jobs", {})).jobs; } catch { return; }
  if (!jobs.started) {
    jobs.started = true;
    list.filter((j) => j.state !== "running" && j.state !== "paused").forEach((j) => jobs.hidden.add(j.id));
  }
  list.forEach((j) => {
    const before = jobs.seen.get(j.id);
    if (before && before !== j.state && ["done", "cancelled", "error"].includes(j.state)) reload();
    jobs.seen.set(j.id, j.state);
    if (!["running", "paused"].includes(j.state) && !jobs.fading?.has(j.id)) {
      (jobs.fading ||= new Set()).add(j.id);
      setTimeout(() => { jobs.hidden.add(j.id); pollJobs(); }, 6000);
    }
  });
  drawJobs(list.filter((j) => !jobs.hidden.has(j.id)));
  if (list.some((j) => j.state === "running" || j.state === "paused")) jobs.timer = setTimeout(pollJobs, 500);
}
const JOB_STATE = { running: "En cours", paused: "En pause", done: "Terminé", cancelled: "Annulé", error: "Erreur" };
function drawJobs(list) {
  let box = $("transfers");
  if (!box) { box = el("div", { id: "transfers", class: "transfers", "aria-label": "Transferts" }); document.body.append(box); }
  box.hidden = !list.length;
  box.replaceChildren(...list.map((j) => {
    const live = j.state === "running" || j.state === "paused";
    const pct = j.total ? Math.min(100, (j.done / j.total) * 100) : j.state === "done" ? 100 : 0;
    const act = live ? [button("", "icon", () => jobAct(j, j.state === "paused" ? "resume" : "pause"), j.state === "paused" ? "play" : "pause"),
      button("", "icon", () => jobAct(j, "cancel"), "x")] : [button("", "icon", () => { jobs.hidden.add(j.id); pollJobs(); }, "x")];
    act.forEach((b, i) => b.setAttribute("aria-label", live ? (i ? "Annuler" : j.state === "paused" ? "Reprendre" : "Mettre en pause") : "Masquer"));
    return el("div", { class: "job" }, el("div", { class: "job-top" },
      el("b", {}, (j.kind === "copy" ? "Copie" : "Déplacement") + " de " + j.count + " élément(s)"), el("span", { class: "acts" }, ...act)),
    el("div", { class: "mono dim" }, JOB_STATE[j.state] + " · " + j.filesDone + "/" + j.files + " fichiers · vers /" + j.dest),
    el("div", { class: "track" }, el("i", { style: "width:" + pct + "%" })), j.error ? el("div", { class: "form-error" }, j.error) : null);
  }));
}
async function jobAct(j, action) {
  try { await post("job/control", { id: j.id, action }); } catch (e) { toast(e.message); }
  pollJobs();
}

/* ---------- aperçu rapide ---------- */
function quickLook(item) {
  if (document.querySelector(".overlay")) return;
  const list = state.shown;
  let idx = item ? list.indexOf(item) : list.findIndex((i) => state.picked.has(i.p));
  if (idx < 0) return toast("Sélectionnez un élément");
  const title = el("h2", {});
  const body = el("div", { class: "preview big none" });
  const close = el("button", { class: "btn icon", type: "button", "aria-label": "Fermer", onclick: () => end() }, icon("x"));
  const overlay = el("div", { class: "overlay", tabindex: "-1" },
    el("div", { class: "ql", role: "dialog", "aria-modal": "true", "aria-label": "Aperçu rapide" },
      el("div", { class: "ql-head" }, title, el("span", { class: "mono dim" }, "← → parcourir · Espace fermer"), close), body));
  const show = () => {
    const it = list[idx];
    title.textContent = it.n;
    body.className = "preview big none";
    fillPreview(body, it, kindOf(it), null);
  };
  const end = () => { overlay.remove(); select(idx); };
  overlay.addEventListener("keydown", (e) => {
    const move = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
    if (e.key === "Escape" || e.key === " ") { e.preventDefault(); e.stopPropagation(); end(); }
    else if (move) { e.preventDefault(); idx = Math.min(Math.max(idx + move, 0), list.length - 1); show(); }
  });
  overlay.addEventListener("mousedown", (e) => { if (e.target === overlay) end(); });
  document.body.append(overlay);
  overlay.focus();
  show();
}

/* ---------- palette de commandes ---------- */
function fuzzy(q, n) {
  q = q.toLowerCase(); n = n.toLowerCase();
  if (!q) return 0;
  if (n.includes(q)) return n.indexOf(q);
  let pos = -1, gaps = 0, first = null;
  for (const ch of q) {
    const x = n.indexOf(ch, pos + 1);
    if (x < 0) return null;
    if (first === null) first = x;
    if (pos >= 0) gaps += x - pos - 1;
    pos = x;
  }
  return 1000 + gaps * 4 + first;
}
function paletteCommands() {
  const base = [
    { icon: "plus", label: "Nouveau dossier", hint: "c", run: createFolder },
    { icon: "up", label: "Dossier parent", hint: "h", run: goUp },
    { icon: "paste", label: "Coller", hint: "Ctrl V", run: pasteClip },
    { icon: "pen", label: "Renommer la sélection", hint: "F2", run: () => renameItems(pickedItems()) },
    { icon: "gear", label: "Ouvrir les paramètres", hint: ",", run: toggleSettings },
    { icon: "sun", label: "Basculer le thème clair / sombre", run: () => $("theme").click() },
    { icon: "eye", label: "Afficher ou masquer les fichiers cachés", hint: ".", run: toggleHidden },
    { icon: "list", label: "Affichage en liste", run: () => setSetting("view", "list") },
    { icon: "grid", label: "Affichage en grille", run: () => setSetting("view", "grid") },
    { icon: "archive", label: "Analyser l'occupation de ce dossier", run: analyseUsage },
    { icon: "copy", label: "Chercher les doublons dans ce dossier", run: findDuplicates },
    { icon: "home", label: "Aller à : Dossier personnel", run: () => go("") },
    { icon: "recent", label: "Aller à : Récents", run: () => goView("recent") },
    { icon: "favorites", label: "Aller à : Favoris", run: () => goView("favorites") },
    { icon: "network", label: "Aller à : Réseau", run: () => goView("network") }];
  return base.concat(state.places.map((p) => ({ icon: p.id, label: "Aller à : " + p.label, run: () => go(p.path) })));
}
function openPalette() {
  if (document.querySelector(".overlay .palette")) return;
  const input = el("input", { class: "field", type: "text", placeholder: "Chercher un fichier ou une commande", "aria-label": "Palette de commandes",
    spellcheck: "false", autocomplete: "off" });
  const results = el("div", { class: "results", role: "listbox" });
  const overlay = el("div", { class: "overlay top" }, el("div", { class: "palette" }, input, results));
  let rows = [], index = 0, ticket = 0, timer;
  const close = () => overlay.remove();
  const run = (r) => { close(); r.run(); };
  const draw = (note) => {
    results.replaceChildren(...rows.map((r, i) => el("button", { class: "res", type: "button", role: "option", "aria-selected": String(i === index),
      onclick: () => run(r) }, icon(r.icon), el("span", { class: "rl" }, r.label, r.sub ? el("small", {}, r.sub) : null),
    el("span", { class: "mono dim" }, r.hint || r.tag || ""))), note ? el("div", { class: "res-note mono dim" }, note) : null);
    results.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" });
  };
  const commands = () => paletteCommands().map((c) => ({ ...c, s: fuzzy(input.value.trim(), c.label), tag: "Commande" }))
    .filter((c) => c.s !== null).sort((a, b) => a.s - b.s).slice(0, 8);
  const fileRows = (items) => items.map((it) => ({ icon: it.d ? "folder" : ICON_BY_KIND[kindOf(it)], label: it.n,
    sub: "/" + parentOf(it.p), tag: it.d ? "Dossier" : "Fichier", run: () => (it.d ? go(it.p) : (state.pending = it.p, go(parentOf(it.p)))) }));
  const search = async () => {
    const q = input.value.trim(), mine = ++ticket;
    rows = commands(); index = 0;
    if (q.length < 2) return draw();
    draw("Recherche dans le dossier personnel…");
    try {
      const data = await api("list", { p: "", h: state.hidden ? "1" : "0", q, f: "1" });
      if (mine !== ticket) return;
      rows = commands().concat(fileRows(data.items.slice(0, 20)));
      draw(data.items.length ? "" : "Aucun fichier ne correspond.");
    } catch (e) { draw(e.message); }
  };
  input.oninput = () => { clearTimeout(timer); rows = commands(); index = 0; draw(); timer = setTimeout(search, 220); };
  overlay.addEventListener("keydown", (e) => {
    if (e.key === "Escape") { e.stopPropagation(); close(); }
    else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      index = Math.min(Math.max(index + (e.key === "ArrowDown" ? 1 : -1), 0), rows.length - 1);
      draw();
    } else if (e.key === "Enter" && rows[index]) { e.preventDefault(); run(rows[index]); }
  });
  overlay.addEventListener("mousedown", (e) => { if (e.target === overlay) close(); });
  document.body.append(overlay);
  input.focus();
  rows = commands();
  draw();
}
