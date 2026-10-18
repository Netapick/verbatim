// Thème : choix mémorisé, sinon celui du système.
const themeNow = () => document.documentElement.dataset.theme || (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
try { const t = localStorage.getItem("theme"); if (t) document.documentElement.dataset.theme = t; } catch (_) {}
const { invoke, convertFileSrc } = window.__TAURI__.core;
const dialog = window.__TAURI__.dialog;
const $ = (s, r = document) => r.querySelector(s);
const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
const main = $("#main");
let view = "meetings", poll = null, renderSeq = 0;
// Un seul minuteur de rafraîchissement actif : le précédent est toujours supprimé.
function setPoll(fn, ms) { clearInterval(poll); poll = setInterval(fn, ms); }

function toast(msg) {
  const t = $("#toast"); t.textContent = msg; t.classList.add("show");
  clearTimeout(t._h); t._h = setTimeout(() => t.classList.remove("show"), 2800);
}

// Confirmation dans la page (le confirm() natif ne bloque pas toujours dans la webview).
function ask(msg, ok = "Supprimer") {
  return new Promise((res) => {
    const o = document.createElement("div"); o.className = "modal";
    o.innerHTML = `<div class="dlg" role="alertdialog" aria-modal="true"><p></p><div class="acts"><button class="btn" data-n>Annuler</button><button class="btn pri dng2" data-y></button></div></div>`;
    o.querySelector("p").textContent = msg; o.querySelector("[data-y]").textContent = ok;
    const done = (v) => { document.removeEventListener("keydown", key, true); o.remove(); res(v); };
    const key = (e) => { if (e.key === "Escape") { e.preventDefault(); done(false); } };
    o.onclick = (e) => { if (e.target === o) done(false); };
    o.querySelector("[data-n]").onclick = () => done(false);
    o.querySelector("[data-y]").onclick = () => done(true);
    document.addEventListener("keydown", key, true);
    document.body.appendChild(o); o.querySelector("[data-n]").focus();
  });
}

function askText(msg, value = "") {
  return new Promise((res) => {
    const o = document.createElement("div"); o.className = "modal";
    o.innerHTML = `<div class="dlg" role="dialog" aria-modal="true"><p></p><input style="margin-bottom:16px"><div class="acts"><button class="btn" data-n>Annuler</button><button class="btn pri" data-y>Valider</button></div></div>`;
    o.querySelector("p").textContent = msg; const inp = o.querySelector("input"); inp.value = value;
    const done = (v) => { o.remove(); res(v); };
    o.onclick = (e) => { if (e.target === o) done(null); };
    o.querySelector("[data-n]").onclick = () => done(null);
    o.querySelector("[data-y]").onclick = () => done(inp.value.trim() || null);
    inp.onkeydown = (e) => { if (e.key === "Enter") done(inp.value.trim() || null); if (e.key === "Escape") done(null); };
    document.body.appendChild(o); inp.focus(); inp.select();
  });
}
const fmtDate = (d) => new Date(d).toLocaleString("fr-FR", { dateStyle: "medium", timeStyle: "short" });
const fmtT = (s) => `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(Math.floor(s % 60)).padStart(2, "0")}`;
const { listen } = window.__TAURI__.event;
const fmtSize = (n) => n >= 1073741824 ? (n / 1073741824).toFixed(2) + " Go" : (n / 1048576).toFixed(0) + " Mo";
const dlPct = (d) => (d.total ? Math.min(100, Math.round((d.done / d.total) * 100)) : 0);
const dlLabel = (d) => `${{ binary: "Moteur", vad: "Détection de parole", model: "Modèle", "llm-binary": "Moteur IA", "llm-model": "Modèle IA", copy: "Copie", "binary-gpu": "Moteur GPU", }[d.phase] || "Téléchargement"} : ${dlPct(d)} % (${fmtSize(d.done)} / ${fmtSize(d.total)})`;

// L'état du téléchargement vit côté Rust : il survit aux changements de page.
// Cette fonction se (re)branche dessus à chaque affichage concerné.
let dlTimer = null, engineHook = null;
function watchDownload() {
  if (dlTimer) return;
  dlTimer = setInterval(async () => {
    const st = await invoke("engine_status");
    const d = st.download;
    document.querySelectorAll("[data-dlmsg]").forEach((e) => { e.textContent = d.active ? dlLabel(d) : ""; });
    document.querySelectorAll("[data-dlbar]").forEach((e) => { e.style.width = dlPct(d) + "%"; });
    if (!d.active) {
      clearInterval(dlTimer); dlTimer = null;
      if (d.error) toast("Téléchargement interrompu : " + d.error + " — vous pourrez le reprendre.");
      else toast("Moteur Whisper prêt");
      if (engineHook) engineHook();
    }
  }, 700);
}
listen("engine-done", () => { if (engineHook) engineHook(); });

// Bandeau « moteur non installé / téléchargement » pour le modèle choisi.
async function engineBox() {
  const [st, cfg] = await Promise.all([invoke("engine_status"), invoke("get_settings")]);
  const m = st.models.find((x) => x.name === cfg.whisper_model) || st.models[0];
  const d = st.download;
  if (d.active) {
    watchDownload();
    return `<div class="card"><b>Téléchargement en cours…</b><div class="prog" style="width:100%;margin:8px 0"><b data-dlbar style="width:${dlPct(d)}%"></b></div>
      <span class="meta" data-dlmsg>${esc(dlLabel(d))}</span><p class="meta" style="margin-top:6px">Vous pouvez changer de page ou fermer l'app : le téléchargement reprendra là où il s'est arrêté.</p></div>`;
  }
  if (st.binary && m.installed) return "";
  const pct = m.partial ? Math.round((m.partial / m.bytes) * 100) : 0;
  return `<div class="card"><b>${m.partial ? "Téléchargement interrompu" : "Moteur de transcription non installé"}</b>
    <p class="meta" style="margin:4px 0 10px">Whisper s'exécute en local, l'audio ne quitte jamais cet ordinateur. Modèle « ${esc(m.label)} » : ${fmtSize(m.bytes)}${m.partial ? ` — déjà ${pct} % téléchargés` : ""}. Téléchargé une seule fois, puis conservé.</p>
    <button class="btn pri" id="eng-go">${m.partial ? "Reprendre le téléchargement" : "Installer Whisper"}</button>
    ${d.error ? `<span class="meta" style="color:var(--danger)"> ${esc(d.error)}</span>` : ""}</div>`;
}
function bindEngine(model) {
  const b = document.getElementById("eng-go"); if (!b) return;
  b.onclick = () => run(async () => {
    b.disabled = true; await invoke("engine_install", { model });
    if (engineHook) engineHook(); watchDownload();
  });
}
async function run(fn) { try { return await fn(); } catch (e) { toast(String(e)); } }

document.getElementById("nav").addEventListener("click", (e) => {
  const b = e.target.closest("button"); if (b) go(b.dataset.view);
});
function go(v, arg) {
  view = v; clearInterval(poll); renderSeq++; main.onclick = null;
  document.querySelectorAll("nav button").forEach((b) => b.classList.toggle("on",
    b.dataset.view === v || (v === "meeting" && b.dataset.view === "meetings") || (v === "brief" && b.dataset.view === "briefs")));
  ({ meetings, meeting, briefs, brief, glossary, trash, settings })[v](arg);
}

// ── Enregistrement depuis le PC ─────────────────────────────────────────
let recTimer = null;
const fmtClock = (n) => `${String(Math.floor(n / 3600)).padStart(2, "0")}:${String(Math.floor(n % 3600 / 60)).padStart(2, "0")}:${String(n % 60).padStart(2, "0")}`;
const meter = (v) => `<span class="meter"><b style="width:${Math.min(100, Math.round((v || 0) / 5))}%"></b></span>`;

const lsGet = (k) => { try { return localStorage.getItem(k) || ""; } catch (_) { return ""; } };
const lsSet = (k, v) => { try { localStorage.setItem(k, v); } catch (_) {} };

const fmtGain = (v) => "× " + (v / 100).toFixed(2).replace(/\.?0+$/, "");
async function recPanel() {
  const box = $("#recbox"); if (!box) return;
  const dv = (await run(() => invoke("rec_devices"))) || { inputs: [], outputs: [] };
  const opts = (list, def, saved) => `<option value="">Par défaut${def ? " — " + esc(def) : ""}</option>` +
    list.map((n) => `<option value="${esc(n)}" ${n === saved ? "selected" : ""}>${esc(n)}</option>`).join("");
  box.innerHTML = `<div class="card"><b>Nouvel enregistrement</b>
    <div class="grid2" style="margin-top:10px">
      <div><label class="chk"><input type="checkbox" id="r-mic" ${lsGet("rec.mic") !== "0" ? "checked" : ""}> Microphone</label>
        <select id="r-micdev" style="margin-top:6px">${opts(dv.inputs, dv.default_input, lsGet("rec.micdev"))}</select>
        <label style="margin-top:10px">Sensibilité du micro <span class="pill" id="r-micg-v"></span></label>
        <input type="range" id="r-micg" min="25" max="800" step="5" value="${Math.round(parseFloat(lsGet("rec.micgain") || "1.5") * 100)}"></div>
      <div><label class="chk"><input type="checkbox" id="r-sys" ${lsGet("rec.sys") === "1" ? "checked" : ""}> Son du PC (visio, vidéo…)</label>
        <select id="r-sysdev" style="margin-top:6px">${opts(dv.outputs, dv.default_output, lsGet("rec.sysdev"))}</select>
        <label style="margin-top:10px">Sensibilité du son du PC <span class="pill" id="r-sysg-v"></span></label>
        <input type="range" id="r-sysg" min="25" max="800" step="5" value="${Math.round(parseFloat(lsGet("rec.sysgain") || "1") * 100)}"></div>
    </div>
    <div class="bar" style="margin:12px 0 0"><button class="btn rec" id="r-go">● Démarrer</button><button class="btn" id="r-x">Annuler</button></div>
    <p class="meta" style="margin-top:8px">Pour une visioconférence, cochez les deux : votre voix et celle de vos interlocuteurs sont enregistrées. Pour « Son du PC », choisissez la sortie sur laquelle vous écoutez (casque, haut-parleurs).</p></div>`;
  const fmtG = fmtGain;
  const gv = (id, out) => { const e = $(id), o = $(out); e.oninput = () => { o.textContent = fmtG(e.value); }; o.textContent = fmtG(e.value); return e; };
  const gm = gv("#r-micg", "#r-micg-v"), gs = gv("#r-sysg", "#r-sysg-v");
  $("#r-x").onclick = () => { box.innerHTML = ""; };
  $("#r-go").onclick = () => run(async () => {
    const mic = $("#r-mic").checked, system = $("#r-sys").checked;
    const micDevice = $("#r-micdev").value, sysDevice = $("#r-sysdev").value;
    const micGain = gm.value / 100, sysGain = gs.value / 100;
    await invoke("rec_start", { mic, system, micDevice: micDevice || null, sysDevice: sysDevice || null, micGain, sysGain });
    lsSet("rec.micgain", String(micGain)); lsSet("rec.sysgain", String(sysGain));
    lsSet("rec.mic", mic ? "1" : "0"); lsSet("rec.sys", system ? "1" : "0"); lsSet("rec.micdev", micDevice); lsSet("rec.sysdev", sysDevice);
    recRefresh();
  });
}

async function recRefresh() {
  clearInterval(recTimer);
  const box = $("#recbox"); if (!box) return;
  const st = await run(() => invoke("rec_status"));
  if (!st || !st.recording) { return; }
  const draw = async () => {
    const b = $("#recbox"); if (!b || view !== "meetings") return clearInterval(recTimer);
    const s = await invoke("rec_status");
    if (!s.recording) { clearInterval(recTimer); b.innerHTML = ""; return; }
    b.innerHTML = `<div class="card recing ${s.paused ? "paused" : ""}"><span class="dot"></span><b>${s.paused ? "En pause" : "Enregistrement en cours"}</b>
      <span class="clock">${fmtClock(s.seconds)}</span>
      ${s.mic != null ? `<span class="meta">Micro</span>${meter(s.mic)}` : ""}${s.system != null ? `<span class="meta">PC</span>${meter(s.system)}` : ""}
      ${s.mic != null ? `<label class="chk" title="Sensibilité du micro" style="gap:6px">Sensibilité <input type="range" id="r-live-g" min="25" max="800" step="5" value="${Math.round((s.mic_gain || 1) * 100)}" style="width:110px"><span class="pill" id="r-live-gv">${fmtGain((s.mic_gain || 1) * 100)}</span></label>` : ""}
      <input id="r-title" placeholder="Titre (facultatif)" style="max-width:240px">
      <button class="btn" id="r-pause">${s.paused ? "▶ Reprendre" : "❚❚ Pause"}</button>
      <button class="btn pri" id="r-stop">■ Terminer</button></div>`;
    const lg = $("#r-live-g");
    if (lg) lg.oninput = () => { $("#r-live-gv").textContent = fmtGain(lg.value); invoke("rec_set_gain", { source: "mic", gain: lg.value / 100 }).catch(() => {}); lsSet("rec.micgain", String(lg.value / 100)); };
    $("#r-pause").onclick = () => run(async () => { await invoke("rec_pause", { paused: !s.paused }); draw(); });
    $("#r-stop").onclick = () => run(async () => {
      clearInterval(recTimer);
      const title = $("#r-title").value;
      const id = await invoke("rec_stop", { title });
      toast("Enregistrement ajouté — transcription lancée"); go("meeting", id);
    });
  };
  await draw(); recTimer = setInterval(async () => {
    if (view !== "meetings" || !$("#recbox")) return clearInterval(recTimer);
    // ne redessine pas pendant la saisie du titre : met seulement à jour horloge et niveaux
    const s = await invoke("rec_status"); if (!s.recording) return draw();
    const wasPaused = document.querySelector(".recing")?.classList.contains("paused") || false;
    if (s.paused !== wasPaused) return draw();
    const c = document.querySelector(".recing .clock"); if (c) c.textContent = fmtClock(s.seconds);
    const ms = document.querySelectorAll(".recing .meter b");
    let i = 0; if (s.mic != null && ms[i]) ms[i++].style.width = Math.min(100, Math.round(s.mic / 5)) + "%";
    if (s.system != null && ms[i]) ms[i].style.width = Math.min(100, Math.round(s.system / 5)) + "%";
  }, 400);
}

// ── Réunions ────────────────────────────────────────────────────────────
async function meetings() {
  main.innerHTML = `<h1>Réunions</h1><p class="sub">Importez un enregistrement ou une transcription existante.</p>
    <div class="bar"><button class="btn rec" id="rec">● Enregistrer</button><button class="btn pri" id="ia">+ Importer un audio</button><button class="btn" id="it">Importer une transcription</button></div>
    <div id="recbox"></div><div id="eng"></div><div id="list"></div>`;
  $("#rec").onclick = () => recPanel();
  recRefresh();
  const cfg0 = await invoke("get_settings");
  const showEng = async () => { const e = $("#eng"); if (!e) return; e.innerHTML = await engineBox(); bindEngine((await invoke("get_settings")).whisper_model); };
  engineHook = () => view === "meetings" && showEng();
  await showEng();
  $("#ia").onclick = () => run(async () => {
    const f = await dialog.open({ multiple: true, filters: [{ name: "Audio", extensions: ["mp3", "wav", "m4a", "ogg", "flac", "aac", "webm", "mp4", "opus"] }] });
    if (!f) return;
    const n = await invoke("import_audio", { paths: [].concat(f) });
    toast(`${n} fichier(s) importé(s)`); draw(); setPoll(() => view === "meetings" && draw(), 1500);
  });
  $("#it").onclick = () => run(async () => {
    const f = await dialog.open({ filters: [{ name: "Texte", extensions: ["txt", "srt", "vtt", "md"] }] });
    if (!f) return;
    const id = await invoke("import_transcript", { path: f }); go("meeting", id);
  });
  async function draw() {
    const ms = (await run(() => invoke("list_meetings", { trashed: false }))) || [];
    const html = ms.length ? ms.map((m) => `<div class="card row" data-id="${m.id}">
      <span class="t">${esc(m.title)}</span><span class="meta">${fmtDate(m.created_at)}</span>
      ${m.stage === "ready" ? `<span class="pill ok">Prête</span>` : m.stage === "error" ? `<span class="pill err">Échec</span>` : m.stage === "needs_engine" ? `<span class="pill warn">Moteur requis</span>` : `<div class="prog"><b style="width:${m.progress}%"></b></div><span class="pill">${esc(m.stage_label)}${m.stage === "transcribing" ? " · " + m.progress + " %" : ""}</span>`}
    </div>`).join("") : `<div class="empty">Aucune réunion pour le moment.</div>`;
    const list = $("#list");
    if (list && list.dataset.h !== html) { list.innerHTML = html; list.dataset.h = html; } // ne redessine que si ça a changé
    // plus rien en cours : inutile de continuer à interroger
    if (ms.length && ms.every((m) => m.stage === "ready" || m.stage === "error" || m.stage === "needs_engine")) clearInterval(poll);
  }
  $("#list").onclick = (e) => { const c = e.target.closest(".row"); if (c) go("meeting", c.dataset.id); };
  await draw();
  if (view === "meetings") setPoll(() => view === "meetings" && draw(), 1500);
}

const reportBadge = (r) => {
  if (!r) return "";
  if (r.mode === "extractif") return `<div class="note">${esc(r.note || "Résumé automatique simplifié.")} <a href="#" id="to-set" class="fr-link">Ouvrir les Réglages</a></div>`;
  const when = r.generated_at ? ` · ${fmtDate(r.generated_at)}` : "";
  return `<p class="meta" style="margin-bottom:8px"><span class="pill ${r.mode === "local" ? "ok" : ""}">${esc(r.model || "IA")}${r.mode === "local" ? " · local" : ""}</span>${r.hardware ? ` <span class="pill hw">${esc(r.hardware)}</span>` : ""}${when}</p>`;
};

async function meeting(id) {
  clearInterval(poll);
  const seq = ++renderSeq;
  const m = await run(() => invoke("get_meeting", { id }));
  if (!m) return go("meetings");
  const bs = await invoke("list_briefs", { trashed: false });
  if (seq !== renderSeq || view !== "meeting") return; // un rendu plus récent a pris la main
  // mémorise la lecture en cours pour la reprendre après le redessin
  const old = $("#aud");
  const keep = old ? { t: old.currentTime, playing: !old.paused, rate: old.playbackRate } : null;
  const r = m.report;
  const lst = (a) => (a && a.length) ? `<ul class="l">${a.map((x) => `<li>${esc(x)}</li>`).join("")}</ul>` : "<p class='meta'>—</p>";
  const origin = (m.source === "audio" ? "Audio · transcription Whisper locale" : "Transcription importée") + (m.source === "audio" && m.transcribe_hw ? " · " + m.transcribe_hw : "");
  const player = m.has_audio ? `<div class="card player"><audio id="aud" controls preload="metadata" src="${esc(convertFileSrc(m.audio_path))}"></audio>
    <select id="spd" title="Vitesse de lecture">${[0.75, 1, 1.25, 1.5, 2].map((x) => `<option value="${x}" ${x === 1 ? "selected" : ""}>${x}×</option>`).join("")}</select></div>` : "";
  const bindPlayer = () => {
    const a = $("#aud"); if (!a) return;
    a.onerror = () => toast("Impossible de lire cet audio (format non pris en charge ?)");
    $("#spd").onchange = (e) => { a.playbackRate = parseFloat(e.target.value); };
    const segs = [...main.querySelectorAll(".seg")];
    segs.forEach((el) => el.onclick = () => { a.currentTime = parseFloat(el.dataset.t); a.play(); });
    let cur = null;
    a.ontimeupdate = () => {
      const t = a.currentTime;
      const hit = segs.find((el) => t >= parseFloat(el.dataset.t) && t < parseFloat(el.dataset.e));
      if (hit === cur) return;
      cur && cur.classList.remove("now"); cur = hit || null;
      if (cur) { cur.classList.add("now"); if (!a.paused) cur.scrollIntoView({ block: "nearest", behavior: "smooth" }); }
    };
  };
  const body = m.stage === "error"
    ? `<div class="card"><b>La transcription a échoué</b><p class="meta" style="margin:6px 0 10px">${esc(m.error || "")}</p><button class="btn pri" id="retry">Réessayer</button> <button class="btn dng" id="del">Corbeille</button></div>`
    : m.stage === "needs_engine"
    ? `<div id="eng2"></div>`
    : m.stage !== "ready"
    ? `<div class="card"><div class="prog" style="width:100%"><b style="width:${m.progress}%"></b></div><p class="meta" style="margin-top:8px">${m.stage === "transcribing" ? `<span class="pill" id="tr-pct">${m.progress} %</span> ` : ""}${esc(m.stage_label)}…${m.transcribe_hw ? ` <span class="pill hw">${esc(m.transcribe_hw)}</span>` : ""}</p></div>`
    : `<div class="bar">
      <button class="btn pri" id="gen" ${m.report_state === "generating" ? "disabled" : ""}>${r ? "Régénérer" : "Générer"} le compte-rendu</button>
      <button class="btn" id="ren">Renommer</button>
      <select id="bf" style="width:auto"><option value="">Aucune préparation liée</option>${bs.map((b) => `<option value="${b.id}" ${m.brief_id === b.id ? "selected" : ""}>${esc(b.subject)}</option>`).join("")}</select>
      <select id="exp" style="width:auto"><option value="">Exporter…</option><optgroup label="Transcription"><option value="txt">Texte (.txt)</option><option value="srt">Sous-titres (.srt)</option><option value="vtt">Sous-titres (.vtt)</option><option value="json">Données (.json)</option></optgroup>${r ? `<optgroup label="Compte-rendu"><option value="report">Compte-rendu (.md)</option></optgroup>` : ""}${m.has_audio ? `<optgroup label="Enregistrement"><option value="audio">Audio</option></optgroup>` : ""}</select>
      <button class="btn dng" id="del">Corbeille</button></div>
    ${m.report_state === "generating" ? `<div class="card"><b>Génération du compte-rendu…</b> <span class="pill" id="rep-pct">${m.report_pct || 0} %</span> <span class="meta" id="rep-prog">${esc(m.report_progress || "")}</span> <span class="pill hw" id="rep-hw" ${m.report_hw ? "" : "hidden"}>${esc(m.report_hw || "")}</span>
      <div class="prog" style="width:100%;margin:8px 0"><b id="rep-bar" style="width:${m.report_pct || 0}%"></b></div>
      <p class="meta">Avec l'IA locale, une longue réunion peut prendre plusieurs minutes. Vous pouvez quitter cette page : la génération continue.</p></div>` : ""}
    ${m.report_state === "error" ? `<div class="card"><b>Échec de la génération</b><p class="meta" style="margin-top:4px">${esc(m.report_error || "")}</p></div>` : ""}
    ${r ? `<div class="card">${reportBadge(r)}<h3 style="margin-top:0">Résumé</h3><p>${esc(r.summary)}</p><h3>Points abordés</h3>${lst(r.topics)}<h3>Décisions</h3>${lst(r.decisions)}<h3>Actions</h3>${lst(r.actions)}<h3>Prochaines étapes</h3>${lst(r.next_steps)}</div>` : ""}
    <div class="card"><h3 style="margin-top:0">Transcription</h3>${m.transcript.map((s) => `<div class="seg" data-t="${s.start}" data-e="${s.end}" title="Écouter à partir d'ici"><span class="tm">${fmtT(s.start)}</span><span class="sp">${esc(s.speaker)}</span><span>${esc(s.text)}</span></div>`).join("")}</div>`;
  main.innerHTML = `<div class="bar"><button class="btn sm" id="back">← Réunions</button></div>
    <h1>${esc(m.title)}</h1><p class="sub">${fmtDate(m.created_at)} · ${origin}</p>${player}${body}`;
  $("#back").onclick = () => go("meetings");
  bindPlayer();
  if (keep) {
    const a = $("#aud"), spd = $("#spd");
    const restore = () => { a.currentTime = keep.t; a.playbackRate = keep.rate; if (spd) spd.value = String(keep.rate); if (keep.playing) a.play().catch(() => {}); };
    if (a.readyState >= 1) restore(); else a.addEventListener("loadedmetadata", restore, { once: true });
  }
  if (m.stage === "error") {
    $("#retry").onclick = () => run(async () => { await invoke("retry_transcription", { id }); meeting(id); });
    $("#del").onclick = () => run(async () => { if (!(await ask("Placer cette réunion dans la corbeille ?", "Mettre à la corbeille"))) return; await invoke("update_meeting", { id, trashed: true }); go("meetings"); });
    return;
  }
  if (m.stage === "needs_engine") {
    const showEng2 = async () => { const e = $("#eng2"); if (!e) return; e.innerHTML = await engineBox(); bindEngine((await invoke("get_settings")).whisper_model); };
    engineHook = () => view === "meeting" && showEng2();
    await showEng2();
    setPoll(async () => { if (view !== "meeting") return; const x = await invoke("get_meeting", { id }); if (x.stage !== m.stage) meeting(id); }, 3000); return;
  }
  if (m.stage !== "ready") {
    setPoll(async () => { if (view !== "meeting") return; const x = await invoke("get_meeting", { id });
      if (x.stage !== m.stage) return meeting(id);
      const b = main.querySelector(".prog b"); if (b) b.style.width = x.progress + "%";
      const tp = $("#tr-pct"); if (tp) tp.textContent = x.progress + " %"; }, 1500);
    return;
  }
  $("#gen").onclick = () => run(async () => { await invoke("generate_report", { id }); meeting(id); });
  const toSet = $("#to-set"); if (toSet) toSet.onclick = (e) => { e.preventDefault(); go("settings"); };
  if (m.report_state === "generating") {
    setPoll(async () => {
      if (view !== "meeting") return;
      const x = await invoke("get_meeting", { id });
      if (x.report_state !== "generating") return meeting(id);
      const el = $("#rep-prog"); if (el) el.textContent = x.report_progress || "";
      const hwe = $("#rep-hw"); if (hwe) { hwe.textContent = x.report_hw || ""; hwe.hidden = !x.report_hw; }
      const pc = x.report_pct || 0, pe = $("#rep-pct"), pb = $("#rep-bar");
      if (pe) pe.textContent = pc + " %"; if (pb) pb.style.width = pc + "%";
    }, 1200);
  }
  $("#ren").onclick = () => run(async () => {
    const t = await askText("Nouveau titre", m.title);
    if (t) { await invoke("update_meeting", { id, title: t }); meeting(id); }
  });
  $("#bf").onchange = (e) => run(() => invoke("update_meeting", { id, briefId: e.target.value }));
  $("#del").onclick = () => run(async () => { if (!(await ask("Placer cette réunion dans la corbeille ?", "Mettre à la corbeille"))) return; await invoke("update_meeting", { id, trashed: true }); toast("Placée dans la corbeille"); go("meetings"); });
  $("#exp").onchange = (e) => run(async () => {
    const x = e.target.value; e.target.value = ""; if (!x) return;
    const ext = x === "report" ? "md" : x === "audio" ? ((m.audio_path.match(/\.([A-Za-z0-9]+)$/) || [])[1] || "wav") : x;
    const dest = await dialog.save({ defaultPath: `${m.title}.${ext}` });
    if (!dest) return;
    await invoke("export_meeting", { id, format: x, dest }); toast("Exporté");
  });
}

// ── Préparations ────────────────────────────────────────────────────────
let KINDS = {};
async function loadKinds() {
  const list = (await run(() => invoke("list_kinds"))) || [];
  KINDS = Object.fromEntries(list.map((k) => [k.id, k.label]));
  return list;
}

// Fenêtre de gestion des types de réunion : ajouter, renommer, supprimer, consigne pour l'IA.
function kindsEditor(list) {
  return new Promise((res) => {
    const o = document.createElement("div"); o.className = "modal";
    o.innerHTML = `<div class="dlg wide" role="dialog" aria-modal="true"><h3 style="margin:0 0 4px">Types de réunion</h3>
      <p class="meta" style="margin-bottom:12px">Le nom apparaît dans les listes. La consigne (facultative) guide l'IA, par exemple « Tu prépares un comité budgétaire, centré sur les chiffres ».</p>
      <div id="krows" class="krows"></div>
      <button class="btn sm" id="kadd" style="margin-top:8px">+ Ajouter un type</button>
      <div class="acts" style="margin-top:16px"><button class="btn" data-n>Annuler</button><button class="btn pri" data-y>Enregistrer</button></div></div>`;
    const rows = o.querySelector("#krows");
    const add = (k = { id: "", label: "", prompt: "" }) => {
      const r = document.createElement("div"); r.className = "krow"; r.dataset.id = k.id;
      r.innerHTML = `<input class="kl" placeholder="Nom du type" value="${esc(k.label)}"><input class="kp" placeholder="Consigne pour l'IA (facultatif)" value="${esc(k.prompt)}"><button class="btn sm ico" title="Supprimer ce type" aria-label="Supprimer ce type">✕</button>`;
      r.querySelector("button").onclick = () => { if (rows.children.length > 1) r.remove(); else toast("Il faut au moins un type"); };
      rows.appendChild(r);
    };
    list.forEach(add);
    o.querySelector("#kadd").onclick = () => { add(); rows.lastChild.firstChild.focus(); };
    const done = (v) => { document.removeEventListener("keydown", key, true); o.remove(); res(v); };
    const key = (e) => { if (e.key === "Escape") { e.preventDefault(); done(null); } };
    o.onclick = (e) => { if (e.target === o) done(null); };
    o.querySelector("[data-n]").onclick = () => done(null);
    o.querySelector("[data-y]").onclick = () => {
      const out = [...rows.children].map((r) => ({ id: r.dataset.id, label: r.querySelector(".kl").value.trim(), prompt: r.querySelector(".kp").value.trim() })).filter((k) => k.label);
      if (!out.length) return toast("Il faut au moins un type");
      done(out);
    };
    document.addEventListener("keydown", key, true);
    document.body.appendChild(o); (rows.querySelector("input") || o).focus();
  });
}

const briefStats = (c) => {
  const ag = c.agenda || [], min = ag.reduce((n, x) => n + (x.minutes || 0), 0), nq = (c.questions || []).length;
  return `${ag.length} point${ag.length > 1 ? "s" : ""}${min ? " · " + min + " min" : ""} · ${nq} question${nq > 1 ? "s" : ""}`;
};

async function briefs(openForm) {
  const kinds = await loadKinds();
  const bs = (await run(() => invoke("list_briefs", { trashed: false }))) || [];
  main.innerHTML = `<div class="bar" style="justify-content:space-between;margin-bottom:6px"><div><h1>Préparations</h1><p class="sub" style="margin:0">Préparez chaque réunion : objectif, ordre du jour et questions à poser.</p></div>
      <button class="btn pri" id="new">+ Nouvelle préparation</button></div>
    <div class="card" id="form" ${bs.length && !openForm ? "hidden" : ""} style="margin-top:18px"><h3>Nouvelle préparation</h3>
      <div class="grid2"><div><label style="margin-top:0">Sujet de la réunion</label><input id="s" placeholder="Ex. Revue de budget T3"></div>
      <div><label style="margin-top:0">Type de réunion <button class="lnk" id="kman" type="button">Gérer les types</button></label><select id="k">${kinds.map((k) => `<option value="${esc(k.id)}">${esc(k.label)}</option>`).join("")}</select></div></div>
      <label>Thèmes à aborder <span class="meta">(séparés par des virgules, facultatif)</span></label><input id="th" placeholder="budget, risques, planning">
      <div class="bar" style="margin:14px 0 0"><button class="btn pri" id="go">Générer la préparation</button>${bs.length ? `<button class="btn" id="cancel">Annuler</button>` : ""}<span class="meta" id="gmsg"></span></div></div>
    <div class="bgrid" style="margin-top:18px">${bs.map((b) => `<div class="card bcard" data-id="${b.id}" tabindex="0">
        <div class="bhead"><span class="pill">${esc(KINDS[b.kind] || b.kind)}</span><span class="meta">${fmtDate(b.created_at)}</span></div>
        <div class="btitle">${esc(b.subject)}</div>
        <div class="bthemes">${(b.themes || []).map((t) => `<span class="tag">${esc(t)}</span>`).join("")}</div>
        <div class="meta bstat">${briefStats(b.content || {})}</div></div>`).join("")}</div>
    ${bs.length ? "" : `<div class="empty" style="margin-top:14px">Aucune préparation pour l'instant.<br>Renseignez un sujet ci-dessus pour générer un plan de réunion.</div>`}`;
  $("#new").onclick = () => { const f = $("#form"); f.hidden = !f.hidden; if (!f.hidden) $("#s").focus(); };
  const c = $("#cancel"); if (c) c.onclick = () => { $("#form").hidden = true; };
  const gen = async () => {
    const subject = $("#s").value.trim();
    if (!subject) { $("#s").focus(); return toast("Indiquez un sujet"); }
    const themes = $("#th").value.split(",").map((x) => x.trim()).filter(Boolean);
    $("#go").disabled = true; $("#gmsg").textContent = "Génération en cours…";
    try { const b = await invoke("create_brief", { subject, kind: $("#k").value, themes }); go("brief", b.id); }
    catch (e) { $("#go").disabled = false; $("#gmsg").textContent = ""; toast(String(e)); }
  };
  $("#go").onclick = gen;
  $("#kman").onclick = () => run(async () => {
    const out = await kindsEditor(kinds); if (!out) return;
    await invoke("set_kinds", { kinds: out }); briefs(true);
  });
  $("#s").onkeydown = (e) => { if (e.key === "Enter") gen(); };
  main.querySelectorAll(".bcard").forEach((r) => { r.onclick = () => go("brief", r.dataset.id); r.onkeydown = (e) => { if (e.key === "Enter") go("brief", r.dataset.id); }; });
}

async function brief(id) {
  await loadKinds();
  const b = (await invoke("list_briefs", { trashed: false })).find((x) => x.id === id);
  if (!b) return go("briefs");
  const c = b.content || {};
  const linked = ((await run(() => invoke("list_meetings", { trashed: false }))) || []).filter((m) => m.brief_id === id);
  main.innerHTML = `<div class="bar"><button class="btn sm" id="back">← Préparations</button></div>
    <div class="bar" style="justify-content:space-between;align-items:flex-start;margin-bottom:6px"><div style="min-width:0"><h1 id="ttl" style="margin:0">${esc(b.subject)}</h1>
      <p class="sub" style="margin:4px 0 0">${esc(KINDS[b.kind] || b.kind)}${b.themes.length ? " · " : ""}${b.themes.map((t) => `<span class="tag">${esc(t)}</span>`).join("")}</p></div>
      <div class="bar" style="margin:0"><button class="btn" id="ren">Renommer</button><button class="btn" id="copy">Copier</button><button class="btn dng" id="del">Corbeille</button></div></div>
    <div class="card" style="margin-top:16px"><h3>Objectif</h3><textarea id="obj" placeholder="Que doit-on avoir obtenu à la fin de la réunion ?">${esc(c.objective || "")}</textarea></div>
    <div class="card"><div class="bar" style="justify-content:space-between;margin:0 0 8px"><h3 style="margin:0">Ordre du jour</h3><span class="meta" id="tot"></span></div>
      <div id="ag"></div><button class="btn sm" id="ag-add" style="margin-top:6px">+ Point</button></div>
    <div class="card"><h3>Questions à poser</h3><div id="qs"></div><button class="btn sm" id="q-add" style="margin-top:6px">+ Question</button></div>
    ${linked.length ? `<div class="card"><h3>Réunions liées</h3>${linked.map((m) => `<div class="lrow" data-m="${m.id}"><span class="t">${esc(m.title)}</span><span class="meta">${fmtDate(m.created_at)}</span></div>`).join("")}</div>` : ""}
    <div class="bar sticky"><button class="btn pri" id="save">Enregistrer</button><span class="meta" id="dirty"></span></div>`;
  const mark = () => { $("#dirty").textContent = "Modifications non enregistrées"; };
  const total = () => { const n = [...document.querySelectorAll("#ag .arow input[type=number]")].reduce((t, i) => t + (parseInt(i.value) || 0), 0); $("#tot").textContent = n ? `Durée prévue : ${n} min` : ""; };
  const addAg = (t = "", m = "") => {
    const r = document.createElement("div"); r.className = "arow";
    r.innerHTML = `<input class="at" placeholder="Point à traiter" value="${esc(t)}"><input type="number" min="0" placeholder="min" value="${esc(m ?? "")}"><button class="btn sm ico" title="Retirer" aria-label="Retirer">✕</button>`;
    r.querySelector("button").onclick = () => { r.remove(); mark(); total(); };
    r.oninput = () => { mark(); total(); };
    $("#ag").appendChild(r);
  };
  const addQ = (t = "") => {
    const r = document.createElement("div"); r.className = "arow";
    r.innerHTML = `<input class="qt" placeholder="Question" value="${esc(t)}"><button class="btn sm ico" title="Retirer" aria-label="Retirer">✕</button>`;
    r.querySelector("button").onclick = () => { r.remove(); mark(); };
    r.oninput = mark;
    $("#qs").appendChild(r);
  };
  (c.agenda || []).forEach((a) => addAg(a.title, a.minutes));
  (c.questions || []).forEach((q) => addQ(q));
  total();
  $("#ag-add").onclick = () => { addAg(); mark(); $("#ag").lastChild.firstChild.focus(); };
  $("#q-add").onclick = () => { addQ(); mark(); $("#qs").lastChild.firstChild.focus(); };
  $("#obj").oninput = mark;
  const collect = () => ({
    objective: $("#obj").value.trim(),
    agenda: [...document.querySelectorAll("#ag .arow")].map((r) => ({ title: r.querySelector(".at").value.trim(), minutes: parseInt(r.querySelector("input[type=number]").value) || null })).filter((a) => a.title),
    questions: [...document.querySelectorAll("#qs .qt")].map((i) => i.value.trim()).filter(Boolean),
  });
  const save = () => run(async () => { await invoke("update_brief", { id, content: collect() }); $("#dirty").textContent = ""; toast("Enregistré"); });
  $("#save").onclick = save;
  $("#back").onclick = () => go("briefs");
  $("#ren").onclick = () => run(async () => {
    const t = await askText("Nouveau titre", b.subject); if (!t) return;
    await invoke("update_brief", { id, subject: t }); brief(id);
  });
  $("#copy").onclick = () => run(async () => {
    const x = collect(), nl = String.fromCharCode(10);
    const md = [`# ${b.subject}`, "", "## Objectif", x.objective, "", "## Ordre du jour", ...x.agenda.map((a, i) => `${i + 1}. ${a.title}${a.minutes ? " (" + a.minutes + " min)" : ""}`), "", "## Questions", ...x.questions.map((q) => `- ${q}`), ""].join(nl);
    await navigator.clipboard.writeText(md); toast("Copié dans le presse-papiers");
  });
  $("#del").onclick = () => run(async () => { if (!(await ask("Placer cette préparation dans la corbeille ?", "Mettre à la corbeille"))) return; await invoke("update_brief", { id, trashed: true }); go("briefs"); });
  main.querySelectorAll("[data-m]").forEach((r) => r.onclick = () => go("meeting", r.dataset.m));
  main.onkeydown = (e) => { if ((e.ctrlKey || e.metaKey) && e.key === "s") { e.preventDefault(); save(); } };
}

