// Thème : choix mémorisé, sinon celui du système.
const themeNow = () => document.documentElement.dataset.theme || (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
// Thème clair par défaut ; « auto » (choix explicite dans Réglages › Général) suit le système.
try { const t = localStorage.getItem("theme") || "light"; if (t !== "auto") document.documentElement.dataset.theme = t; } catch (_) { document.documentElement.dataset.theme = "light"; }
// Palettes : [clé, nom, teinte de l'accent, teinte des fonds, saturation des fonds, options]. « » = thème d'origine.
// Options : s = vivacité de l'accent, L = luminosités de l'accent [clair : texte, bouton, survol ; sombre : texte, bouton, survol].
const PALETTES = [["", "Verbatim", 231, 0, 0], ["midnight", "Minuit", 215, 222, 38], ["slate", "Ardoise", 205, 210, 12], ["ocean", "Océan", 182, 192, 34], ["forest", "Forêt", 150, 150, 24],
  ["plum", "Prune", 285, 280, 26], ["rose", "Rose", 335, 335, 22], ["amber", "Ambre", 32, 30, 24], ["graphite", "Graphite", 0, 0, 0, { s: 0, L: [25, 25, 15, 80, 42, 52] }],
  ["kawaii", "Kawaii", 330, 330, 60, { s: 1.2, L: [50, 55, 46, 78, 58, 66] }], ["gamer", "Gamer", 140, 265, 45, { s: 1.5, L: [30, 30, 24, 58, 30, 38] }]];
function applyAccent() {
  const st = document.documentElement.style, root = document.documentElement;
  let k = ""; try { k = localStorage.getItem("palette") || ""; } catch (_) {}
  const pal = PALETTES.find((x) => x[0] === k);
  const props = ["--bg", "--panel", "--panel-2", "--ink", "--mute", "--line", "--line-2", "--brand", "--brand-2", "--brand-btn", "--brand-btn-h", "--brand-soft", "--side", "--side-2"];
  if (k) root.dataset.palette = k; else delete root.dataset.palette;
  root.dataset.mode = themeNow();
  if (!pal || !pal[0]) { props.forEach((x) => st.removeProperty(x)); return; }
  const [, , h, nh, ns, o = {}] = pal, dark = themeNow() === "dark", s = o.s ?? 1, L = o.L || [44, 44, 36, 76, 52, 62];
  const sat = (x) => Math.min(100, Math.round(x * s));
  const v = dark
    ? [`hsl(${nh} ${ns}% 8%)`, `hsl(${nh} ${ns}% 11%)`, `hsl(${nh} ${ns}% 14%)`, `hsl(${nh} ${ns * 0.5}% 92%)`, `hsl(${nh} ${ns * 0.4}% 63%)`, `hsl(${nh} ${ns}% 18%)`, `hsl(${nh} ${ns}% 25%)`,
      `hsl(${h} ${sat(85)}% ${L[3]}%)`, `hsl(${h} ${sat(85)}% ${L[3]}%)`, `hsl(${h} ${sat(60)}% ${L[4]}%)`, `hsl(${h} ${sat(70)}% ${L[5]}%)`, `hsl(${h} ${sat(85)}% 70% / .13)`,
      `hsl(${nh} ${ns}% 5%)`, `hsl(${nh} ${ns}% 5%)`]
    : [`hsl(${nh} ${ns}% 96%)`, `#fff`, `hsl(${nh} ${ns}% 98%)`, `hsl(${nh} ${ns + 10}% 10%)`, `hsl(${nh} ${ns * 0.4}% 42%)`, `hsl(${nh} ${ns}% 91%)`, `hsl(${nh} ${ns}% 84%)`,
      `hsl(${h} ${sat(62)}% ${L[0]}%)`, `hsl(${h} ${sat(62)}% ${L[0]}%)`, `hsl(${h} ${sat(62)}% ${L[1]}%)`, `hsl(${h} ${sat(62)}% ${L[2]}%)`, `hsl(${h} ${sat(62)}% 44% / .08)`,
      `hsl(${nh} ${ns + 10}% 11%)`, `hsl(${nh} ${ns + 10}% 11%)`];
  props.forEach((x, i) => st.setProperty(x, v[i]));
}
applyAccent();
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
const fmtDate = (d) => new Date(d).toLocaleString(I18N.locale(), { dateStyle: "medium", timeStyle: "short" });
const fmtT = (s) => `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(Math.floor(s % 60)).padStart(2, "0")}`;
const { listen } = window.__TAURI__.event;
const fmtSize = (n) => n >= 1073741824 ? (n / 1073741824).toFixed(2) + " Go" : (n / 1048576).toFixed(0) + " Mo";
const dlPct = (d) => (d.total ? Math.min(100, Math.round((d.done / d.total) * 100)) : 0);
const dlLabel = (d) => `${{ binary: "Moteur", vad: "Détection de parole", model: "Modèle", "llm-binary": "Moteur IA", "llm-model": "Modèle IA", copy: "Copie", "binary-gpu": "Moteur GPU", diarization: "Séparation des voix", }[d.phase] || "Téléchargement"} : ${dlPct(d)} % (${fmtSize(d.done)} / ${fmtSize(d.total)})`;

// L'état du téléchargement vit côté Rust : il survit aux changements de page.
// Cette fonction se (re)branche dessus à chaque affichage concerné.
let dlTimer = null, engineHook = null;
// Bouton « Arrêter » à côté de chaque message de progression : annule le téléchargement et supprime la partie déjà reçue.
function dlStopButtons() {
  document.querySelectorAll("[data-dlmsg]").forEach((e) => {
    if (e.parentElement.querySelector("[data-dlstop]")) return;
    const b = document.createElement("button");
    b.className = "btn sm dng"; b.dataset.dlstop = "1"; b.style.marginLeft = "10px"; b.textContent = "Arrêter"; b.title = "Annule le téléchargement et supprime la partie déjà reçue";
    b.onclick = async (ev) => { ev.preventDefault(); ev.stopPropagation(); b.disabled = true; b.textContent = "Arrêt…"; try { await invoke("download_cancel"); } catch (x) {} };
    e.after(b);
  });
}
function watchDownload() {
  if (dlTimer) return;
  setTimeout(dlStopButtons, 150);
  dlTimer = setInterval(async () => {
    const st = await invoke("engine_status");
    const d = st.download;
    dlStopButtons();
    document.querySelectorAll("[data-dlmsg]").forEach((e) => { e.textContent = d.active ? dlLabel(d) : ""; });
    document.querySelectorAll("[data-dlbar]").forEach((e) => { e.style.width = dlPct(d) + "%"; });
    if (!d.active) {
      clearInterval(dlTimer); dlTimer = null;
      if (d.error === "Téléchargement annulé.") toast("Téléchargement arrêté — vous pourrez le reprendre.");
      else if (d.error) toast("Téléchargement interrompu : " + d.error + " — vous pourrez le reprendre.");
      else toast(d.phase === "diarization" ? "Modèles de séparation des intervenants prêts" : "Moteur Whisper prêt");
      if (engineHook) engineHook();
    }
  }, 700);
}
listen("engine-done", () => { if (engineHook) engineHook(); });
listen("diarize-done", (e) => { if (!e.payload.ok) toast("Séparation des intervenants impossible : " + e.payload.error); });

// Glisser-déposer : des fichiers déposés sur la fenêtre (quelle que soit la page) sont importés.
const DROP_AUDIO = ["mp3", "wav", "m4a", "ogg", "flac", "aac", "webm", "mp4", "opus"], DROP_TEXT = ["txt", "srt", "vtt", "md"];
const dropZone = document.createElement("div");
dropZone.id = "dropzone"; dropZone.hidden = true;
dropZone.innerHTML = `<div>Déposez vos fichiers ici pour les importer</div>`;
document.body.appendChild(dropZone);
listen("tauri://drag-enter", () => { dropZone.hidden = false; });
listen("tauri://drag-leave", () => { dropZone.hidden = true; });
listen("tauri://drag-drop", (e) => {
  dropZone.hidden = true;
  const ext = (p) => (p.split(".").pop() || "").toLowerCase();
  const paths = (e.payload && e.payload.paths) || [];
  const audio = paths.filter((p) => DROP_AUDIO.includes(ext(p))), text = paths.filter((p) => DROP_TEXT.includes(ext(p)));
  if (!audio.length && !text.length) return toast("Fichier non géré par Verbatim.");
  run(async () => {
    if (audio.length) {
      const n = await invoke("import_audio", { paths: audio });
      toast(`${n} fichier(s) importé(s)`);
    }
    // une transcription déposée s'ouvre directement (seulement si aucun audio n'est importé en même temps)
    if (text.length && !audio.length) { const id = await invoke("import_transcript", { path: text[0] }); return go("meeting", id); }
    go("meetings");
  });
});

// Bandeaux « non installé / téléchargement » : Whisper pour le modèle choisi ; sur la liste des réunions (`all`), aussi la
// séparation des intervenants et le modèle de compte-rendu, une fois Whisper installé.
async function engineBox(all = false) {
  const [st, cfg] = await Promise.all([invoke("engine_status"), invoke("get_settings")]);
  const m = st.models.find((x) => x.name === cfg.whisper_model) || st.models[0];
  const d = st.download;
  if (d.active) {
    watchDownload();
    return `<div class="card"><b>Téléchargement en cours…</b><div class="prog" style="width:100%;margin:8px 0"><b data-dlbar style="width:${dlPct(d)}%"></b></div>
      <span class="meta" data-dlmsg>${esc(dlLabel(d))}</span><p class="meta" style="margin-top:6px">Vous pouvez changer de page ou fermer l'app : le téléchargement reprendra là où il s'est arrêté. « Arrêter » l'annule et supprime la partie déjà reçue.</p></div>`;
  }
  if (st.binary && m.installed) return all ? extraBoxes(st, cfg) : "";
  const pct = m.partial ? Math.round((m.partial / m.bytes) * 100) : 0;
  return `<div class="card"><b>${m.partial ? "Téléchargement interrompu" : "Moteur de transcription non installé"}</b>
    <p class="meta" style="margin:4px 0 10px">Whisper s'exécute en local, l'audio ne quitte jamais cet ordinateur. Modèle « ${esc(m.label)} » : ${fmtSize(m.bytes)}${m.partial ? ` — déjà ${pct} % téléchargés` : ""}. Téléchargé une seule fois, puis conservé.</p>
    <button class="btn pri" id="eng-go">${m.partial ? "Reprendre le téléchargement" : "Installer Whisper"}</button>
    ${d.error ? `<span class="meta" style="color:var(--danger)"> ${esc(d.error)}</span>` : ""}</div>`;
}
// Le compte-rendu est prêt avec une API configurée, un modèle local importé ou un modèle du catalogue installé et choisi.
function reportReady(st, cfg) {
  if ((cfg.llm_base_url || "").trim()) return true;
  const sel = cfg.llm_local || "";
  if (!sel) return false;
  if (sel.startsWith("gguf:") || sel.startsWith("ollama:")) return true;
  return st.llm.models.some((x) => x.name === sel && x.installed);
}
function extraBoxes(st, cfg) {
  let h = "";
  if (!st.diarization.installed) {
    h += `<div class="card"><b>Séparation des intervenants non installée</b>
      <p class="meta" style="margin:4px 0 10px">Repère qui parle et attribue chaque passage à un intervenant. Calcul local : l'audio ne quitte jamais cet ordinateur. Téléchargé une seule fois, puis conservé : <span>${fmtSize(st.diarization.bytes)}</span></p>
      <button class="btn pri" id="eng-dia">Installer la séparation des intervenants</button></div>`;
  }
  if (!reportReady(st, cfg)) {
    const min = Math.min(...st.llm.models.map((x) => x.bytes));
    h += `<div class="card"><b>Modèle de compte-rendu non installé</b>
      <p class="meta" style="margin:4px 0 10px">Le compte-rendu (résumé, décisions, actions) est rédigé par une IA qui s'exécute sur cet ordinateur. La transcription fonctionne sans. Modèles à partir de <span>${fmtSize(min)}</span></p>
      <button class="btn pri" id="eng-llm">Choisir un modèle</button></div>`;
  }
  return h;
}
function bindEngine(model) {
  const b = document.getElementById("eng-go");
  if (b) b.onclick = () => run(async () => {
    b.disabled = true; await invoke("engine_install", { model });
    if (engineHook) engineHook(); watchDownload();
  });
  const d = document.getElementById("eng-dia");
  if (d) d.onclick = () => run(async () => {
    d.disabled = true; await invoke("diarize_install");
    if (engineHook) engineHook(); watchDownload();
  });
  const l = document.getElementById("eng-llm");
  if (l) l.onclick = () => { setTab = "ia"; go("settings"); };
}
async function run(fn) { try { return await fn(); } catch (e) { toast(String(e)); } }

document.getElementById("nav").addEventListener("click", (e) => {
  const b = e.target.closest("button"); if (b) go(b.dataset.view);
});
function go(v, arg) {
  if (v !== "meetings" && PREP.state && !LIVE.state) { recPrepStop(); }
  view = v; clearInterval(poll); renderSeq++; main.onclick = null;
  document.querySelectorAll("nav button").forEach((b) => b.classList.toggle("on",
    b.dataset.view === v || (v === "meeting" && b.dataset.view === "meetings") || (v === "brief" && b.dataset.view === "briefs")));
  return ({ meetings, meeting, briefs, brief, glossary, trash, settings })[v](arg);
}

// ── Enregistrement depuis le PC ─────────────────────────────────────────
let recTimer = null;
const fmtClock = (n) => `${String(Math.floor(n / 3600)).padStart(2, "0")}:${String(Math.floor(n % 3600 / 60)).padStart(2, "0")}:${String(n % 60).padStart(2, "0")}`;
const meter = (v) => `<span class="meter"><b style="width:${Math.min(100, Math.round((v || 0) / 5))}%"></b></span>`;

const lsGet = (k) => { try { return localStorage.getItem(k) || ""; } catch (_) { return ""; } };
const lsSet = (k, v) => { try { localStorage.setItem(k, v); } catch (_) {} };

// Transcription en direct : l'état est gardé ici, les événements du moteur l'alimentent
const LIVE = { segs: [], tr: {}, partial: "", state: "", msg: "", trMsg: "", wantTr: false, warn: "" };
// Chargement de Whisper dès l'ouverture du panneau d'enregistrement : « Démarrer » reste grisé tant qu'il n'est pas prêt
const PREP = { state: "", model: "", hw: "", msg: "" };
function recPrepUI() {
  const go = $("#r-go"), msg = $("#r-prep"), chk = $("#live-chk");
  if (!go || !msg || !chk) return;
  const want = chk.checked;
  go.disabled = want && PREP.state === "loading";
  go.title = go.disabled ? "Whisper est en cours de chargement…" : "";
  const spin = `<span class="spin"></span>`;
  msg.hidden = !want || !PREP.state;
  msg.innerHTML = PREP.state === "loading" ? `${spin} Chargement de Whisper${PREP.model ? ` (modèle ${esc(PREP.model)})` : ""}… le bouton Démarrer sera disponible dans un instant.`
    : PREP.state === "ready" ? `<span class="pill ok">Whisper prêt</span> ${PREP.model ? `<span class="pill">${esc(PREP.model)}</span> ` : ""}${PREP.hw ? `<span class="pill hw">${esc(PREP.hw)}</span>` : ""}`
    : PREP.state === "error" ? `<span style="color:var(--danger)">Transcription en direct indisponible : ${esc(PREP.msg)}</span><br>L'enregistrement démarrera sans elle.` : "";
}
function recPrepStart() {
  if (PREP.state === "ready") return recPrepUI();
  PREP.state = "loading"; PREP.msg = "";
  recPrepUI();
  invoke("live_prepare").catch((e) => { PREP.state = "error"; PREP.msg = String(e); recPrepUI(); });
}
function recPrepStop() {
  if (!PREP.state) return;
  PREP.state = ""; PREP.model = ""; PREP.hw = "";
  invoke("live_release").catch(() => {});
}
let liveRaf = 0;
function liveRender() {
  if (liveRaf) return;
  liveRaf = requestAnimationFrame(() => {
    liveRaf = 0;
    const box = $("#livebox"); if (!box) return;
    if (!LIVE.state) { box.innerHTML = ""; return; }
    const stateTxt = LIVE.state === "loading" ? "Chargement de Whisper…" : LIVE.state === "error" ? (LIVE.msg || "Erreur") : "";
    const old = box.querySelector(".live-lines"), stick = !old || old.scrollHeight - old.scrollTop - old.clientHeight < 40;
    box.innerHTML = `<div class="card live"><b>Transcription en direct</b> <span class="meta">texte provisoire : la transcription complète est refaite à l'arrêt</span>
      ${LIVE.hw ? `<span class="pill hw">${esc(LIVE.hw)}</span>` : ""}${stateTxt ? `<p class="meta" style="margin-top:6px">${esc(stateTxt)}</p>` : ""}${LIVE.warn ? `<p class="meta" style="margin-top:6px;color:var(--danger)">${esc(LIVE.warn)}</p>` : ""}${LIVE.state === "ready" && !LIVE.segs.length && !LIVE.partial && !LIVE.warn ? `<p class="meta" style="margin-top:6px">En attente de parole…</p>` : ""}${LIVE.trMsg ? `<p class="meta" style="margin-top:6px">${esc(LIVE.trMsg)}</p>` : ""}
      <div class="live-lines">${LIVE.segs.map((s) => `<div class="ll"><span class="tm">${fmtClock(Math.floor(s.start))}</span><span class="lt">${esc(s.text)}${LIVE.tr[s.i] ? `<br><span class="ltr">${esc(LIVE.tr[s.i])}</span>` : ""}</span></div>`).join("")}${LIVE.partial ? `<div class="ll part"><span class="tm"></span><span class="lt">${esc(LIVE.partial)}</span></div>` : ""}</div></div>`;
    const n = box.querySelector(".live-lines"); if (n && stick) n.scrollTop = n.scrollHeight;
  });
}
listen("live-segment", (e) => { LIVE.warn = ""; LIVE.segs.push(e.payload); if (LIVE.segs.length > 400) LIVE.segs.shift(); LIVE.partial = ""; if (LIVE.state === "loading") LIVE.state = "ready"; liveRender(); });
listen("live-partial", (e) => { LIVE.warn = ""; LIVE.partial = e.payload.text || ""; liveRender(); });
listen("live-translation", (e) => { LIVE.tr[e.payload.i] = e.payload.text; liveRender(); });
listen("live-state", (e) => {
  const { state, message, prep, model, hw } = e.payload;
  if (prep) {
    PREP.state = state; PREP.msg = message || ""; if (model) PREP.model = model; if (state === "ready") PREP.hw = message; else if (hw) PREP.hw = hw;
    recPrepUI();
    return;
  }
  if (state === "recording") { LIVE.state = "ready"; LIVE.hw = message; }
  else if (state === "translate-loading") LIVE.trMsg = "Chargement de la traduction : " + message;
  else if (state === "translate-ready") LIVE.trMsg = "";
  else if (state === "translate-error") LIVE.trMsg = message;
  else if (state === "warn") { LIVE.warn = message; }
  else { LIVE.state = state; LIVE.msg = message; }
  liveRender();
});

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
    <div style="margin-top:12px"><label class="chk" title="Affiche le texte au fil de l'enregistrement ; la transcription complète est refaite à l'arrêt"><input type="checkbox" id="live-chk"> Transcription en direct</label>
      <select id="live-tr" style="margin-top:6px;max-width:260px" title="Traduire en direct" hidden><option value="">Sans traduction</option>${TR_LANGS.map(([k, v]) => `<option value="${k}" ${lsGet("rec.livetr") === k ? "selected" : ""}>Traduire en ${v}</option>`).join("")}</select>
      <p class="meta" id="r-prep" style="margin-top:8px" hidden></p></div>
    <div class="bar" style="margin:12px 0 0"><button class="btn rec" id="r-go">● Démarrer</button><button class="btn" id="r-x">Annuler</button></div>
    <p class="meta" style="margin-top:8px">Pour une visioconférence, cochez les deux : votre voix et celle de vos interlocuteurs sont enregistrées. Pour « Son du PC », choisissez la sortie sur laquelle vous écoutez (casque, haut-parleurs).</p></div>`;
  $("#live-chk").onchange = (e) => {
    $("#live-tr").hidden = !e.target.checked;
    if (e.target.checked) recPrepStart(); else { recPrepStop(); recPrepUI(); }
  };
  if ($("#live-chk").checked) recPrepStart(); else recPrepUI();
  $("#live-tr").onchange = (e) => lsSet("rec.livetr", e.target.value);
  const fmtG = fmtGain;
  const gv = (id, out) => { const e = $(id), o = $(out); e.oninput = () => { o.textContent = fmtG(e.value); }; o.textContent = fmtG(e.value); return e; };
  const gm = gv("#r-micg", "#r-micg-v"), gs = gv("#r-sysg", "#r-sysg-v");
  $("#r-x").onclick = () => { recPrepStop(); box.innerHTML = ""; };
  $("#r-go").onclick = () => run(async () => {
    if (!(await helloModal())) return; // l'enregistrement ne démarre qu'au clic sur « Compris »
    const mic = $("#r-mic").checked, system = $("#r-sys").checked;
    const micDevice = $("#r-micdev").value, sysDevice = $("#r-sysdev").value;
    const micGain = gm.value / 100, sysGain = gs.value / 100;
    const live = !!($("#live-chk") && $("#live-chk").checked) && PREP.state === "ready", translateTo = live && $("#live-tr") ? $("#live-tr").value || null : null;
    LIVE.hw = live ? PREP.hw : ""; LIVE.segs = []; LIVE.tr = {}; LIVE.partial = ""; LIVE.warn = ""; LIVE.state = live ? "ready" : ""; LIVE.msg = ""; LIVE.trMsg = ""; LIVE.wantTr = !!translateTo;
    trayLabels();
    await invoke("rec_start", { mic, system, micDevice: micDevice || null, sysDevice: sysDevice || null, micGain, sysGain, live, translateTo });
    lsSet("rec.micgain", String(micGain)); lsSet("rec.sysgain", String(sysGain));
    lsSet("rec.mic", mic ? "1" : "0"); lsSet("rec.sys", system ? "1" : "0"); lsSet("rec.micdev", micDevice); lsSet("rec.sysdev", sysDevice);
    recRefresh();
  });
}

// Rappel avant un enregistrement : se présenter permet à Verbatim de nommer les voix. Renvoie true au clic sur « Compris »
// (ou tout de suite si « Ne plus afficher » a été coché), false si on annule.
function helloModal() {
  if (lsGet("rec.hello") === "0") return Promise.resolve(true);
  return new Promise((res) => {
    const o = document.createElement("div"); o.className = "modal";
    o.innerHTML = `<div class="dlg" role="dialog" aria-modal="true"><h3 style="margin:0 0 10px;font-size:18px;font-weight:600;text-transform:none;letter-spacing:0;color:var(--ink)">Dites bonjour</h3>
      <p style="font-weight:400">Demandez à chaque personne de dire <b>« Bonjour, je suis&nbsp;… »</b> la première fois qu'elle prend la parole : Verbatim reconnaît sa voix et la nomme pendant toute la réunion.</p>
      <label class="chk" style="margin-bottom:16px"><input type="checkbox" id="hello-never"> Ne plus afficher ce message</label>
      <div class="acts"><button class="btn" id="hello-no">Annuler</button><button class="btn pri" id="hello-ok">Compris</button></div></div>`;
    const done = (ok) => {
      if (ok && o.querySelector("#hello-never").checked) lsSet("rec.hello", "0");
      document.removeEventListener("keydown", key, true); o.remove(); res(ok);
    };
    const key = (e) => { if (e.key === "Escape") { e.preventDefault(); done(false); } };
    o.querySelector("#hello-ok").onclick = () => done(true);
    o.querySelector("#hello-no").onclick = () => done(false);
    o.onclick = (e) => { if (e.target === o) done(false); };
    document.addEventListener("keydown", key, true);
    document.body.appendChild(o); o.querySelector("#hello-ok").focus();
  });
}

// Termine l'enregistrement (bouton « Terminer », pastille flottante ou icône de la zone de notification).
let stoppingRec = false;
async function stopRecording() {
  if (stoppingRec) return;
  stoppingRec = true;
  try {
    clearInterval(recTimer);
    const title = $("#r-title") ? $("#r-title").value : "";
    const id = await invoke("rec_stop", { title });
    LIVE.state = ""; PREP.state = ""; PREP.model = ""; PREP.hw = ""; liveRender();
    toast("Enregistrement ajouté — transcription lancée"); go("meeting", id);
  } finally { stoppingRec = false; }
}
listen("overlay-stop", () => run(stopRecording));

// Libellés de l'icône de la zone de notification (menu natif : traduit ici, dans la langue de l'interface)
const trayLabels = () => { const t = window.I18N ? I18N.t : (s) => s; invoke("tray_labels", { show: t("Afficher Verbatim"), pause: t("Pause / Reprendre"), stop: t("Terminer l'enregistrement") }).catch(() => {}); };
trayLabels();

async function recRefresh() {
  clearInterval(recTimer);
  const box = $("#recbox"); if (!box) return;
  const st = await run(() => invoke("rec_status"));
  if (!st || !st.recording) { return; }
  const draw = async () => {
    const b = $("#recbox"); if (!b || view !== "meetings") return clearInterval(recTimer);
    const s = await invoke("rec_status");
    if (!s.recording) { clearInterval(recTimer); b.innerHTML = ""; return; }
    b.innerHTML = `<div class="card recing ${s.paused ? "paused" : ""} ${s.live ? "live" : ""}"><span class="dot"></span><b>${s.paused ? "En pause" : "Enregistrement en cours"}</b>
      <span class="clock">${fmtClock(s.seconds)}</span>
      ${s.mic != null ? `<span class="meta">Micro</span>${meter(s.mic)}` : ""}${s.system != null ? `<span class="meta">PC</span>${meter(s.system)}` : ""}
      ${s.mic != null ? `<label class="chk" title="Sensibilité du micro" style="gap:6px">Sensibilité <input type="range" id="r-live-g" min="25" max="800" step="5" value="${Math.round((s.mic_gain || 1) * 100)}" style="width:110px"><span class="pill" id="r-live-gv">${fmtGain((s.mic_gain || 1) * 100)}</span></label>` : ""}
      <input id="r-title" placeholder="Titre (facultatif)" style="max-width:240px">
      <button class="btn" id="r-pause">${s.paused ? "▶ Reprendre" : "❚❚ Pause"}</button>
      <button class="btn pri" id="r-stop">■ Terminer</button>
      ${lsGet("rec.hello") === "0" ? `<div class="rec-tip">Rappel : chaque personne dit <b>« Bonjour, je suis&nbsp;… »</b> en prenant la parole pour être reconnue et nommée.</div>` : ""}</div>`;
    const lg = $("#r-live-g");
    if (lg) lg.oninput = () => { $("#r-live-gv").textContent = fmtGain(lg.value); invoke("rec_set_gain", { source: "mic", gain: lg.value / 100 }).catch(() => {}); lsSet("rec.micgain", String(lg.value / 100)); };
    $("#r-pause").onclick = () => run(async () => { await invoke("rec_pause", { paused: !s.paused }); draw(); });
    $("#r-stop").onclick = () => run(stopRecording);
  };
  liveRender();
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
    <div id="recbox"></div><div id="livebox"></div><div id="eng"></div><div id="list"></div>`;
  $("#rec").onclick = () => recPanel();
  recRefresh();
  const cfg0 = await invoke("get_settings");
  const showEng = async () => { const e = $("#eng"); if (!e) return; e.innerHTML = await engineBox(true); bindEngine((await invoke("get_settings")).whisper_model); };
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
      ${m.stage === "ready" ? `<span class="pill ok">Prête</span>` : m.stage === "error" ? `<span class="pill err">Échec</span>` : m.stage === "needs_engine" ? `<span class="pill warn">Moteur requis</span>` : `<div class="prog"><b style="width:${m.progress}%"></b></div><span class="pill">${esc(m.stage_label)}${m.stage === "transcribing" || m.stage === "diarizing" ? " · " + m.progress + " %" : ""}</span>`}
      <button class="btn sm ico ren" data-ren="${m.id}" title="Renommer" aria-label="Renommer"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 20h4L19 9a2.8 2.8 0 0 0-4-4L4 16v4zM13.5 6.5l4 4"/></svg></button>
    </div>`).join("") : `<div class="empty">Aucune réunion pour le moment.</div>`;
    const list = $("#list");
    if (list && list.dataset.h !== html) { list.innerHTML = html; list.dataset.h = html; } // ne redessine que si ça a changé
    // plus rien en cours : inutile de continuer à interroger
    if (ms.length && ms.every((m) => m.stage === "ready" || m.stage === "error" || m.stage === "needs_engine")) clearInterval(poll);
  }
  $("#list").onclick = (e) => {
    const r = e.target.closest("[data-ren]");
    if (r) { // renommer sans ouvrir la réunion
      e.stopPropagation();
      run(async () => {
        const t = await askText("Nouveau titre", r.closest(".row").querySelector(".t").textContent);
        if (t) { await invoke("update_meeting", { id: r.dataset.ren, title: t }); await draw(); }
      });
      return;
    }
    const c = e.target.closest(".row"); if (c) go("meeting", c.dataset.id);
  };
  await draw();
  if (view === "meetings") setPoll(() => view === "meetings" && draw(), 1500);
}

// Langues de rédaction du compte-rendu et titres des sections dans chacune
const REP_LANGS = [["auto", "Automatique"], ["fr", "Compte-rendu en français"], ["en", "Compte-rendu en anglais"], ["es", "Compte-rendu en espagnol"], ["de", "Compte-rendu en allemand"], ["it", "Compte-rendu en italien"], ["pt", "Compte-rendu en portugais"], ["nl", "Compte-rendu en néerlandais"]];
const REP_LABELS = {
  fr: ["Résumé", "Points abordés", "Décisions", "Actions", "Prochaines étapes"],
  en: ["Summary", "Topics discussed", "Decisions", "Actions", "Next steps"],
  es: ["Resumen", "Temas tratados", "Decisiones", "Acciones", "Próximos pasos"],
  de: ["Zusammenfassung", "Besprochene Punkte", "Entscheidungen", "Aufgaben", "Nächste Schritte"],
  it: ["Riepilogo", "Argomenti trattati", "Decisioni", "Azioni", "Prossimi passi"],
  pt: ["Resumo", "Pontos abordados", "Decisões", "Ações", "Próximos passos"],
  nl: ["Samenvatting", "Besproken punten", "Beslissingen", "Acties", "Volgende stappen"],
};
const repLangs = (sel) => REP_LANGS.map(([k, v]) => `<option value="${k}" ${(sel || "auto") === k ? "selected" : ""}>${v}</option>`).join("");

// Traduction de la transcription : langue affichée par réunion (les segments gardent les horodatages d'origine)
const TL_VIEW = {};
const TR_LANGS = [["en", "Anglais"], ["fr", "Français"], ["es", "Espagnol"], ["de", "Allemand"], ["it", "Italien"], ["pt", "Portugais"], ["nl", "Néerlandais"], ["pl", "Polonais"], ["ru", "Russe"], ["ja", "Japonais"], ["zh", "Chinois"], ["ar", "Arabe"], ["tr", "Turc"], ["ko", "Coréen"]];
const trName = (c) => (TR_LANGS.find(([k]) => k === c) || [c, c])[1];
const trOptions = (m) => {
  const have = m.translation_langs || [];
  const cur = TL_VIEW[m.id] || "";
  return `<option value="" data-act="orig" ${cur === "" ? "selected" : ""}>Texte d'origine</option>`
    + (have.length ? `<optgroup label="Afficher">${have.map((c) => `<option value="show:${c}" ${cur === c ? "selected" : ""}>${trName(c)}</option>`).join("")}</optgroup>` : "")
    + `<optgroup label="Traduire en…">${TR_LANGS.map(([k, v]) => `<option value="do:${k}">${v}${have.includes(k) ? " (refaire)" : ""}</option>`).join("")}</optgroup>`;
};

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
  const cfg = await invoke("get_settings");
  if (seq !== renderSeq || view !== "meeting") return; // un rendu plus récent a pris la main
  // mémorise la lecture en cours pour la reprendre après le redessin
  const old = $("#aud");
  const keep = old ? { t: old.currentTime, playing: !old.paused, rate: old.playbackRate } : null;
  const r = m.report;
  const lst = (a) => (a && a.length) ? `<ul class="l">${a.map((x) => `<li>${esc(x)}</li>`).join("")}</ul>` : "<p class='meta'>—</p>";
  const origin = (m.source === "audio" ? "Audio · transcription Whisper locale" : "Transcription importée") + (m.source === "audio" && m.transcribe_hw ? " · " + m.transcribe_hw : "");
  const player = m.has_audio ? `<div class="card player"><audio id="aud" controls preload="metadata" src="${esc(convertFileSrc(m.play_path || m.audio_path))}"></audio>
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
  const shown = TL_VIEW[id] && m.translations && m.translations[TL_VIEW[id]] ? TL_VIEW[id] : "";
  const segsShown = shown ? m.translations[shown] : m.transcript;
  // une couleur par intervenant dès qu'il y en a plusieurs
  const speakers = [...new Set(m.transcript.map((s) => s.speaker))];
  const spkColor = (n) => speakers.length > 1 ? `style="color:hsl(${Math.round((speakers.indexOf(n) * 137.5 + 230) % 360)} 62% var(--sp-l))"` : "";
  const body = m.stage === "error"
    ? `<div class="card"><b>La transcription a échoué</b><p class="meta" style="margin:6px 0 10px">${esc(m.error || "")}</p><button class="btn pri" id="retry">Réessayer</button> <button class="btn dng" id="del">Corbeille</button></div>`
    : m.stage === "needs_engine"
    ? `<div id="eng2"></div>`
    : m.stage !== "ready"
    ? `<div class="card"><div class="prog" style="width:100%"><b style="width:${m.progress}%"></b></div><p class="meta" style="margin-top:8px">${m.stage === "transcribing" || m.stage === "diarizing" ? `<span class="pill" id="tr-pct">${m.progress} %</span> ` : ""}${esc(m.stage_label)}…${m.transcribe_hw ? ` <span class="pill hw">${esc(m.transcribe_hw)}</span>` : ""}</p></div>`
    : `<div class="bar">
      <button class="btn pri" id="gen" ${m.report_state === "generating" ? "disabled" : ""}>${r ? "Régénérer" : "Générer"} le compte-rendu</button>
      <select id="rl" style="width:auto" title="Langue de rédaction du compte-rendu">${repLangs(r && r.lang || cfg.report_lang)}</select>
      <button class="btn" id="ren">Renommer</button>
      ${m.has_audio ? `<button class="btn" id="dia" title="Repère qui parle et attribue chaque passage à un intervenant">Identifier les intervenants</button>` : ""}
      <select id="bf" style="width:auto"><option value="">Aucune préparation liée</option>${bs.map((b) => `<option value="${b.id}" ${m.brief_id === b.id ? "selected" : ""}>${esc(b.subject)}</option>`).join("")}</select>
      <select id="exp" style="width:auto"><option value="">Exporter…</option><optgroup label="${shown ? `Transcription (${trName(shown)})` : "Transcription"}"><option value="txt">Texte (.txt)</option><option value="srt">Sous-titres (.srt)</option><option value="vtt">Sous-titres (.vtt)</option><option value="json">Données (.json)</option></optgroup>${r ? `<optgroup label="Compte-rendu"><option value="report">Compte-rendu (.md)</option></optgroup>` : ""}${m.has_audio ? `<optgroup label="Enregistrement"><option value="audio">Audio</option></optgroup>` : ""}</select>
      <button class="btn dng" id="del">Corbeille</button></div>
    ${m.report_state === "generating" ? `<div class="card"><b>Génération du compte-rendu…</b> <span class="pill" id="rep-pct">${m.report_pct || 0} %</span> <span class="meta" id="rep-prog">${esc(m.report_progress || "")}</span> <span class="pill hw" id="rep-hw" ${m.report_hw ? "" : "hidden"}>${esc(m.report_hw || "")}</span>
      <div class="prog" style="width:100%;margin:8px 0"><b id="rep-bar" style="width:${m.report_pct || 0}%"></b></div>
      <p class="meta">Avec l'IA locale, une longue réunion peut prendre plusieurs minutes. Vous pouvez quitter cette page : la génération continue.</p></div>` : ""}
    ${m.report_state === "error" ? `<div class="card"><b>Échec de la génération</b><p class="meta" style="margin-top:4px">${esc(m.report_error || "")}</p></div>` : ""}
    ${r ? (() => { const L = REP_LABELS[r.lang]; const h = (i, fr, st = "") => L ? `<h3 data-nt${st ? ` style="${st}"` : ""}>${L[i]}</h3>` : `<h3${st ? ` style="${st}"` : ""}>${fr}</h3>`;
      return `<div class="card">${reportBadge(r)}${h(0, "Résumé", "margin-top:0")}<p>${esc(r.summary)}</p>${h(1, "Points abordés")}${lst(r.topics)}${h(2, "Décisions")}${lst(r.decisions)}${h(3, "Actions")}${lst(r.actions)}${h(4, "Prochaines étapes")}${lst(r.next_steps)}</div>`; })() : ""}
    <div class="card"><div class="bar" style="margin:0 0 8px;align-items:center"><h3 style="margin:0;flex:1">Transcription</h3>
      ${m.translate_state === "translating" ? "" : `<select id="tl-sel" style="width:auto" title="Traduire la transcription">${trOptions(m)}</select>`}
      ${shown ? `<button class="btn sm dng" id="tl-del" title="Supprimer cette traduction">Supprimer la traduction</button>` : ""}</div>
      ${m.translate_state === "translating" ? `<div class="tl-status"><span class="meta">Traduction en ${esc(trName(m.translate_lang))}…</span>
        <span class="pill" id="tl-label" ${m.translate_label ? "" : "hidden"}>${esc(m.translate_label || "")}</span>
        <span class="pill hw" id="tl-hw" ${m.translate_hw ? "" : "hidden"}>${esc(m.translate_hw || "")}</span>
        <span class="pill" id="tl-pct">${m.translate_pct || 0} %</span>
        <button class="btn sm dng" id="tl-stop">Arrêter</button></div>` : ""}
      ${shown && m.translation_models && m.translation_models[shown] ? `<p class="tl-status"><span class="pill" title="Modèle et matériel utilisés pour cette traduction">${esc(m.translation_models[shown])}</span></p>` : ""}
      ${m.translate_state === "error" ? `<p class="meta" style="color:var(--danger);margin-bottom:8px">${esc(m.translate_error || "La traduction a échoué.")}</p>` : ""}
      ${segsShown.map((s) => `<div class="seg" data-t="${s.start}" data-e="${s.end}" title="Écouter à partir d'ici"><span class="tm">${fmtT(s.start)}</span><span class="sp" data-sp="${esc(s.speaker)}" ${spkColor(s.speaker)} title="Renommer cet intervenant">${esc(s.speaker)}</span><span>${esc(s.text)}</span></div>`).join("")}</div>`;
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
  $("#gen").onclick = () => run(async () => { await invoke("generate_report", { id, lang: $("#rl").value }); meeting(id); });
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
  const dia = $("#dia");
  if (dia) dia.onclick = () => run(async () => {
    if (m.transcript.some((s) => !/^Intervenant$/.test(s.speaker)) && !(await ask("Les intervenants actuels (et les noms que vous leur avez donnés) seront remplacés. Continuer ?", "Identifier"))) return;
    await invoke("diarize_meeting", { id }); meeting(id);
  });
  main.querySelectorAll(".seg .sp").forEach((el) => el.onclick = (e) => {
    e.stopPropagation();
    run(async () => {
      const from = el.dataset.sp, to = await askText(`Nouveau nom pour « ${from} »`, from);
      if (to && to !== from) { await invoke("rename_speaker", { id, from, to }); meeting(id); }
    });
  });
  const tsel = $("#tl-sel");
  if (tsel) tsel.onchange = () => run(async () => {
    const v = tsel.value;
    if (v === "") { delete TL_VIEW[id]; return meeting(id); }
    if (v.startsWith("show:")) { TL_VIEW[id] = v.slice(5); return meeting(id); }
    const lang = v.slice(3);
    tsel.value = TL_VIEW[id] || "";
    await invoke("translate_meeting", { id, lang }); TL_VIEW[id] = lang; meeting(id);
  });
  const tdel = $("#tl-del"); if (tdel) tdel.onclick = () => run(async () => { if (!(await ask("Supprimer cette traduction ?", "Supprimer"))) return; await invoke("translation_remove", { id, lang: shown }); delete TL_VIEW[id]; meeting(id); });
  const tstop = $("#tl-stop");
  if (tstop) tstop.onclick = () => run(async () => { tstop.disabled = true; tstop.textContent = "Arrêt…"; await invoke("translate_stop"); });
  if (m.translate_state === "translating") {
    setPoll(async () => {
      if (view !== "meeting") return;
      const x = await invoke("get_meeting", { id });
      if (x.translate_state !== "translating") return meeting(id);
      const pe = $("#tl-pct"); if (pe) pe.textContent = (x.translate_pct || 0) + " %";
      const le = $("#tl-label"); if (le) { le.textContent = x.translate_label || ""; le.hidden = !x.translate_label; }
      const he = $("#tl-hw"); if (he) { he.textContent = x.translate_hw || ""; he.hidden = !x.translate_hw; }
    }, 1500);
  }
  $("#bf").onchange = (e) => run(() => invoke("update_meeting", { id, briefId: e.target.value }));
  $("#del").onclick = () => run(async () => { if (!(await ask("Placer cette réunion dans la corbeille ?", "Mettre à la corbeille"))) return; await invoke("update_meeting", { id, trashed: true }); toast("Placée dans la corbeille"); go("meetings"); });
  $("#exp").onchange = (e) => run(async () => {
    const x = e.target.value; e.target.value = ""; if (!x) return;
    const ext = x === "report" ? "md" : x === "audio" ? ((m.audio_path.match(/\.([A-Za-z0-9]+)$/) || [])[1] || "wav") : x;
    const dest = await dialog.save({ defaultPath: `${m.title}.${ext}` });
    if (!dest) return;
    await invoke("export_meeting", { id, format: x, dest, lang: shown || null }); toast("Exporté");
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
    const L = I18N.t, md = [`# ${b.subject}`, "", "## " + L("Objectif"), x.objective, "", "## " + L("Ordre du jour"), ...x.agenda.map((a, i) => `${i + 1}. ${a.title}${a.minutes ? " (" + a.minutes + " min)" : ""}`), "", "## " + L("Questions"), ...x.questions.map((q) => `- ${q}`), ""].join(nl);
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
let setTab = "gen";
async function settings() {
  const s = (await run(() => invoke("get_settings"))) || {};
  const langs = [["fr", "Français"], ["en", "Anglais"], ["es", "Espagnol"], ["de", "Allemand"], ["it", "Italien"], ["auto", "Détection auto"]];
  main.innerHTML = `<h1>Réglages</h1><p class="sub">Tout est enregistré automatiquement, sauf la connexion API.</p>
    <div class="tabs" id="tabs" role="tablist">${[["gen", "Général"], ["tr", "Transcription"], ["ia", "Compte-rendu"], ["tl", "Traduction"], ["mod", "Modèles"], ["net", "Connexions"]].map(([k, v]) => `<button role="tab" data-tab="${k}" class="${setTab === k ? "on" : ""}">${v}</button>`).join("")}</div>
    <section class="pane" data-pane="gen" ${setTab === "gen" ? "" : "hidden"}>
      <div class="card"><h3>Langue de l'interface</h3>
        <select id="ui-lang" style="max-width:260px"><option value="auto" ${I18N.pref === "auto" ? "selected" : ""}>Automatique</option><option value="fr" data-nt ${I18N.pref === "fr" ? "selected" : ""}>Français</option><option value="en" data-nt ${I18N.pref === "en" ? "selected" : ""}>English</option></select></div>
      <div class="card"><h3>Apparence</h3>
        <label style="margin-top:0">Thème</label>
        <select id="th-mode" style="max-width:260px"><option value="light">Clair</option><option value="">Automatique (système)</option><option value="dark">Sombre</option></select>
        <label>Palette de l'application</label>
        <div class="palettes" id="accents"></div></div>
      <div class="card"><h3>Mises à jour</h3>
        <p class="meta" style="margin-bottom:8px">Verbatim cherche au démarrage la dernière version publiée sur GitHub. Avec l'option cochée, elle est téléchargée en arrière-plan ; l'installation ne démarre que lorsque vous cliquez sur « Mettre à jour » (l'application redémarre alors), et jamais pendant un enregistrement, une transcription ou une rédaction.</p>
        <label class="chk"><input type="checkbox" id="au" ${s.auto_update !== false ? "checked" : ""}> Installer automatiquement les nouvelles versions</label>
        <div class="bar" style="margin:12px 0 0"><button class="btn" id="au-chk">Rechercher maintenant</button><span class="meta" id="au-msg"></span></div></div>
    </section>
    <section class="pane" data-pane="tr" ${setTab === "tr" ? "" : "hidden"}>
      <div class="card avn"><h3>Antivirus : ajoutez une exclusion</h3>
        <p class="meta">Les moteurs d'IA (whisper.cpp, llama.cpp) ne sont pas signés : votre antivirus peut les supprimer ou les bloquer (erreurs « accès refusé » ou « connexion fermée »). Pour l'éviter, ajoutez ce dossier aux exclusions de votre antivirus :</p>
        <p style="margin:8px 0"><code class="avn-path"></code> <button class="btn sm avn-copy">Copier le chemin</button></p>
        <p class="meta">Windows Defender : Sécurité Windows › Protection contre les virus et menaces › Gérer les paramètres › Exclusions › Ajouter une exclusion › Dossier.</p></div>
      <div class="card"><h3>Langue et calcul</h3>
        <div class="grid2"><div><label style="margin-top:0">Langue parlée</label><select id="wl">${langs.map(([k, v]) => `<option value="${k}" ${s.whisper_lang === k ? "selected" : ""}>${v}</option>`).join("")}</select></div></div>
        <div id="wcalc"></div></div>
      <div class="card"><h3>Modèle de transcription</h3><div id="models"></div>
        <p class="meta" style="margin-top:10px">Les modèles « compressés » sont presque aussi précis que les complets, pour moitié moins d'espace. Plus le modèle est gros, plus c'est lent.</p></div>
      <div class="card"><h3>Séparation des intervenants</h3>
        <p class="meta" style="margin-bottom:8px">Attribue chaque passage à un intervenant (« Intervenant 1 », « Intervenant 2 »…), renommable ensuite. Nombre détecté automatiquement ; calcul local, sur le processeur.</p>
        <p class="meta" style="margin-bottom:12px"><b>Astuce :</b> si chacun dit « Bonjour, je suis Marie » en prenant la parole, sa voix est reconnue et nommée pendant toute la réunion.</p>
        <div id="diar"></div></div>
    </section>
    <section class="pane" data-pane="ia" ${setTab === "ia" ? "" : "hidden"}>
      <div class="card avn"><h3>Antivirus : ajoutez une exclusion</h3>
        <p class="meta">Les moteurs d'IA (whisper.cpp, llama.cpp) ne sont pas signés : votre antivirus peut les supprimer ou les bloquer (erreurs « accès refusé » ou « connexion fermée »). Pour l'éviter, ajoutez ce dossier aux exclusions de votre antivirus :</p>
        <p style="margin:8px 0"><code class="avn-path"></code> <button class="btn sm avn-copy">Copier le chemin</button></p>
        <p class="meta">Windows Defender : Sécurité Windows › Protection contre les virus et menaces › Gérer les paramètres › Exclusions › Ajouter une exclusion › Dossier.</p></div>
      <div class="card"><h3>Langue du compte-rendu</h3>
        <p class="meta" style="margin-bottom:8px">Le compte-rendu peut être rédigé dans une autre langue que celle de la réunion. Vous pouvez aussi la choisir réunion par réunion.</p>
        <select id="rl-def" style="max-width:260px">${repLangs(s.report_lang)}</select></div>
      <div class="card"><h3>Calcul de la rédaction</h3><div id="ldev"></div></div>
      <div class="card"><h3>IA de rédaction</h3>
        <p class="meta" style="margin-bottom:10px">Un modèle installé sur cet ordinateur rédige le compte-rendu, sans rien envoyer en ligne. Sans modèle local ni API, seul un résumé simplifié est produit.</p>
        <div id="llms"></div></div>
    </section>
    <section class="pane" data-pane="tl" ${setTab === "tl" ? "" : "hidden"}>
      <div class="card"><h3>Calcul de la traduction</h3><div id="tdev"></div></div>
      <div class="card"><h3>Modèle de traduction</h3>
        <p class="meta" style="margin-bottom:10px">Sert à traduire les transcriptions et, en direct, ce qui se dit pendant l'enregistrement. Tout reste sur cet ordinateur.</p>
        <div id="trms"></div>
        <p class="meta" style="margin-top:8px">Dossier des modèles de traduction : <code id="tdir"></code></p></div>
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
  let curModel = s.whisper_model, curLlm = s.llm_local || "", curWdev = s.whisper_device || "auto", curDev = !s.llm_device || s.llm_device === "cpu" ? "auto" : s.llm_device, curRocm = s.rocm_root || "", curTr = s.translate_model || "", curTdev = s.translate_device || "auto";
  // Mémoire vidéo : un modèle ne tient sur le GPU que s'il y a de la place pour lui + le contexte (~1,5 Go)
  invoke("models_dir_info").then((r) => {
    document.querySelectorAll(".avn-path").forEach((e) => { e.textContent = r.path; });
    document.querySelectorAll(".avn-copy").forEach((b) => b.onclick = async () => { try { await navigator.clipboard.writeText(r.path); toast("Chemin copié"); } catch (x) { toast(r.path); } });
  });
  let gpuVram = 0;
  const noFit = (bytes) => gpuVram > 0 && curDev !== "cpu_only" && bytes + 1.5 * 1073741824 > gpuVram;
  const fitMsg = (bytes) => `<br><span class="meta" style="color:var(--danger)">Trop volumineux pour la mémoire vidéo de votre carte (${(gpuVram / 1073741824).toFixed(0)} Go) : environ ${((bytes + 1.5 * 1073741824) / 1073741824).toFixed(1)} Go nécessaires. Une partie du modèle tournera sur le processeur : rédaction très lente.</span>`;
  const saveAll = async () => invoke("set_settings", { settings: { llm_base_url: $("#u").value.trim(), llm_api_key: $("#k").value, llm_model: $("#m").value.trim(), whisper_model: curModel, whisper_lang: $("#wl").value, llm_local: curLlm, whisper_device: curWdev, llm_device: curDev, rocm_root: curRocm, auto_update: $("#au").checked, report_lang: $("#rl-def").value, translate_model: curTr, translate_device: curTdev } });
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
    const wdev = ["cpu", "vulkan"].includes(curWdev) ? curWdev : "auto";
    const anyGpu = g.cuda || g.rocm || g.vulkan, gcuda = !g.rocm && g.cuda;
    const gname = wdev === "vulkan" && g.vulkan ? "Vulkan" : g.rocm ? "AMD Radeon · ROCm" : g.cuda ? "NVIDIA · CUDA" : g.vulkan ? "Vulkan" : "";
    const wstate = wdev === "cpu" ? `<span class="pill">CPU</span>` : anyGpu && (g.rocm || g.installed || g.vulkan) ? `<span class="pill ok">GPU actif · ${gname}</span>` : `<span class="pill warn">CPU (GPU indisponible)</span>`;
    $("#wcalc").innerHTML = `<label>Calcul de la transcription ${wstate}</label>
      <div class="seg2">
        <label class="opt ${wdev === "auto" ? "sel" : ""}"><input type="radio" name="wd" value="auto" ${wdev === "auto" ? "checked" : ""}><span><b>Automatique</b><br><span class="meta">${anyGpu ? "GPU (" + gname + "), repli sur le CPU en cas de problème" : "GPU si disponible, sinon CPU"}</span></span></label>
        ${g.vulkan ? `<label class="opt ${wdev === "vulkan" ? "sel" : ""}"><input type="radio" name="wd" value="vulkan" ${wdev === "vulkan" ? "checked" : ""}><span><b>Vulkan</b><br><span class="meta">Force Vulkan (AMD, Intel, NVIDIA), même si ROCm ou CUDA sont disponibles</span></span></label>` : ""}
        <label class="opt ${wdev === "cpu" ? "sel" : ""}"><input type="radio" name="wd" value="cpu" ${wdev === "cpu" ? "checked" : ""}><span><b>CPU uniquement</b><br><span class="meta">Processeur, compatible partout</span></span></label></div>
      ${anyGpu ? "" : `<p class="meta" style="margin-top:8px;color:var(--danger)">Aucun GPU utilisable${g.names.length ? " (" + esc(g.names.join(", ")) + ")" : ""} : il faut une carte NVIDIA (CUDA), AMD RDNA3 (ROCm), ou un pilote Vulkan (AMD, Intel) avec le moteur Whisper Vulkan.</p>`}
      <details style="margin-top:10px"><summary class="meta">Moteurs et emplacements</summary>
        <p class="meta" style="word-break:break-all;margin-top:6px">CPU : <code>${esc(st.binary_path)}</code> ${st.binary ? "" : "(installé avec le premier modèle)"}</p>
        ${g.rocm_engine ? `<p class="meta" style="word-break:break-all">ROCm : <code>${esc(g.rocm_path)}</code></p>` : ""}
        ${g.vulkan_engine ? `<p class="meta" style="word-break:break-all">Vulkan : <code>${esc(g.vulkan_path)}</code>${g.vulkan_driver ? "" : " (pilote Vulkan introuvable)"}</p>` : ""}
        ${gcuda ? `<p class="meta" style="word-break:break-all">CUDA : <code>${esc(g.path)}</code> ${g.installed ? "" : `<button class="btn sm" id="g-dl">Télécharger (${fmtSize(g.bytes)})</button>`}</p>${gbusy ? `<div class="prog" style="width:100%;margin-top:6px"><b data-dlbar style="width:${dlPct(d)}%"></b></div><span class="meta" data-dlmsg>${esc(dlLabel(d))}</span>` : ""}` : ""}
      </details>`;
    $("#wcalc").querySelectorAll("[name=wd]").forEach((r) => r.onchange = () => run(async () => { curWdev = r.value; await saveAll(); renderModels(); }));
    const dg = st.diarization, dbusy = d.active && d.phase === "diarization";
    $("#diar").innerHTML = dg.installed
      ? `<div class="bar" style="margin:0;justify-content:space-between"><span><span class="pill ok">Actif</span> <span class="meta">Modèles installés · ${fmtSize(dg.bytes)}</span></span><button class="btn sm dng" id="dia-del">Supprimer</button></div>
        <details style="margin-top:8px"><summary class="meta">Emplacement et réunion déjà transcrite</summary>
          <p class="meta" style="margin-top:6px;word-break:break-all">Modèles : <code>${esc(dg.path)}</code></p>
          <p class="meta" style="margin-top:4px">Pour une réunion déjà transcrite, utilisez « Identifier les intervenants » dans la réunion.</p></details>`
      : dbusy ? `<div class="prog" style="width:100%"><b data-dlbar style="width:${dlPct(d)}%"></b></div><span class="meta" data-dlmsg>${esc(dlLabel(d))}</span>`
      : `<p class="meta" style="margin-bottom:8px">Deux petits modèles sont nécessaires (${fmtSize(dg.bytes)} en tout), téléchargés une seule fois.</p><button class="btn" id="dia-get">Télécharger (${fmtSize(dg.bytes)})</button>`;
    const dGet = $("#dia-get"); if (dGet) dGet.onclick = () => run(async () => { await invoke("diarize_install"); renderModels(); watchDownload(); });
    const dDel = $("#dia-del"); if (dDel) dDel.onclick = () => run(async () => { if (await ask("Supprimer les modèles de séparation des intervenants ? Les transcriptions suivantes n'auront plus d'intervenants distincts.")) { await invoke("diarize_remove"); renderModels(); } });
    const gd = $("#g-dl"); if (gd) gd.onclick = () => run(async () => { await invoke("engine_install_gpu"); renderModels(); watchDownload(); });
    const llm = st.llm;
    gpuVram = llm.vram || 0;
    const lrows = [`<div class="card mrow ${curLlm === "" ? "sel" : ""}"><label class="chk" style="flex:1"><input type="radio" name="lm" value="" ${curLlm === "" ? "checked" : ""}>
      <span><b>Aucune IA locale</b><br><span class="meta">Utiliser l'API distante si elle est configurée, sinon un résumé simplifié</span></span></label></div>`].concat(llm.models.filter((m) => !m.translator).map((m) => {
      const busy = d.active && d.model === m.name;
      const pct = m.partial ? Math.round((m.partial / m.bytes) * 100) : 0;
      const state = m.installed ? `<span class="pill ok">Installé</span>` : busy ? `<span class="pill">Téléchargement…</span>` : m.partial ? `<span class="pill warn">Interrompu ${pct} %</span>` : "";
      const act = m.installed ? `<button class="btn sm dng" data-ldel="${m.name}">Supprimer</button>`
        : busy ? "" : `<button class="btn sm" data-lget="${m.name}">${m.partial ? "Reprendre" : "Télécharger"}</button>`;
      return `<div class="card mrow ${curLlm === m.name ? "sel" : ""} ${noFit(m.bytes) ? "nofit" : ""}"><label class="chk" style="flex:1"><input type="radio" name="lm" value="${m.name}" ${curLlm === m.name ? "checked" : ""} ${m.installed ? "" : "disabled"}>
        <span><b>${esc(m.label)}</b> <span class="meta">· ${fmtSize(m.bytes)}</span><br><span class="meta">${esc(m.desc)}</span><br><span class="meta" style="word-break:break-all">${m.installed ? "Emplacement" : "Sera installé dans"} : <code>${esc(m.path)}</code></span>${noFit(m.bytes) ? fitMsg(m.bytes) : ""}
        ${busy ? `<div class="prog" style="width:100%;margin-top:6px"><b data-dlbar style="width:${dlPct(d)}%"></b></div><span class="meta" data-dlmsg>${esc(dlLabel(d))}</span>` : ""}</span></label>${state}${act}</div>`;
    }));
    $("#llms").innerHTML = lrows.join("") + `<p class="meta" style="margin-top:6px">Moteur d'IA locale (llama.cpp) : ${llm.binary ? "installé" : "non installé (téléchargé avec le premier modèle)"} · <code>${esc(llm.binary_path)}</code></p><p class="meta" style="margin-top:6px">Le modèle n'est chargé en mémoire (4 à 24 Go selon le modèle) que pendant la rédaction.</p>`;
    $("#llms").querySelectorAll("[name=lm]").forEach((r) => r.onchange = () => run(async () => { curLlm = r.value; await saveAll(); renderModels(); if (typeof renderOllama === "function") renderOllama(); if (typeof renderGguf === "function") renderGguf(); }));
    $("#llms").querySelectorAll("[data-lget]").forEach((b) => b.onclick = () => run(async () => { await invoke("llm_install", { model: b.dataset.lget }); renderModels(); watchDownload(); }));
    $("#llms").querySelectorAll("[data-ldel]").forEach((b) => b.onclick = () => run(async () => { if (await ask("Supprimer ce modèle du disque ?")) { await invoke("llm_remove", { model: b.dataset.ldel }); curLlm = (await invoke("get_settings")).llm_local || ""; renderModels(); renderGguf(); } }));
    // modèles de traduction (TranslateGemma)
    const trRow = (m) => {
      const busy = d.active && d.model === m.name;
      const pct = m.partial ? Math.round((m.partial / m.bytes) * 100) : 0;
      const state = m.installed ? `<span class="pill ok">Installé</span>` : busy ? `<span class="pill">Téléchargement…</span>` : m.partial ? `<span class="pill warn">Interrompu ${pct} %</span>` : "";
      const act = m.installed ? `<button class="btn sm dng" data-ldel="${m.name}">Supprimer</button>` : busy ? "" : `<button class="btn sm" data-lget="${m.name}">${m.partial ? "Reprendre" : "Télécharger"}</button>`;
      return `<div class="card mrow ${curTr === m.name ? "sel" : ""} ${noFit(m.bytes) ? "nofit" : ""}"><label class="chk" style="flex:1"><input type="radio" name="tm" value="${m.name}" ${curTr === m.name ? "checked" : ""} ${m.installed ? "" : "disabled"}>
        <span><b>${esc(m.label)}</b> <span class="meta">· ${fmtSize(m.bytes)}</span><br><span class="meta">${esc(m.desc)}</span><br><span class="meta" style="word-break:break-all">${m.installed ? "Emplacement" : "Sera installé dans"} : <code>${esc(m.path)}</code></span>${noFit(m.bytes) ? fitMsg(m.bytes) : ""}
        ${busy ? `<div class="prog" style="width:100%;margin-top:6px"><b data-dlbar style="width:${dlPct(d)}%"></b></div><span class="meta" data-dlmsg>${esc(dlLabel(d))}</span>` : ""}</span></label>${state}${act}</div>`;
    };
    const tdev = $("#tdev");
    if (tdev) {
      const tdefs = [["auto", "Automatique", "Carte graphique si possible (CUDA, ROCm ou Vulkan), sinon processeur"], ["cpu_only", "Processeur uniquement", "Plus lent, mais n'utilise pas la mémoire vidéo : utile si le GPU est occupé par Whisper"]];
      tdev.innerHTML = tdefs.map(([k, v, d2]) => `<div class="card mrow ${curTdev === k ? "sel" : ""}"><label class="chk" style="flex:1"><input type="radio" name="td" value="${k}" ${curTdev === k ? "checked" : ""}><span><b>${v}</b><br><span class="meta">${d2}</span></span></label></div>`).join("")
        + `<p class="meta" style="margin-top:6px">Calcul utilisé actuellement : <span class="pill hw">${esc(curTdev === "cpu_only" ? "CPU" : llm.translate_hw)}</span></p>`;
      tdev.querySelectorAll("[name=td]").forEach((r) => r.onchange = () => run(async () => { curTdev = r.value; await saveAll(); renderModels(); }));
      const td = $("#tdir"); if (td) td.textContent = llm.translation_dir;
    }
    const trs = $("#trms");
    if (trs) {
      trs.innerHTML = `<div class="card mrow ${curTr === "" ? "sel" : ""}"><label class="chk" style="flex:1"><input type="radio" name="tm" value="" ${curTr === "" ? "checked" : ""}>
        <span><b>Même modèle que le compte-rendu</b><br><span class="meta">Pratique, mais un modèle de traduction dédié donne de meilleurs résultats</span></span></label></div>` + llm.models.filter((m) => m.translator).map(trRow).join("");
      trs.querySelectorAll("[name=tm]").forEach((r) => r.onchange = () => run(async () => { curTr = r.value; await saveAll(); renderModels(); }));
      trs.querySelectorAll("[data-lget]").forEach((b) => b.onclick = () => run(async () => { await invoke("llm_install", { model: b.dataset.lget }); renderModels(); watchDownload(); }));
      trs.querySelectorAll("[data-ldel]").forEach((b) => b.onclick = () => run(async () => { if (await ask("Supprimer ce modèle du disque ?")) { await invoke("llm_remove", { model: b.dataset.ldel }); curTr = (await invoke("get_settings")).translate_model || ""; renderModels(); } }));
    }
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
      return `<div class="card mrow ${curLlm === val ? "sel" : ""} ${noFit(m.bytes) ? "nofit" : ""}"><label class="chk" style="flex:1"><input type="radio" name="lm" value="${esc(val)}" ${curLlm === val ? "checked" : ""}>
        <span><b>${esc(m.name)}</b> <span class="meta">· ${fmtSize(m.bytes)}${m.params ? " · " + esc(m.params) : ""}${m.quant ? " · " + esc(m.quant) : ""}</span>${noFit(m.bytes) ? fitMsg(m.bytes) : ""}
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
    gpuVram = st.llm.vram || 0;
    box.innerHTML = (list.length ? `<label style="margin-top:0">Modèles personnalisés installés</label>` : "") + list.map((m) => {
      const val = "gguf:" + m.path;
      return `<div class="card mrow ${curLlm === val ? "sel" : ""} ${noFit(m.bytes) ? "nofit" : ""}"><label class="chk" style="flex:1"><input type="radio" name="lm" value="${esc(val)}" ${curLlm === val ? "checked" : ""}>
        <span><b>${esc(m.name)}</b> <span class="meta">· ${fmtSize(m.bytes)}</span><br><span class="meta" style="word-break:break-all">Emplacement : <code>${esc(m.path)}</code></span>${noFit(m.bytes) ? fitMsg(m.bytes) : ""}${m.bytes > 12 * 1073741824 ? `<br><span class="meta">Gros modèle : chargement long, rédaction lente sans carte graphique.</span>` : ""}</span></label>
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
  $("#rl-def").onchange = () => run(saveAll);
  $("#au").onchange = () => run(saveAll);
  $("#ui-lang").onchange = (e) => { I18N.set(e.target.value); };
  $("#th-mode").value = lsGet("theme") || "light";
  $("#th-mode").onchange = (e) => {
    const t = e.target.value;
    if (t) document.documentElement.dataset.theme = t; else delete document.documentElement.dataset.theme;
    try { localStorage.setItem("theme", t || "auto"); } catch (_) {}
    applyAccent(); drawAccents();
  };
  const drawAccents = () => {
    const cur = lsGet("palette"), dark = themeNow() === "dark";
    $("#accents").innerHTML = PALETTES.map(([k, name, h, nh, ns, o = {}]) => {
      const d = !k ? (dark ? [220, 22, 8] : [220, 20, 96]) : [nh, ns, dark ? 8 : 96];
      const bg = `hsl(${d[0]} ${d[1]}% ${d[2]}%)`, pn = dark ? `hsl(${d[0]} ${d[1]}% 13%)` : "#fff", br = `hsl(${h} ${Math.min(100, Math.round(70 * (o.s ?? 1)))}% ${dark ? (o.L ? o.L[3] : 66) : (o.L ? o.L[1] + 8 : 46)}%)`;
      return `<button type="button" class="pal ${cur === k ? "on" : ""}" data-ac="${k}" aria-pressed="${cur === k}" style="--pb:${bg};--pp:${pn};--pc:${br}"><span class="pv"><i></i><b></b><u></u></span><span class="pn">${name}</span></button>`;
    }).join("");
  };
  $("#accents").onclick = (e) => {
    const b = e.target.closest("[data-ac]"); if (!b) return;
    try { b.dataset.ac ? localStorage.setItem("palette", b.dataset.ac) : localStorage.removeItem("palette"); } catch (_) {}
    applyAccent(); drawAccents();
  };
  drawAccents();
  $("#au-chk").onclick = async () => {
    const msg = $("#au-msg"); msg.textContent = "Recherche…";
    try {
      const u = await updCheck(true);
      if (!u) { msg.textContent = "Vous avez la dernière version (v" + (await invoke("app_version")).version + ")."; return; }
      msg.textContent = "Version " + u.version + " disponible.";
    } catch (e) { msg.textContent = updErr(e); }
  };
  renderModels(); renderOllama(); renderGguf(); renderFound();
  $("#save").onclick = () => run(async () => { await saveAll(); toast("Réglages enregistrés"); });
}

