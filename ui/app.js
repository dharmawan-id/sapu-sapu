// Sapu frontend logic. Talks to the Rust backend over Tauri's invoke bridge.
const invoke = window.__TAURI__.core.invoke;

const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));

function fmtBytes(n) {
  if (!n || n <= 0) return "0 B";
  const u = ["B", "KB", "MB", "GB", "TB"];
  let i = Math.floor(Math.log(n) / Math.log(1024));
  i = Math.max(0, Math.min(i, u.length - 1));
  const v = n / Math.pow(1024, i);
  return (v >= 100 ? v.toFixed(0) : v.toFixed(1)) + " " + u[i];
}

// Escape every dynamic value before it goes into innerHTML. Paths and names
// come from the local disk, so this is defence-in-depth, not paranoia.
function esc(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

let toastTimer = null;
function toast(msg, kind = "") {
  const t = $("#toast");
  t.textContent = msg;
  t.className = "toast is-show " + (kind ? "is-" + kind : "");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    t.className = "toast";
  }, 4200);
}

// ---------- gauges ----------
async function renderGauge(drive) {
  const el = $(`.gauge[data-drive="${drive}"]`);
  try {
    const d = await invoke("disk_info", { drive });
    if (!d.exists) {
      el.innerHTML = `<div class="gauge__skeleton">${drive}: not found</div>`;
      return;
    }
    const usedPct = d.total ? (d.used / d.total) * 100 : 0;
    const freePct = 100 - usedPct;
    const tight = freePct < 15;
    el.innerHTML = `
      <div class="gauge__top">
        <span class="gauge__drive">${drive}:</span>
        <span class="gauge__free">free<b>${fmtBytes(d.free)}</b></span>
      </div>
      <div class="gauge__bar">
        <div class="gauge__fill ${tight ? "is-tight" : ""}" style="width:${usedPct.toFixed(1)}%"></div>
      </div>
      <div class="gauge__meta">
        <span>${fmtBytes(d.used)} used</span>
        <span>${freePct.toFixed(0)}% free of ${fmtBytes(d.total)}</span>
      </div>`;
  } catch (e) {
    el.innerHTML = `<div class="gauge__skeleton">${drive}: ${esc(e)}</div>`;
  }
}

async function refreshGauges() {
  await Promise.all([renderGauge("C"), renderGauge("D")]);
}

// ---------- tabs ----------
function wireTabs() {
  $$("#tabs .tab").forEach((tab) => {
    tab.addEventListener("click", () => {
      $$("#tabs .tab").forEach((t) => t.classList.remove("is-active"));
      tab.classList.add("is-active");
      const view = tab.dataset.view;
      $("#view-overview").classList.toggle("is-hidden", view !== "overview");
      $("#view-clean").classList.toggle("is-hidden", view !== "clean");
    });
  });
}

// ---------- overview ----------
function listRows(items) {
  if (!items.length) return `<div class="empty">Nothing found.</div>`;
  return items
    .map(
      (e) => `
      <div class="listrow">
        <span class="listrow__path" title="${esc(e.path)}">${esc(e.path)}</span>
        <span class="listrow__size">${fmtBytes(e.size)}</span>
      </div>`
    )
    .join("");
}

function typeBars(items) {
  const max = items.reduce((m, x) => Math.max(m, x.size), 0) || 1;
  return items
    .map(
      (x) => `
      <div class="typebar">
        <span class="typebar__label">${esc(x.category)}</span>
        <span class="typebar__track"><span class="typebar__fill" style="width:${((x.size / max) * 100).toFixed(1)}%"></span></span>
        <span class="typebar__val">${fmtBytes(x.size)}</span>
      </div>`
    )
    .join("");
}

async function scanOverview(drive) {
  const status = $("#ov-status");
  const out = $("#ov-results");
  status.textContent = `Scanning ${drive}: ...`;
  status.classList.add("is-busy");
  $$('[data-scan]').forEach((b) => (b.disabled = true));
  try {
    const ov = await invoke("scan_overview", { drive, topn: 25 });
    out.innerHTML = `
      <div class="panel">
        <p class="panel__title">${drive}: biggest folders</p>
        ${listRows(ov.top_folders)}
      </div>
      <div class="panel">
        <p class="panel__title">${drive}: biggest files</p>
        ${listRows(ov.top_files)}
      </div>
      <div class="panel panel--wide">
        <p class="panel__title">${drive}: by file type</p>
        ${typeBars(ov.by_type)}
      </div>`;
    status.textContent = `${drive}: done. ${ov.scanned_files.toLocaleString()} files read.`;
  } catch (e) {
    toast("Scan failed: " + e, "warn");
    status.textContent = "Scan failed.";
  } finally {
    status.classList.remove("is-busy");
    $$('[data-scan]').forEach((b) => (b.disabled = false));
  }
}

// ---------- clean: green caches ----------
function cardHtml(t) {
  return `
    <div class="card" data-id="${esc(t.id)}">
      <div class="card__top">
        <input type="checkbox" class="card__check" ${t.size > 0 ? "checked" : ""} ${t.size > 0 ? "" : "disabled"} />
        <div>
          <div class="card__label">${esc(t.label)}</div>
          <div class="card__size">${fmtBytes(t.size)} <small>${t.files.toLocaleString()} files</small></div>
        </div>
      </div>
      <div class="card__note">${esc(t.note)}</div>
      <div class="card__foot">
        <span class="card__paths">${t.paths.length} path${t.paths.length === 1 ? "" : "s"}</span>
      </div>
    </div>`;
}