// ── Glossaire ───────────────────────────────────────────────────────────
async function glossary() {
  const terms = (await run(() => invoke("get_glossary"))) || [];
  main.innerHTML = `<h1>Glossaire</h1><p class="sub">Sigles et termes métier propres à votre organisation.</p>
    <div class="card"><div id="rows"></div><div class="bar" style="margin:12px 0 0"><button class="btn" id="add">+ Terme</button><button class="btn pri" id="save">Enregistrer</button></div></div>`;
  const rows = $("#rows");
  const add = (t = "", d = "") => {
    const r = document.createElement("div"); r.className = "grid2"; r.style.marginBottom = "8px";
    r.innerHTML = `<input placeholder="Terme" value="${esc(t)}"><input placeholder="Définition" value="${esc(d)}">`;
    rows.appendChild(r);
  };
  terms.forEach((t) => add(t.term, t.definition));
  if (!terms.length) add();
  $("#add").onclick = () => add();
  $("#save").onclick = () => run(async () => {
    const ts = [...rows.children].map((r) => ({ term: r.children[0].value, definition: r.children[1].value }));
    await invoke("set_glossary", { terms: ts }); toast("Glossaire enregistré");
  });
}

// ── Corbeille ───────────────────────────────────────────────────────────
async function trash() {
  const ms = await invoke("list_meetings", { trashed: true });
  const bs = await invoke("list_briefs", { trashed: true });
  const item = (kind, id, t) => `<div class="card row"><span class="t">${esc(t)}</span><span class="pill">${kind}</span>
    <button class="btn sm" data-a="restore" data-k="${kind}" data-id="${id}">Restaurer</button><button class="btn sm dng" data-a="purge" data-k="${kind}" data-id="${id}">Supprimer</button></div>`;
  main.innerHTML = `<h1>Corbeille</h1><p class="sub">Les éléments supprimés définitivement ne sont pas récupérables.</p>` +
    (ms.length + bs.length ? ms.map((m) => item("Réunion", m.id, m.title)).join("") + bs.map((b) => item("Brief", b.id, b.subject)).join("") : `<div class="empty">La corbeille est vide.</div>`);
  main.onclick = (e) => {
    const b = e.target.closest("[data-a]");
    if (!b) return;
    run(async () => {
      const isM = b.dataset.k === "Réunion", id = b.dataset.id;
      if (b.dataset.a === "restore") await invoke(isM ? "update_meeting" : "update_brief", { id, trashed: false });
      else if (await ask("Supprimer définitivement ? Cette action est irréversible.")) await invoke(isM ? "purge_meeting" : "purge_brief", { id });
      else return;
      trash();
    });
  };
}