// Version affichée : v{version}-{horodatage du build} ; la comparaison des versions se fait sur le numéro seul.
invoke("app_version").then((v) => { const e = $("#ver"); e.textContent = v.label; e.title = "Verbatim " + v.label; }).catch(() => {});

// ── Mise à jour : plugin officiel Tauri (téléchargement, signature, installation), redémarrage par l'installateur ──
// phases : idle → available → downloading → ready → installing | error. `view` : fenêtre centrée affichée ("" = aucune).
const UPD = { phase: "idle", update: null, version: "", notes: "", pct: 0, done: 0, total: 0, error: "", view: "", busy: "", dismissed: false, timer: null, last: 0 };
const updMb = (n) => (n / 1048576).toFixed(0) + " Mo";
function updErr(e) {
  const s = String((e && e.message) || e);
  if (/network|request|dns|connect|timed? ?out|error sending|offline/i.test(s)) return "Pas de connexion à internet. Vérifiez-la puis réessayez.";
  if (/signature/i.test(s)) return "La signature de la mise à jour est invalide : elle n'a pas été installée.";
  return s;
}
async function updBusy() {
  try { const b = await invoke("update_busy"); UPD.busy = b.busy ? b.reason : ""; } catch (_) { UPD.busy = ""; }
}
// Cherche une version plus récente ; renvoie la mise à jour ou null. `manual` : réaffiche le bandeau même s'il avait été ignoré.
async function updCheck(manual) {
  if (["downloading", "ready", "installing"].includes(UPD.phase)) return UPD.update;
  const api = (window.__TAURI__ || {}).updater;
  if (!api) throw new Error("La mise à jour n'est pas disponible dans cette version.");
  const u = await api.check();
  if (!u) { UPD.phase = "idle"; UPD.update = null; updRender(); return null; }
  Object.assign(UPD, { phase: "available", update: u, version: u.version, notes: u.body || "", error: "" });
  if (manual) UPD.dismissed = false;
  updRender();
  return u;
}
async function updDownload() {
  Object.assign(UPD, { phase: "downloading", pct: 0, done: 0, total: 0, error: "" });
  updRender();
  try {
    await UPD.update.download((ev) => {
      if (ev.event === "Started") UPD.total = ev.data.contentLength || 0;
      else if (ev.event === "Progress") {
        UPD.done += ev.data.chunkLength;
        if (UPD.total) UPD.pct = Math.min(100, Math.round((UPD.done / UPD.total) * 100));
        if (Date.now() - UPD.last > 150) { UPD.last = Date.now(); updRender(); } // pas à chaque paquet reçu
      } else if (ev.event === "Finished") UPD.pct = 100;
    });
    UPD.phase = "ready";
  } catch (e) {
    UPD.phase = "error"; UPD.error = updErr(e);
    if (UPD.view) UPD.view = "error";
  }
  updRender();
}
// Installe puis redémarre : l'installateur (fenêtre passive) ferme l'application, installe, et la relance.
async function updInstall() {
  await updBusy();
  if (UPD.busy) { UPD.view = "ready"; updRender(); return; }
  UPD.phase = "installing"; UPD.view = "installing"; updRender();
  try {
    await invoke("update_prepare"); // arrête les moteurs d'IA (le plugin quitte sans passer par la fermeture normale)
    await UPD.update.install();
    await window.__TAURI__.process.relaunch();
  } catch (e) {
    UPD.phase = "error"; UPD.error = updErr(e); UPD.view = "error"; updRender();
  }
}
// Bouton « Mettre à jour » du bandeau ou des notes : télécharge (fenêtre de progression) puis installe.
async function updManual() {
  try {
    if (!UPD.update) { if (!(await updCheck(true))) return; }
    UPD.dismissed = false;
    if (UPD.phase !== "ready") {
      UPD.view = "progress";
      await updDownload();
      if (UPD.phase !== "ready") return;
    }
    await updInstall();
  } catch (e) { UPD.phase = "error"; UPD.error = updErr(e); UPD.view = "error"; updRender(); }
}
async function updAct(a) {
  if (a === "go") return updManual();
  if (a === "install") return updInstall();
  if (a === "x") { UPD.dismissed = true; return updRender(); }
  if (a === "notes") { UPD.view = "notes"; return updRender(); }
  if (a === "ready") { UPD.view = "ready"; await updBusy(); return updRender(); }
  UPD.view = ""; updRender(); // bg, later, close
}
function updRender() {
  const bar = $("#upd-bar"), p = UPD.phase, v = esc(UPD.version), pct = UPD.pct;
  let html = "";
  if (!UPD.view) {
    if (p === "available" && !UPD.dismissed) html = `<span class="msg"><b>La version ${v} est disponible.</b></span><button class="btn pri sm" data-u="go">Mettre à jour</button><button class="btn sm" data-u="notes">Notes de version</button><button class="btn sm" data-u="x" title="Ignorer pour cette session" aria-label="Ignorer pour cette session">×</button>`;
    else if (p === "downloading") html = `<span class="msg">Téléchargement de la version ${v} : ${pct} %</span><div class="prog"><b style="width:${pct}%"></b></div>`;
    else if (p === "ready") html = `<span class="msg"><b>La version ${v} est prête.</b> Redémarrez l'application pour l'installer.</span><button class="btn pri sm" data-u="ready">Mettre à jour</button>`;
    else if (p === "error" && !UPD.dismissed) html = `<span class="msg">Mise à jour impossible : ${esc(UPD.error)}</span><button class="btn pri sm" data-u="go">Réessayer</button><button class="btn sm" data-u="x" aria-label="Fermer">×</button>`;
  }
  bar.hidden = !html;
  if (bar.dataset.h !== html) { bar.innerHTML = html; bar.dataset.h = html; }
  // fenêtre centrée (fond flouté)
  let m = $("#upd-modal");
  if (!UPD.view) { if (m) m.remove(); clearInterval(UPD.timer); UPD.timer = null; return; }
  if (!m) {
    m = document.createElement("div"); m.id = "upd-modal"; m.className = "modal";
    m.onclick = (e) => {
      const b = e.target.closest("[data-u]");
      if (b) return updAct(b.dataset.u);
      if (e.target === m && UPD.view !== "installing") updAct("close");
    };
    document.body.appendChild(m);
  }
  const busy = UPD.busy ? `<p class="meta upd-busy">${esc(UPD.busy)} Terminez-le avant de mettre à jour.</p>` : "";
  const size = UPD.total ? ` · ${updMb(UPD.done)} / ${updMb(UPD.total)}` : "";
  const body = {
    progress: `<h3>Téléchargement de la version ${v}</h3><div class="prog wide"><b style="width:${pct}%"></b></div><p class="meta">${pct} %${size}</p><div class="acts"><button class="btn" data-u="close">Continuer en arrière-plan</button></div>`,
    ready: `<h3>La version ${v} est prête</h3><p>Verbatim va se fermer puis redémarrer tout seul pour appliquer la mise à jour.</p>${busy}<div class="acts"><button class="btn" data-u="close">Plus tard</button><button class="btn pri" data-u="install" ${UPD.busy ? "disabled" : ""}>Mettre à jour</button></div>`,
    installing: `<div class="spin" aria-hidden="true"></div><h3>Installation de la version ${v}…</h3><p>Verbatim va se fermer puis redémarrer tout seul. Cela peut prendre plusieurs minutes : ne l'éteignez pas.</p>`,
    error: `<h3>Mise à jour impossible</h3><p>${esc(UPD.error)}</p><div class="acts"><button class="btn" data-u="close">Fermer</button><button class="btn pri" data-u="go">Réessayer</button></div>`,
    notes: `<h3>Version ${v}</h3><pre class="upd-notes">${esc(UPD.notes || "Aucune note pour cette version.")}</pre><div class="acts"><button class="btn" data-u="close">Fermer</button><button class="btn pri" data-u="go">Mettre à jour</button></div>`,
  }[UPD.view] || "";
  const html2 = `<div class="dlg" role="dialog" aria-modal="true">${body}</div>`;
  if (m.dataset.h !== html2) { m.innerHTML = html2; m.dataset.h = html2; }
  // tant que la fenêtre propose d'installer, on surveille les travaux en cours (le bouton se dégrise tout seul)
  if (["ready", "notes"].includes(UPD.view) && !UPD.timer) {
    UPD.timer = setInterval(async () => { const old = UPD.busy; await updBusy(); if (old !== UPD.busy) updRender(); }, 2000);
  } else if (!["ready", "notes"].includes(UPD.view)) { clearInterval(UPD.timer); UPD.timer = null; }
}
$("#upd-bar").onclick = (e) => { const b = e.target.closest("[data-u]"); if (b) updAct(b.dataset.u); };
// Au démarrage : recherche, puis (si « installer automatiquement » est coché) téléchargement en arrière-plan.
// Rien n'est installé ni redémarré sans un clic sur « Mettre à jour ».
async function updStartup() {
  try {
    const u = await updCheck(false); if (!u) return;
    const cfg = await invoke("get_settings");
    if (cfg.auto_update === false) return;
    await updDownload();
    if (UPD.phase !== "ready") return;
    await updBusy();
    if (!UPD.busy) { UPD.view = "ready"; updRender(); } // sinon le bandeau « prête » reste affiché jusqu'au clic
  } catch (_) {}
}
setTimeout(updStartup, 3000);
invoke("install_lang").then((v) => I18N.setInstall(v)).catch(() => {});
// Fenêtre d'information avec des boutons ; renvoie la valeur du bouton cliqué (Échap ou clic à côté : la valeur du premier bouton).
function infoModal(title, paras, buttons) {
  return new Promise((res) => {
    const o = document.createElement("div"); o.className = "modal";
    o.innerHTML = `<div class="dlg" role="dialog" aria-modal="true"><h3 style="margin:0 0 10px;font-size:18px;font-weight:600;text-transform:none;letter-spacing:0;color:var(--ink)"></h3>
      ${paras.map(() => `<p style="font-weight:400;margin-bottom:12px"></p>`).join("")}
      <div class="acts" style="margin-top:16px">${buttons.map((b, i) => `<button class="btn ${b.cls || ""}" data-i="${i}"></button>`).join("")}</div></div>`;
    o.querySelector("h3").textContent = title;
    o.querySelectorAll("p").forEach((p, i) => { p.textContent = paras[i]; });
    o.querySelectorAll("[data-i]").forEach((b) => { b.textContent = buttons[b.dataset.i].label; });
    const done = (v) => { document.removeEventListener("keydown", key, true); o.remove(); res(v); };
    const key = (e) => { if (e.key === "Escape") { e.preventDefault(); done(buttons[0].val); } };
    o.onclick = (e) => { const b = e.target.closest("[data-i]"); if (b) done(buttons[b.dataset.i].val); else if (e.target === o) done(buttons[0].val); };
    document.addEventListener("keydown", key, true);
    document.body.appendChild(o);
    const pri = o.querySelector(".pri") || o.querySelector("[data-i]"); if (pri) pri.focus();
  });
}

