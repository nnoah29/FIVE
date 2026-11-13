"use strict";

async function start() {
  loadSettings();
  wire();
  $("paste").onclick = pasteClip;
  try {
    const data = await api("places", {});
    state.places = data.places;
    state.root = data.root;
    state.user = data.user;
    api("disks", {}).then((r) => { state.disks = r.disks; renderSide(); }).catch(() => {});
    let first = { path: "", virtual: null };
    if (S.restore) {
      try { first = JSON.parse(localStorage.getItem(SESSION) || "null") || first; } catch { /* session absente */ }
      if (first.path && !first.virtual) {
        try { await api("list", { p: first.path, h: "0" }); } catch { first = { path: "", virtual: null }; }
      }
    }
    await load(first.path || "", first.virtual || null);
    pollJobs();
  } catch (e) {
    $("items").replaceChildren(emptyState(e.message));
  }
}
start();