// ── Réglages ────────────────────────────────────────────────────────────
let setTab = "tr";
async function settings() {
  const s = (await run(() => invoke("get_settings"))) || {};
  const langs = [["fr", "Français"], ["en", "Anglais"], ["es", "Espagnol"], ["de", "Allemand"], ["it", "Italien"], ["auto", "Détection auto"]];
  main.innerHTML = `<h1>Réglages</h1><p class="sub">Tout est enregistré automatiquement, sauf la connexion API.</p>
    <div class="tabs" id="tabs" role="tablist">${[["tr", "Transcription"], ["ia", "Compte-rendu"], ["mod", "Modèles"], ["net", "Connexions"]].map(([k, v]) => `<button role="tab" data-tab="${k}" class="${setTab === k ? "on" : ""}">${v}</button>`).join("")}</div>
    <section class="pane" data-pane="tr" ${setTab === "tr" ? "" : "hidden"}>
      <div class="card"><h3>Langue et calcul</h3>
        <div class="grid2"><div><label style="margin-top:0">Langue parlée</label><select id="wl">${langs.map(([k, v]) => `<option value="${k}" ${s.whisper_lang === k ? "selected" : ""}>${v}</option>`).join("")}</select></div></div>
        <div id="wcalc"></div></div>
      <div class="card"><h3>Modèle de transcription</h3><div id="models"></div>
        <p class="meta" style="margin-top:10px">Les modèles « compressés » sont presque aussi précis que les complets, pour moitié moins d'espace. Plus le modèle est gros, plus c'est lent.</p></div>
    </section>
    <section class="pane" data-pane="ia" ${setTab === "ia" ? "" : "hidden"}>
      <div class="card"><h3>Calcul de la rédaction</h3><div id="ldev"></div></div>
      <div class="card"><h3>IA de rédaction</h3>
        <p class="meta" style="margin-bottom:10px">Un modèle installé sur cet ordinateur rédige le compte-rendu, sans rien envoyer en ligne. Sans modèle local ni API, seul un résumé simplifié est produit.</p>
        <div id="llms"></div></div>
    </section>
    <section class="pane" data-pane="mod" ${setTab === "mod" ? "" : "hidden"}>
      <div class="card"><h3>Utiliser des modèles déjà présents</h3>
        <p class="meta" style="margin-bottom:6px">Verbatim a trouvé des modèles d'IA déjà présents sur cet ordinateur (par exemple ceux d'Ollama). Cochez ceux à utiliser : ils sont <b>copiés</b> dans le dossier de Verbatim, les originaux ne sont ni déplacés ni modifiés.</p>
        <p class="meta" style="margin-bottom:10px">Dossier de destination : <code id="mdir"></code></p>
        <div id="found"></div>
        <div class="bar" style="margin:10px 0 0"><button class="btn sm" id="f-dir">Choisir un dossier…</button><button class="btn sm" id="f-file">Choisir un fichier…</button><button class="btn sm" id="f-rescan">Rechercher</button></div></div>
      <div class="card"><h3>Modèles personnalisés installés</h3><div id="ggufs"></div></div>
    </section>
    <section class="pane" data-pane="net" ${setTab === "net" ? "" : "hidden"}>
      <div class="card"><h3>Ollama</h3>
        <p class="meta" style="margin-bottom:10px">Si Ollama est installé, Verbatim peut l'utiliser pour rédiger avec n'importe lequel de vos modèles.</p>
        <div id="ollama"></div></div>
      <div class="card"><h3>Mises à jour</h3>
        <p class="meta" style="margin-bottom:8px">Verbatim cherche au démarrage la dernière version publiée sur GitHub. L'installation ne se lance jamais pendant un enregistrement, une transcription ou une rédaction.</p>
        <label class="chk"><input type="checkbox" id="au" ${s.auto_update !== false ? "checked" : ""}> Installer automatiquement les nouvelles versions</label>
        <div class="bar" style="margin:12px 0 0"><button class="btn" id="au-chk">Rechercher maintenant</button><span class="meta" id="au-msg"></span></div></div>
      <div class="card"><h3>API distante (facultatif)</h3><p class="meta" style="margin-bottom:8px">Utilisée si aucun modèle local n'est sélectionné.</p>
        <label style="margin-top:0">URL de l'API</label><input id="u" placeholder="https://api.exemple.com/v1" value="${esc(s.llm_base_url)}">
        <label>Clé d'API</label><input id="k" type="password" value="${esc(s.llm_api_key)}"><label>Modèle</label><input id="m" placeholder="gpt-4o-mini" value="${esc(s.llm_model)}">
        <div class="bar" style="margin:14px 0 0"><button class="btn pri" id="save">Enregistrer</button></div></div>
    </section>`;
  $("#tabs").onclick = (e) => {
    const b = e.target.closest("[data-tab]"); if (!b) return;
    setTab = b.dataset.tab;
    document.querySelectorAll("#tabs [data-tab]").forEach((x) => x.classList.toggle("on", x === b));
    document.querySelectorAll(".pane").forEach((x) => { x.hidden = x.dataset.pane !== setTab; });
  };
  let curModel = s.whisper_model, curLlm = s.llm_local || "", curWdev = s.whisper_device || "auto", curDev = !s.llm_device || s.llm_device === "cpu" ? "auto" : s.llm_device, curRocm = s.rocm_root || "";
  const saveAll = async () => invoke("set_settings", { settings: { llm_base_url: $("#u").value.trim(), llm_api_key: $("#k").value, llm_model: $("#m").value.trim(), whisper_model: curModel, whisper_lang: $("#wl").value, llm_local: curLlm, whisper_device: curWdev, llm_device: curDev, rocm_root: curRocm, auto_update: $("#au").checked } });
  const renderModels = async () => {
    const st = await invoke("engine_status"), d = st.download;
    $("#models").innerHTML = st.models.map((m) => {
      const busy = d.active && d.model === m.name;
      const pct = m.partial ? Math.round((m.partial / m.bytes) * 100) : 0;
      const state = m.installed ? `<span class="pill ok">Installé</span>` : busy ? `<span class="pill">Téléchargement…</span>` : m.partial ? `<span class="pill warn">Interrompu ${pct} %</span>` : "";
      const act = m.installed
        ? `<button class="btn sm dng" data-del="${m.name}">Supprimer</button>`
        : busy ? "" : `<button class="btn sm" data-get="${m.name}">${m.partial ? "Reprendre" : "Télécharger"}</button>`;
      return `<div class="card mrow ${m.selected ? "sel" : ""}"><label class="chk" style="flex:1"><input type="radio" name="wm" value="${m.name}" ${m.selected ? "checked" : ""} ${m.installed ? "" : "disabled"}>
        <span><b>${esc(m.label)}</b> <span class="meta">· ${fmtSize(m.bytes)}</span><br><span class="meta">${esc(m.desc)}</span><br><span class="meta" style="word-break:break-all">${m.installed ? "Emplacement" : "Sera installé dans"} : <code>${esc(m.path)}</code></span>
        ${busy ? `<div class="prog" style="width:100%;margin-top:6px"><b data-dlbar style="width:${dlPct(d)}%"></b></div><span class="meta" data-dlmsg>${esc(dlLabel(d))}</span>` : ""}</span></label>${state}${act}</div>`;
    }).join("");
    const g = st.gpu, gbusy = d.active && d.phase === "binary-gpu";
    const anyGpu = g.cuda || g.rocm, gcuda = !g.rocm && g.cuda;
    const gname = g.rocm ? "AMD Radeon · ROCm" : g.cuda ? "NVIDIA · CUDA" : "";
    const wstate = curWdev === "cpu" ? `<span class="pill">CPU</span>` : anyGpu && (g.rocm || g.installed) ? `<span class="pill ok">GPU actif · ${gname}</span>` : `<span class="pill warn">CPU (GPU indisponible)</span>`;
    $("#wcalc").innerHTML = `<label>Calcul de la transcription ${wstate}</label>
      <div class="seg2">
        <label class="opt ${curWdev !== "cpu" ? "sel" : ""}"><input type="radio" name="wd" value="auto" ${curWdev !== "cpu" ? "checked" : ""}><span><b>Automatique</b><br><span class="meta">${anyGpu ? "GPU (" + gname + "), repli sur le CPU en cas de problème" : "GPU si disponible, sinon CPU"}</span></span></label>
        <label class="opt ${curWdev === "cpu" ? "sel" : ""}"><input type="radio" name="wd" value="cpu" ${curWdev === "cpu" ? "checked" : ""}><span><b>CPU uniquement</b><br><span class="meta">Processeur, compatible partout</span></span></label></div>
      ${anyGpu ? "" : `<p class="meta" style="margin-top:8px;color:var(--danger)">Aucun GPU utilisable${g.names.length ? " (" + esc(g.names.join(", ")) + ")" : ""} : il faut une carte NVIDIA (CUDA) ou AMD RDNA3 (ROCm).</p>`}
      <details style="margin-top:10px"><summary class="meta">Moteurs et emplacements</summary>
        <p class="meta" style="word-break:break-all;margin-top:6px">CPU : <code>${esc(st.binary_path)}</code> ${st.binary ? "" : "(installé avec le premier modèle)"}</p>
        ${g.rocm_engine ? `<p class="meta" style="word-break:break-all">ROCm : <code>${esc(g.rocm_path)}</code></p>` : ""}
        ${gcuda ? `<p class="meta" style="word-break:break-all">CUDA : <code>${esc(g.path)}</code> ${g.installed ? "" : `<button class="btn sm" id="g-dl">Télécharger (${fmtSize(g.bytes)})</button>`}</p>${gbusy ? `<div class="prog" style="width:100%;margin-top:6px"><b data-dlbar style="width:${dlPct(d)}%"></b></div><span class="meta" data-dlmsg>${esc(dlLabel(d))}</span>` : ""}` : ""}
      </details>`;
    $("#wcalc").querySelectorAll("[name=wd]").forEach((r) => r.onchange = () => run(async () => { curWdev = r.value; await saveAll(); renderModels(); }));
    const gd = $("#g-dl"); if (gd) gd.onclick = () => run(async () => { await invoke("engine_install_gpu"); renderModels(); watchDownload(); });
    const llm = st.llm;
    const lrows = [`<div class="card mrow ${curLlm === "" ? "sel" : ""}"><label class="chk" style="flex:1"><input type="radio" name="lm" value="" ${curLlm === "" ? "checked" : ""}>
      <span><b>Aucune IA locale</b><br><span class="meta">Utiliser l'API distante si elle est configurée, sinon un résumé simplifié</span></span></label></div>`].concat(llm.models.map((m) => {
      const busy = d.active && d.model === m.name;
      const pct = m.partial ? Math.round((m.partial / m.bytes) * 100) : 0;
      const state = m.installed ? `<span class="pill ok">Installé</span>` : busy ? `<span class="pill">Téléchargement…</span>` : m.partial ? `<span class="pill warn">Interrompu ${pct} %</span>` : "";
      const act = m.installed ? `<button class="btn sm dng" data-ldel="${m.name}">Supprimer</button>`
        : busy ? "" : `<button class="btn sm" data-lget="${m.name}">${m.partial ? "Reprendre" : "Télécharger"}</button>`;
      return `<div class="card mrow ${curLlm === m.name ? "sel" : ""}"><label class="chk" style="flex:1"><input type="radio" name="lm" value="${m.name}" ${curLlm === m.name ? "checked" : ""} ${m.installed ? "" : "disabled"}>
        <span><b>${esc(m.label)}</b> <span class="meta">· ${fmtSize(m.bytes)}</span><br><span class="meta">${esc(m.desc)}</span><br><span class="meta" style="word-break:break-all">${m.installed ? "Emplacement" : "Sera installé dans"} : <code>${esc(m.path)}</code></span>
        ${busy ? `<div class="prog" style="width:100%;margin-top:6px"><b data-dlbar style="width:${dlPct(d)}%"></b></div><span class="meta" data-dlmsg>${esc(dlLabel(d))}</span>` : ""}</span></label>${state}${act}</div>`;
    }));
    $("#llms").innerHTML = lrows.join("") + `<p class="meta" style="margin-top:6px">Moteur d'IA locale (llama.cpp) : ${llm.binary ? "installé" : "non installé (téléchargé avec le premier modèle)"} · <code>${esc(llm.binary_path)}</code></p><p class="meta" style="margin-top:6px">Le modèle n'est chargé en mémoire (4 à 24 Go selon le modèle) que pendant la rédaction.</p>`;
    $("#llms").querySelectorAll("[name=lm]").forEach((r) => r.onchange = () => run(async () => { curLlm = r.value; await saveAll(); renderModels(); if (typeof renderOllama === "function") renderOllama(); if (typeof renderGguf === "function") renderGguf(); }));
    $("#llms").querySelectorAll("[data-lget]").forEach((b) => b.onclick = () => run(async () => { await invoke("llm_install", { model: b.dataset.lget }); renderModels(); watchDownload(); }));
    $("#llms").querySelectorAll("[data-ldel]").forEach((b) => b.onclick = () => run(async () => { if (await ask("Supprimer ce modèle du disque ?")) { await invoke("llm_remove", { model: b.dataset.ldel }); curLlm = (await invoke("get_settings")).llm_local || ""; renderModels(); renderGguf(); } }));
    const eng = llm.engines, gbusyL = d.active && d.phase === "llm-binary";
    const hasNv = llm.gpus.some((n) => /nvidia/i.test(n)), hasAmd = llm.gpus.some((n) => /amd|radeon/i.test(n) && !/graphics/i.test(n));
    const cudaOk = eng.cuda.installed && hasNv, rocmOk = eng.rocm.runtime && eng.rocm.installed;
    const gpuOk = cudaOk || rocmOk || eng.vulkan.installed;
    const gpuName = cudaOk ? "CUDA" : rocmOk ? "ROCm" : eng.vulkan.installed ? "Vulkan" : "";
    const lstate = curDev === "cpu_only" ? `<span class="pill">CPU</span>` : gpuOk ? `<span class="pill ok">GPU actif · ${gpuName}</span>` : `<span class="pill warn">CPU (moteur GPU non installé)</span>`;
    const devRow = (id, label, desc, extra) => {
      const e = eng[id], need = !e.installed;
      return `<div class="mrow card ${curDev === id ? "sel" : ""}" style="margin-bottom:6px"><label class="chk" style="flex:1"><input type="radio" name="ld" value="${id}" ${curDev === id ? "checked" : ""}>
        <span><b>${label}</b> <span class="meta">${desc}</span><br><span class="meta" style="word-break:break-all">${e.installed ? "Installé" : "À installer"} : <code>${esc(e.path)}</code></span>${extra || ""}</span></label>
        ${need ? (gbusyL ? "" : `<button class="btn sm" data-ldev-get="${id}">Télécharger (${fmtSize(e.bytes)})</button>`) : ""}</div>`;
    };
    const rocmExtra = eng.rocm.runtime
      ? `<br><span class="meta" style="word-break:break-all">Runtime ROCm : <code>${esc(eng.rocm.runtime)}</code></span>`
      : `<br><span class="meta" style="color:var(--danger)">Runtime ROCm introuvable.</span>`;
    $("#ldev").innerHTML = `<label style="margin-top:0">Calcul de la rédaction ${lstate}</label>
      <div class="seg2">
        <label class="opt ${curDev !== "cpu_only" ? "sel" : ""}"><input type="radio" name="ld" value="auto" ${curDev !== "cpu_only" ? "checked" : ""}><span><b>Automatique</b><br><span class="meta">GPU (${hasNv ? "CUDA" : "ROCm"}, sinon Vulkan), repli sur le CPU${llm.gpus.length ? " · " + esc(llm.gpus.join(", ")) : ""}</span></span></label>
        <label class="opt ${curDev === "cpu_only" ? "sel" : ""}"><input type="radio" name="ld" value="cpu_only" ${curDev === "cpu_only" ? "checked" : ""}><span><b>CPU uniquement</b><br><span class="meta">Processeur, compatible partout (beaucoup plus lent)</span></span></label></div>
      ${gpuOk || curDev === "cpu_only" ? "" : `<p class="meta" style="margin-top:8px">Aucun moteur GPU installé. <button class="btn sm" data-ldev-get="${hasNv ? "cuda" : "vulkan"}">Télécharger le moteur ${hasNv ? "CUDA" : "Vulkan"} (${fmtSize((hasNv ? eng.cuda : eng.vulkan).bytes)})</button></p>`}
      ${gbusyL ? `<div class="prog" style="width:100%;margin-top:6px"><b data-dlbar style="width:${dlPct(d)}%"></b></div><span class="meta" data-dlmsg>${esc(dlLabel(d))}</span>` : ""}
      <details style="margin-top:10px"><summary class="meta">Moteurs et réglages avancés</summary>
        <p class="meta" style="margin:6px 0;word-break:break-all">CPU : <code>${esc(eng.cpu.path)}</code> ${eng.cpu.installed ? "" : "(installé avec le premier modèle)"}</p>
        ${devRow("vulkan", "Forcer Vulkan", "(AMD, NVIDIA, Intel)")}
        ${hasNv || eng.cuda.installed || curDev === "cuda" ? devRow("cuda", "Forcer CUDA", "(NVIDIA ; le plus rapide sur carte NVIDIA)") : ""}
        ${hasAmd || eng.rocm.installed || curDev === "rocm" ? devRow("rocm", "Forcer ROCm", "(AMD RDNA3 ; lecture des longs textes plus rapide)", rocmExtra) : ""}
        ${hasAmd || eng.rocm.installed || curDev === "rocm" ? `<div class="bar" style="margin:6px 0 0"><button class="btn sm" id="rocm-pick">Choisir le dossier du runtime ROCm…</button>${curRocm ? `<button class="btn sm" id="rocm-auto">Détection automatique</button>` : ""}</div>` : ""}
      </details>`;
    $("#ldev").querySelectorAll("[name=ld]").forEach((r) => r.onchange = () => run(async () => { curDev = r.value; await saveAll(); renderModels(); }));
    $("#ldev").querySelectorAll("[data-ldev-get]").forEach((b) => b.onclick = () => run(async () => { await invoke("llm_install_engine", { device: b.dataset.ldevGet }); renderModels(); watchDownload(); }));
    const rp = $("#rocm-pick"); if (rp) rp.onclick = () => run(async () => { const f = await dialog.open({ directory: true }); if (!f) return; curRocm = await invoke("llm_rocm_check", { path: f }); await saveAll(); toast("Runtime ROCm trouvé"); renderModels(); });
    const ra = $("#rocm-auto"); if (ra) ra.onclick = () => run(async () => { curRocm = ""; await saveAll(); renderModels(); });
    if (d.active) watchDownload();
    $("#models").querySelectorAll("[name=wm]").forEach((r) => r.onchange = () => run(async () => { curModel = r.value; await saveAll(); renderModels(); }));
    $("#models").querySelectorAll("[data-get]").forEach((b) => b.onclick = () => run(async () => { curModel = b.dataset.get; await saveAll(); await invoke("engine_install", { model: curModel }); renderModels(); watchDownload(); }));
    $("#models").querySelectorAll("[data-del]").forEach((b) => b.onclick = () => run(async () => { if (await ask("Supprimer ce modèle du disque ?")) { await invoke("engine_remove", { model: b.dataset.del }); renderModels(); } }));
  };
  const renderOllama = async () => {
    const box = $("#ollama"); if (!box) return;
    box.innerHTML = `<p class="meta">Recherche d'Ollama…</p>`;
    const o = await invoke("ollama_status");
    if (!o.installed) { box.innerHTML = `<p class="meta">Ollama n'est pas installé (ollama.com).</p>`; return; }
    if (!o.running) {
      box.innerHTML = `<p class="meta" style="margin-bottom:8px">Ollama est installé (${esc(o.exe)}) mais n'est pas démarré.</p><button class="btn" id="o-start">Démarrer Ollama</button>`;
      $("#o-start").onclick = () => run(async () => { $("#o-start").disabled = true; toast("Démarrage d'Ollama…"); await invoke("ollama_start"); renderOllama(); });
      return;
    }
    if (!o.models.length) { box.innerHTML = `<p class="meta">Ollama fonctionne, mais aucun modèle n'est téléchargé. Dans un terminal : <code>ollama pull qwen3:8b</code></p>`; return; }
    box.innerHTML = o.models.map((m) => {
      const val = "ollama:" + m.name, big = m.bytes > 40 * 1073741824;
      return `<div class="card mrow ${curLlm === val ? "sel" : ""}"><label class="chk" style="flex:1"><input type="radio" name="lm" value="${esc(val)}" ${curLlm === val ? "checked" : ""}>
        <span><b>${esc(m.name)}</b> <span class="meta">· ${fmtSize(m.bytes)}${m.params ? " · " + esc(m.params) : ""}${m.quant ? " · " + esc(m.quant) : ""}</span>
        ${big ? `<br><span class="meta" style="color:var(--danger)">Très volumineux : sans assez de mémoire (RAM + carte graphique), la rédaction sera extrêmement lente.</span>` : ""}</span></label></div>`;
    }).join("");
    box.querySelectorAll("[name=lm]").forEach((r) => r.onchange = () => run(async () => { curLlm = r.value; await saveAll(); renderModels(); renderOllama(); renderGguf(); }));
  };
  let foundExtra = null;
  const renderGguf = async () => {
    const box = $("#ggufs"); if (!box) return;
    $("#mdir").textContent = (await invoke("models_dir_info")).path;
    const [list, st] = await Promise.all([invoke("local_gguf_list"), invoke("engine_status")]);
    const d = st.download;
    box.innerHTML = (list.length ? `<label style="margin-top:0">Modèles personnalisés installés</label>` : "") + list.map((m) => {
      const val = "gguf:" + m.path;
      return `<div class="card mrow ${curLlm === val ? "sel" : ""}"><label class="chk" style="flex:1"><input type="radio" name="lm" value="${esc(val)}" ${curLlm === val ? "checked" : ""}>
        <span><b>${esc(m.name)}</b> <span class="meta">· ${fmtSize(m.bytes)}</span><br><span class="meta" style="word-break:break-all">Emplacement : <code>${esc(m.path)}</code></span>${m.bytes > 12 * 1073741824 ? `<br><span class="meta">Gros modèle : chargement long, rédaction lente sans carte graphique.</span>` : ""}</span></label>
        <button class="btn sm dng" data-gdel="${esc(m.path)}">Supprimer</button></div>`;
    }).join("") + (list.length && !st.llm.binary ? `<p class="meta" style="margin-top:6px">Le moteur d'IA locale n'est pas installé. <button class="btn sm" id="g-eng">Télécharger le moteur (~20 Mo)</button></p>` : "");
    box.querySelectorAll("[name=lm]").forEach((r) => r.onchange = () => run(async () => { curLlm = r.value; await saveAll(); renderModels(); renderOllama(); renderGguf(); }));
    box.querySelectorAll("[data-gdel]").forEach((b) => b.onclick = () => run(async () => { if (await ask("Supprimer ce modèle du disque ?")) { await invoke("llm_remove_file", { path: b.dataset.gdel }); curLlm = (await invoke("get_settings")).llm_local || ""; renderModels(); renderGguf(); } }));
    const ge = $("#g-eng"); if (ge) ge.onclick = () => run(async () => { await invoke("llm_install_engine", { device: "cpu" }); renderGguf(); watchDownload(); });
  };
  const renderFound = async () => {
    const box = $("#found"); if (!box) return;
    box.innerHTML = `<p class="meta">Recherche…</p>`;
    const [items, st] = await Promise.all([invoke("models_scan", { extra: foundExtra }), invoke("engine_status")]);
    const d = st.download;
    if (d.active && d.phase === "copy") {
      box.innerHTML = `<div class="prog" style="width:100%;margin:6px 0"><b data-dlbar style="width:${dlPct(d)}%"></b></div><span class="meta" data-dlmsg>${esc(dlLabel(d))}</span>`;
      watchDownload(); return;
    }
    if (!items.length) { box.innerHTML = `<p class="meta">Aucun modèle existant à copier${foundExtra ? " à cet emplacement" : ""}.</p>`; return; }
    box.innerHTML = items.map((i, n) => `<label class="chk" style="margin:4px 0"><input type="checkbox" data-fi="${n}">
      <span><b>${esc(i.name)}</b> <span class="meta">· ${fmtSize(i.bytes)} · ${esc(i.source)}</span></span></label>`).join("") +
      `<button class="btn" id="f-copy" style="margin-top:8px">Copier la sélection dans Verbatim</button>`;
    $("#f-copy").onclick = () => run(async () => {
      const paths = [...box.querySelectorAll("[data-fi]")].filter((c) => c.checked).map((c) => items[+c.dataset.fi].src);
      if (!paths.length) return;
      await invoke("models_import", { paths, extra: foundExtra });
      renderFound(); watchDownload();
    });
  };
  $("#f-dir").onclick = () => run(async () => { const f = await dialog.open({ directory: true }); if (f) { foundExtra = f; renderFound(); } });
  $("#f-file").onclick = () => run(async () => { const f = await dialog.open({ filters: [{ name: "Modèles", extensions: ["gguf", "bin"] }] }); if (f) { foundExtra = f; renderFound(); } });
  $("#f-rescan").onclick = () => { foundExtra = null; renderFound(); };
  engineHook = () => view === "settings" && renderModels();
  $("#wl").onchange = () => run(saveAll);
  $("#au").onchange = () => run(saveAll);
  $("#au-chk").onclick = () => run(async () => {
    const msg = $("#au-msg"); msg.textContent = "Recherche…";
    const u = await invoke("update_check");
    if (!u.available) { msg.textContent = "Vous avez la dernière version (v" + u.current + ")."; return; }
    msg.textContent = "Version " + u.latest + " disponible.";
    updateBanner(u);
  });
  renderModels(); renderOllama(); renderGguf(); renderFound();
  $("#save").onclick = () => run(async () => { await saveAll(); toast("Réglages enregistrés"); });
}