// Fermeture de la fenêtre pendant un travail en cours : Rust la retient et demande confirmation (`app_quit` quitte vraiment).
let closeAsking = false;
listen("close-requested", async (e) => {
  if (closeAsking) return;
  closeAsking = true;
  const reason = String(e.payload || "Un travail est en cours.");
  const hint = /enregistrement/.test(reason) ? "Il sera arrêté ; ce qui est déjà enregistré sera récupéré au prochain lancement."
    : /transcription/.test(reason) ? "Elle reprendra au prochain lancement."
    : /téléchargement/.test(reason) ? "Il reprendra là où il s'est arrêté."
    : /compte-rendu/.test(reason) ? "Il sera interrompu : il faudra le relancer."
    : "Elle sera interrompue : il faudra la relancer.";
  const quit = await infoModal("Fermer Verbatim ?", [reason, hint], [{ label: "Rester", cls: "pri", val: false }, { label: "Quitter quand même", cls: "dng2", val: true }]);
  closeAsking = false;
  if (quit) invoke("app_quit").catch(() => {});
});

// Première utilisation : aucun modèle de transcription installé → on invite à les choisir dans les Réglages.
async function welcomeCheck() {
  if (lsGet("welcome") === "1") return;
  let st;
  try { st = await invoke("engine_status"); } catch (_) { return; }
  if (st.models.some((m) => m.installed)) { lsSet("welcome", "1"); return; }
  // un seul bouton : quelle que soit la façon de fermer la fenêtre, on arrive dans les Réglages
  await infoModal("Bienvenue dans Verbatim",
    ["Verbatim travaille entièrement sur votre ordinateur : il lui faut d'abord des modèles d'IA.",
     "Dans les Réglages, choisissez les modèles de transcription, de compte-rendu et de traduction (facultatif).",
     "Chacun n'est téléchargé qu'une seule fois."],
    [{ label: "Ouvrir les Réglages", cls: "pri", val: true }]);
  lsSet("welcome", "1"); setTab = "tr"; go("settings");
}

