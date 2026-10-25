// Pastille flottante d'enregistrement : horloge, pause / reprise, terminer, retour à la fenêtre principale.
// toujours sombre : lisible sur n'importe quel écran (le thème est fixé dans overlay.html)
const { invoke } = window.__TAURI__.core;
const $ = (s) => document.querySelector(s);
const fmtClock = (n) => `${String(Math.floor(n / 3600)).padStart(2, "0")}:${String(Math.floor(n % 3600 / 60)).padStart(2, "0")}:${String(n % 60).padStart(2, "0")}`;
const ICON_PAUSE = `<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="6" y="5" width="4" height="14" rx="1"/><rect x="14" y="5" width="4" height="14" rx="1"/></svg>`;
const ICON_PLAY = `<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5.5v13a1 1 0 0 0 1.5.86l10.5-6.5a1 1 0 0 0 0-1.72L9.5 4.64A1 1 0 0 0 8 5.5z"/></svg>`;

let paused = null, busy = false;
function setPaused(p) {
  if (p === paused) return;
  paused = p;
  $("#bar").classList.toggle("paused", p);
  const b = $("#b-pause");
  b.innerHTML = p ? ICON_PLAY : ICON_PAUSE;
  const tip = p ? "Reprendre l'enregistrement" : "Mettre en pause";
  b.title = tip; b.setAttribute("aria-label", tip);
}
setPaused(false);

async function refresh() {
  try {
    const s = await invoke("rec_status");
    if (!s.recording) return;
    $("#clk").textContent = fmtClock(s.seconds);
    setPaused(!!s.paused);
  } catch (_) {}
}

$("#b-pause").onclick = async () => { try { await invoke("rec_pause", { paused: !paused }); } catch (_) {} refresh(); };
$("#b-stop").onclick = async () => {
  if (busy) return; busy = true;
  $("#b-stop").disabled = true;
  try { await invoke("overlay_action", { action: "stop" }); } catch (_) {}
  setTimeout(() => { busy = false; $("#b-stop").disabled = false; }, 8000); // la pastille se ferme d'elle-même si l'arrêt aboutit
};
$("#b-show").onclick = () => invoke("overlay_action", { action: "show" }).catch(() => {});
// Glissement : géré côté Rust (`overlay_drag`) à partir des coordonnées d'écran du pointeur.
const bar = $("#bar");
let dragging = false;
bar.addEventListener("pointerdown", (e) => {
  if (e.button !== 0 || e.target.closest("button")) return;
  dragging = true; bar.setPointerCapture(e.pointerId);
  invoke("overlay_drag", { phase: "start", x: e.screenX, y: e.screenY }).catch(() => {});
});
bar.addEventListener("pointermove", (e) => { if (dragging) invoke("overlay_drag", { phase: "move", x: e.screenX, y: e.screenY }).catch(() => {}); });
const endDrag = () => { if (dragging) { dragging = false; invoke("overlay_drag", { phase: "end", x: 0, y: 0 }).catch(() => {}); } };
bar.addEventListener("pointerup", endDrag); bar.addEventListener("pointercancel", endDrag);
document.addEventListener("contextmenu", (e) => e.preventDefault());

refresh();
setInterval(refresh, 500);