let greenTargets = [];
async function scanGreen() {
  const wrap = $("#green-cards");
  wrap.innerHTML = `<div class="empty is-busy">Reading cache sizes ...</div>`;
  $("#green-scan").disabled = true;
  try {
    greenTargets = await invoke("list_clean_targets");
    if (!greenTargets.length) {
      wrap.innerHTML = `<div class="empty">No caches found.</div>`;
      return;
    }
    wrap.innerHTML = greenTargets.map(cardHtml).join("");
    const total = greenTargets.reduce((s, t) => s + t.size, 0);
    toast(`Found ${fmtBytes(total)} across ${greenTargets.length} caches.`);
  } catch (e) {
    wrap.innerHTML = `<div class="empty">Scan failed: ${esc(e)}</div>`;
  } finally {
    $("#green-scan").disabled = false;
  }
}

async function cleanGreen() {
  const chosen = $$("#green-cards .card").filter((c) => {
    const cb = $(".card__check", c);
    return cb && cb.checked && !cb.disabled;
  });
  if (!chosen.length) {
    toast("Nothing selected.", "warn");
    return;
  }
  const ids = chosen.map((c) => c.dataset.id);
  const paths = greenTargets.filter((t) => ids.includes(t.id)).flatMap((t) => t.paths);
  if (!paths.length) {
    toast("Nothing to clean.", "warn");
    return;
  }
  $("#green-clean").disabled = true;
  toast("Cleaning ...");
  try {
    const r = await invoke("clean_paths", { paths });
    toast(
      `Freed ${fmtBytes(r.freed)}. ${r.deleted.toLocaleString()} deleted, ${r.skipped.toLocaleString()} skipped (in use).`,
      "ok"
    );
    await refreshGauges();
    await scanGreen();
  } catch (e) {
    toast("Clean failed: " + e, "warn");
  } finally {
    $("#green-clean").disabled = false;
  }
}

// ---------- clean: project artifacts ----------
let projArtifacts = [];
function rowHtml(a, i) {
  const flag = a.git_dirty
    ? `<span class="flag flag--dirty">uncommitted</span>`
    : a.modified_days >= 0 && a.modified_days < 30
    ? `<span class="flag flag--fresh">${a.modified_days}d old</span>`
    : `<span class="row__age">${a.modified_days}d old</span>`;
  return `
    <div class="row" data-i="${i}">
      <input type="checkbox" class="row__check" ${a.safe ? "checked" : ""} />
      <span class="row__path" title="${esc(a.path)}">${esc(a.path)}</span>
      <span class="row__kind">${esc(a.kind)}</span>
      ${flag}
      <span class="row__size">${fmtBytes(a.size)}</span>
    </div>`;
}

async function scanProjects() {
  const root = $("#proj-root").value.trim() || "D:\\Kerja";
  const wrap = $("#proj-rows");
  wrap.innerHTML = `<div class="empty is-busy">Scanning ${esc(root)} ...</div>`;
  $("#proj-scan").disabled = true;
  try {
    projArtifacts = await invoke("scan_projects", { root, depth: 6 });
    if (!projArtifacts.length) {
      wrap.innerHTML = `<div class="empty">No build or dependency folders found under ${esc(root)}.</div>`;
      return;
    }
    wrap.innerHTML = projArtifacts.map(rowHtml).join("");
    const total = projArtifacts.reduce((s, a) => s + a.size, 0);
    const safe = projArtifacts.filter((a) => a.safe).reduce((s, a) => s + a.size, 0);
    toast(`${projArtifacts.length} artifacts, ${fmtBytes(total)} total, ${fmtBytes(safe)} flagged safe.`);
  } catch (e) {
    wrap.innerHTML = `<div class="empty">Scan failed: ${esc(e)}</div>`;
  } finally {
    $("#proj-scan").disabled = false;
  }
}

async function deleteProjects() {
  const chosen = $$("#proj-rows .row").filter((r) => {
    const cb = $(".row__check", r);
    return cb && cb.checked;
  });
  if (!chosen.length) {
    toast("Nothing selected.", "warn");
    return;
  }
  const paths = chosen.map((r) => projArtifacts[Number(r.dataset.i)].path);
  $("#proj-delete").disabled = true;
  toast(`Deleting ${paths.length} folders ...`);
  try {
    const r = await invoke("delete_projects", { paths });
    toast(
      `Freed ${fmtBytes(r.freed)}. ${r.deleted.toLocaleString()} entries removed, ${r.skipped.toLocaleString()} skipped.`,
      "ok"
    );
    await refreshGauges();
    await scanProjects();
  } catch (e) {
    toast("Delete failed: " + e, "warn");
  } finally {
    $("#proj-delete").disabled = false;
  }
}

// ---------- boot ----------
function wire() {
  wireTabs();
  $$('[data-scan]').forEach((b) => b.addEventListener("click", () => scanOverview(b.dataset.scan)));
  $("#green-scan").addEventListener("click", scanGreen);
  $("#green-clean").addEventListener("click", cleanGreen);
  $("#proj-scan").addEventListener("click", scanProjects);
  $("#proj-delete").addEventListener("click", deleteProjects);
}

window.addEventListener("DOMContentLoaded", () => {
  wire();
  refreshGauges();
});