// Écran de démarrage : la première page se charge pendant que Rust précharge Whisper et la séparation des intervenants
// (`warm_status`). La barre suit le plus lent des deux : le temps minimal d'affichage (1,6 s, pour voir l'animation) ou le
// préchargement ; au bout de 10 s on ouvre l'application quand même, le préchargement continue en arrière-plan.
(async () => {
  const sp = $("#splash"), fill = $("#sp-fill"), msg = sp && sp.querySelector(".sp-msg"), t0 = Date.now();
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  let cur = 0; // la barre ne recule jamais
  const step = (x) => { cur = Math.max(cur, x); if (fill) fill.style.transform = `scaleX(${cur})`; };
  await sleep(60); step(0.05);
  const first = Promise.resolve(go("meetings")).catch(() => {});
  for (;;) {
    const el = Date.now() - t0, timeFrac = Math.min(1, el / 1600);
    let w = null;
    try { w = await invoke("warm_status"); } catch (_) {}
    const warmFrac = w ? (w.done ? 1 : Math.min(0.97, w.pct / 100)) : 1;
    step(Math.min(timeFrac, warmFrac));
    if (msg && w && w.label && !w.done) msg.textContent = w.label;
    if ((timeFrac >= 1 && warmFrac >= 1) || el > 10000) break;
    await sleep(120);
  }
  await first; step(1);
  await sleep(300); // laisse la barre finir de se remplir
  if (!sp) return;
  sp.classList.add("out");
  setTimeout(() => sp.remove(), 450);
  setTimeout(welcomeCheck, 600);
})();
$("#theme").onclick = () => {
  const t = themeNow() === "dark" ? "light" : "dark";
  document.documentElement.dataset.theme = t;
  try { localStorage.setItem("theme", t); } catch (_) {}
  applyAccent();
};