try { window.__TAURI__.app.getVersion().then((v) => { $("#ver").textContent = "Verbatim v" + v; }); } catch (_) {}

// ── Mise à jour (dernière release GitHub) ───────────────────────────────
async function updateInstall(u) {
  const b = $("#upd"); if (b) b.remove();
  toast("Installation de la version " + u.latest + " : l'application va redémarrer…");
  try { await invoke("update_install", { url: u.url, sha: u.sha }); }
  catch (e) { toast(String(e)); updateBanner(u); }
}
function updateBanner(u) {
  let b = $("#upd");
  if (!b) { b = document.createElement("div"); b.id = "upd"; document.body.appendChild(b); }
  b.innerHTML = `<span><b>Version ${esc(u.latest)} disponible</b></span><button class="btn pri sm" id="upd-go">Mettre à jour</button><button class="btn sm" id="upd-x">Plus tard</button>`;
  $("#upd-go").onclick = () => updateInstall(u);
  $("#upd-x").onclick = () => b.remove();
}
async function updateStartup() {
  try {
    const u = await invoke("update_check"); if (!u.available) return;
    const [cfg, rec, ms] = await Promise.all([invoke("get_settings"), invoke("rec_status"), invoke("list_meetings", { trashed: false })]);
    // jamais pendant un enregistrement, une transcription ou une rédaction : on propose seulement
    const busy = rec.recording || ms.some((m) => !["ready", "error", "needs_engine"].includes(m.stage) || m.report_state === "generating");
    if (cfg.auto_update !== false && !busy) return updateInstall(u);
    updateBanner(u);
  } catch (_) {}
}
setTimeout(updateStartup, 3000);
go("meetings");
$("#theme").onclick = () => {
  const t = themeNow() === "dark" ? "light" : "dark";
  document.documentElement.dataset.theme = t;
  try { localStorage.setItem("theme", t); } catch (_) {}
};