// Animation au clic, propre à chaque thème (Kawaii et Gamer seulement), à l'endroit du curseur
const FX = {
  kawaii: () => {
    const cols = ["#ff8fc7", "#c3b1ff", "#8fd3ff", "#8fe3c4", "#ffd34d"];
    let h = "";
    for (let n = 0; n < 9; n++) {
      const a = Math.random() * 360, d = 20 + Math.random() * 26, s = 8 + Math.round(Math.random() * 6);
      h += `<i class="kp" style="--a:${a.toFixed(0)}deg;--d:${d.toFixed(0)}px;--s:${s}px;--c:${cols[n % cols.length]};--r:${Math.round(Math.random() * 80 - 40)}deg">${n % 2 ? "★" : "♥"}</i>`;
    }
    return h;
  },
  gamer: () => {
    const cols = ["#39ff88", "#ff3df2", "#2de2ff", "#ffe600"];
    let h = `<i class="gx"></i>`;
    for (let n = 0; n < 6; n++) h += `<i class="gp" style="--a:${n * 60 + Math.round(Math.random() * 20)}deg;--d:${22 + Math.round(Math.random() * 18)}px;--c:${cols[n % cols.length]}"></i>`;
    return h;
  },
};
document.addEventListener("pointerdown", (e) => {
  const fxp = FX[document.documentElement.dataset.palette];
  if (!fxp || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
  const b = e.target.closest && e.target.closest(".btn, nav button, .tabs button, .pal, .theme-btn, .lnk, .opt, .mrow");
  if (!b || b.disabled) return;
  const fx = document.createElement("div");
  fx.className = "click-fx fx-" + document.documentElement.dataset.palette;
  fx.style.left = e.clientX + "px"; fx.style.top = e.clientY + "px";
  fx.innerHTML = fxp();
  document.body.appendChild(fx);
  setTimeout(() => fx.remove(), 750);
});
