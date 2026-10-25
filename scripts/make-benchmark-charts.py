"""Génère les graphiques de performances (SVG, français et anglais) dans docs/benchmarks/.

Données : Ryzen 7 5800X3D, RX 7800 XT 16 Go, enregistrement réel de 15 min 16 s.
 - Transcription et direct : mesures du 25/10/2026 (version 0.4.0, Whisper Large v3 Turbo compressé, réglages de l'application).
 - Compte-rendu : mesures du 25/10/2026 par le vrai parcours de l'application (Qwen 3.5 Q4_K_M, parties de 11 000 caractères puis fusion).
Usage : python scripts/make-benchmark-charts.py
"""
import json
import os
from xml.sax.saxutils import escape

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "docs", "benchmarks")
os.makedirs(OUT, exist_ok=True)

# ── Données ──────────────────────────────────────────────────────────────
TR = [  # moteur, durée (s, moyenne de 2 passes ; 1 pour le processeur), RAM pic (Mo), RAM privée (Mo), VRAM pic (Mo), % processeur du PC (cœurs utilisés / 16)
    ("rocm", 22.4, 367, 325, 1460, 8),
    ("vulkan", 35.6, 699, 657, 982, 6),
    ("cpu", 278.5, 1275, 1270, 0, 50),
]
SRV = [  # direct (whisper-server) : moteur, démarrage (s), RAM (Mo), VRAM (Mo), 1er extrait de 10 s (s), extrait suivant (s, médiane)
    ("rocm", 1.64, 94, 1362, 0.58, 0.40),
    ("vulkan", 1.54, 335, 978, 0.74, 0.68),
    ("cpu", 1.02, 920, 0, 13.41, 13.36),
]
SEP_S = 38.0  # séparation des intervenants sur 15 min 16 s (s), processeur : 37,6 s et 38,4 s ; 1 h : 150 s
LLM = [  # modèle, moteur, durée 15 min (s, 1re rédaction après chargement), lecture tok/s, écriture tok/s, RAM pic Mo, (inutilisé), VRAM pic Mo, % processeur du PC, durée 1 h (s)
    ("4B", "rocm", 8.6, 3400, 105, 3819, None, 3368, 6, 20.1),
    ("4B", "vulkan", 26.2, 1040, 46, 4595, None, 2773, 3, 54.1),
    ("4B", "cpu", 149.3, 97, 10.3, 5269, None, 0, 50, None),
    ("9B", "rocm", 14.7, 2050, 74, 6099, None, 5719, 5, 37.0),
    ("9B", "vulkan", 35.4, 767, 39, 7137, None, 5038, 2, 83.1),
    ("27B", "rocm", 42.2, 655, 34, 12372, None, 11132, 4, 109.3),
    ("27B", "vulkan", 101.6, 250, 15, 13550, None, 10352, 2, 241.0),
    ("27B", "cpu", 1008.3, 16, 2.4, 13793, None, 0, 50, None),
]
# Versions précédentes (0.3.x : whisper.cpp 1.9.2 identique, mais -bs 5 et une seule instance), mesurées le 25/10/2026 :
# cas, durée avant (s), durée 0.4.0 (s), explication
PREV = [
    ("rocm15", 34.5, 22.4), ("rocm1h", 129.7, 45.0), ("cpu15", 346.5, 278.5), ("vk15", 346.5, 35.6),
]
# Rédaction : vrai parcours de l'application (parties de 11 000 caractères, notes par partie puis fusion), mesuré le 25/10/2026.
# 15 min : 2 parties + fusion (≈ 9 900 tokens lus, ≈ 570 à 930 écrits). 1 h : 6 parties + fusion (≈ 25 000 lus, ≈ 1 300 à 2 100 écrits).
AUDIO_S = 916.0

COL = {"st_tr": "#3B82F6", "st_sep": "#10B981", "st_wr": "#F59E0B", "rocm": "#E5393C", "vulkan": "#8B5CF6", "cpu": "#64748B", "vram": "#F59E0B", "priv": "#3B82F6", "ram": "#BFDBFE"}
FONT = "Segoe UI, -apple-system, Helvetica, Arial, sans-serif"

T = {
    "fr": {
        "rocm": "ROCm (RX 7800 XT)", "vulkan": "Vulkan", "cpu": "Processeur", "cpu8": "Processeur (8 threads)", "rocm_s": "ROCm", "vulkan_s": "Vulkan", "cpu_s": "Processeur",
        "m4": "Qwen 3.5 4B", "m9": "Qwen 3.5 9B", "m27": "Qwen 3.8 27B",
        "tr_title": "Transcription d'un enregistrement de 15 min 16 s", "tr_sub": "Whisper Large v3 Turbo compressé · plus c'est court, mieux c'est",
        "tr_fast": "{n}× plus rapide", "rt": "temps réel",
        "ev_title": "Transcription : gain par rapport aux versions 0.3.x", "ev_sub": "mêmes moteurs, réglages améliorés · plus c'est court, mieux c'est",
        "ev_rocm15": "ROCm · 15 min", "ev_rocm1h": "ROCm · 1 h", "ev_cpu15": "Processeur · 15 min", "ev_vk15": "AMD avant RX 7000, Intel · 15 min",
        "ev_old": "avant", "ev_new": "0.4.0", "ev_old_vk": "processeur",
        "pl_title": "De l'enregistrement au compte-rendu", "pl_sub": "enregistrement de 15 min 16 s, séparation des intervenants comprise · plus c'est court, mieux c'est",
        "st_tr": "Transcription", "st_sep": "Séparation des intervenants", "st_wr": "Compte-rendu (Qwen 3.5 4B)",
        "lv_title": "Transcription en direct : temps pour un extrait de 10 s", "lv_sub": "serveur Whisper gardé en mémoire · en dessous de la ligne, le texte suit la voix",
        "lv_line": "10 s : durée de l'extrait", "lv_late": "trop lent pour le direct", "lv_ok": "fluide",
        "wr_title": "Rédaction du compte-rendu : durée", "wr_sub": "enregistrement de 15 min 16 s (2 parties puis fusion) · entre parenthèses, une réunion d'1 h · plus c'est court, mieux c'est",
        "sp_title": "Rédaction du compte-rendu : vitesse", "sp_sub": "tokens par seconde, mesurés sur une réunion d'1 h (cartes) ou 15 min (processeur) · plus c'est long, mieux c'est",
        "read": "Lecture du texte", "write": "Écriture",
        "mem_title": "Mémoire utilisée au pic", "mem_sub": "VRAM de la carte graphique et RAM du PC",
        "vram": "VRAM", "priv": "RAM réservée par Verbatim", "ram": "RAM totale (modèle mappé inclus)",
        "wh_rocm": "Whisper · ROCm", "wh_vulkan": "Whisper · Vulkan", "wh_cpu": "Whisper · Processeur",
        "cpu_title": "Charge du processeur pendant le traitement", "cpu_sub": "part du processeur total du PC (16 threads) · plus c'est bas, mieux c'est",
        "s": "s", "min": "min", "gb": "Go", "mb": "Mo",
    },
    "en": {
        "rocm": "ROCm (RX 7800 XT)", "vulkan": "Vulkan", "cpu": "Processor", "cpu8": "Processor (8 threads)", "rocm_s": "ROCm", "vulkan_s": "Vulkan", "cpu_s": "Processor",
        "m4": "Qwen 3.5 4B", "m9": "Qwen 3.5 9B", "m27": "Qwen 3.8 27B",
        "tr_title": "Transcribing a 15 min 16 s recording", "tr_sub": "Whisper Large v3 Turbo compressed · shorter is better",
        "tr_fast": "{n}× faster", "rt": "real time",
        "ev_title": "Transcription: gain over versions 0.3.x", "ev_sub": "same engines, improved settings · shorter is better",
        "ev_rocm15": "ROCm · 15 min", "ev_rocm1h": "ROCm · 1 h", "ev_cpu15": "Processor · 15 min", "ev_vk15": "AMD before RX 7000, Intel · 15 min",
        "ev_old": "before", "ev_new": "0.4.0", "ev_old_vk": "processor",
        "pl_title": "From recording to meeting report", "pl_sub": "15 min 16 s recording, speaker separation included · shorter is better",
        "st_tr": "Transcription", "st_sep": "Speaker separation", "st_wr": "Report (Qwen 3.5 4B)",
        "lv_title": "Live transcription: time for a 10 s excerpt", "lv_sub": "Whisper server kept in memory · below the line, the text keeps up with the voice",
        "lv_line": "10 s: length of the excerpt", "lv_late": "too slow for live", "lv_ok": "smooth",
        "wr_title": "Report writing: duration", "wr_sub": "15 min 16 s recording (2 parts then merge) · in brackets, a 1 h meeting · shorter is better",
        "sp_title": "Report writing: speed", "sp_sub": "tokens per second, measured on a 1 h meeting (graphics cards) or 15 min (processor) · longer is better",
        "read": "Reading the text", "write": "Writing",
        "mem_title": "Memory used at peak", "mem_sub": "Graphics card VRAM and PC RAM",
        "vram": "VRAM", "priv": "RAM reserved by Verbatim", "ram": "Total RAM (mapped model included)",
        "wh_rocm": "Whisper · ROCm", "wh_vulkan": "Whisper · Vulkan", "wh_cpu": "Whisper · Processor",
        "cpu_title": "Processor load during processing", "cpu_sub": "share of the PC's total processor (16 threads) · lower is better",
        "s": "s", "min": "min", "gb": "GB", "mb": "MB",
    },
}


def num(v, lang, nd=0):
    s = f"{v:,.{nd}f}"
    return s.replace(",", " ") .replace(".", ",") if lang == "fr" else s


def dur(v, lang):
    t = T[lang]
    if v >= 120:
        m, s = divmod(round(v), 60)
        return f"{m} {t['min']} {s:02d} {t['s']}"
    return f"{num(v, lang, 0 if v >= 20 else 1)} {t['s']}"


def svg(w, h, title, sub, body):
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="{FONT}" role="img" aria-label="{escape(title)}">
<rect x="0.5" y="0.5" width="{w - 1}" height="{h - 1}" rx="14" fill="#FFFFFF" stroke="#E5E7EB"/>
<rect x="0" y="0" width="{w}" height="6" rx="3" fill="url(#g)"/>
<defs><linearGradient id="g" x1="0" x2="1"><stop offset="0" stop-color="#E5393C"/><stop offset="0.5" stop-color="#8B5CF6"/><stop offset="1" stop-color="#3B82F6"/></linearGradient></defs>
<text x="28" y="44" font-size="20" font-weight="700" fill="#111827">{escape(title)}</text>
<text x="28" y="66" font-size="13" fill="#6B7280">{escape(sub)}</text>
{body}
</svg>
'''


def bar(x, y, wpx, h, color, label_left=None, value=None, lw=190, bold=False):
    out = ""
    if label_left:
        out += f'<text x="{x - 12}" y="{y + h / 2 + 5}" font-size="13" fill="#374151" text-anchor="end">{escape(label_left)}</text>'
    out += f'<rect x="{x}" y="{y}" width="{max(wpx, 3):.1f}" height="{h}" rx="6" fill="{color}"/>'
    if value:
        out += f'<text x="{x + max(wpx, 3) + 10:.1f}" y="{y + h / 2 + 5}" font-size="13" font-weight="{700 if bold else 600}" fill="#111827">{escape(value)}</text>'
    return out


def legend(items, x, y):
    out, cx = "", x
    for color, text in items:
        out += f'<rect x="{cx}" y="{y - 10}" width="12" height="12" rx="3" fill="{color}"/><text x="{cx + 18}" y="{y}" font-size="12" fill="#374151">{escape(text)}</text>'
        cx += 44 + len(text) * 7
    return out


def chart_transcription(lang):
    t = T[lang]
    x0, wmax = 230, 300
    body = ""
    rows = [("rocm", t["rocm"]), ("vulkan", t["vulkan"]), ("cpu", t["cpu8"])]
    dmax = max(r[1] for r in TR)
    for i, (k, label) in enumerate(rows):
        d = next(r[1] for r in TR if r[0] == k)
        y = 100 + i * 58
        body += bar(x0, y, wmax * d / dmax, 34, COL[k], label, f"{dur(d, lang)}  ·  ×{num(AUDIO_S / d, lang, 1)} {t['rt']}", bold=(k == "rocm"))
    n = round(next(r[1] for r in TR if r[0] == "cpu") / next(r[1] for r in TR if r[0] == "rocm"))
    body += f'<rect x="{x0 + 250}" y="108" width="132" height="26" rx="13" fill="#DCFCE7"/><text x="{x0 + 316}" y="126" font-size="13" font-weight="700" fill="#166534" text-anchor="middle">{escape(t["tr_fast"].format(n=n))}</text>'
    return svg(780, 100 + 3 * 58 + 8, t["tr_title"], t["tr_sub"], body)


def chart_evolution(lang):
    t = T[lang]
    x0, wmax, vmax = 250, 330, 350.0
    body, y = "", 96
    for k, old, new in PREV:
        body += f'<text x="28" y="{y + 4}" font-size="14" font-weight="700" fill="#111827">{escape(t["ev_" + k])}</text>'
        y += 12
        olabel = t["ev_old_vk"] if k == "vk15" else t["ev_old"]
        body += bar(x0, y, wmax * old / vmax, 24, "#CBD5E1", olabel, dur(old, lang))
        y += 32
        col = {"rocm15": COL["rocm"], "rocm1h": COL["rocm"], "cpu15": COL["cpu"], "vk15": COL["vulkan"]}[k]
        body += bar(x0, y, wmax * new / vmax, 24, col, "Vulkan 0.4.0" if k == "vk15" else t["ev_new"], f"{dur(new, lang)}", bold=True)
        body += f'<rect x="{x0 + wmax + 90}" y="{y - 14}" width="62" height="24" rx="12" fill="#DCFCE7"/><text x="{x0 + wmax + 121}" y="{y + 3}" font-size="13" font-weight="700" fill="#166534" text-anchor="middle">×{num(old / new, lang, 1)}</text>'
        y += 40
    return svg(780, y + 6, t["ev_title"], t["ev_sub"], body)


def chart_pipeline(lang):
    t = T[lang]
    x0, wmax = 150, 440
    wr = {r[1]: r[2] for r in LLM if r[0] == "4B"}  # Qwen 3.5 4B
    tot = {k: next(r[1] for r in TR if r[0] == k) + SEP_S + wr[k] for k in wr}
    tmax = max(tot.values())
    body, y = "", 100
    for k in ("rocm", "vulkan", "cpu"):
        parts = [("st_tr", next(r[1] for r in TR if r[0] == k)), ("st_sep", SEP_S), ("st_wr", wr[k])]
        body += f'<text x="{x0 - 12}" y="{y + 22}" font-size="13" fill="#374151" text-anchor="end">{escape(t[k + "_s"])}</text>'
        x = x0
        for sk, v in parts:
            if v <= 0:
                continue
            w = wmax * v / tmax
            body += f'<rect x="{x:.1f}" y="{y}" width="{max(w - 2, 2):.1f}" height="30" rx="4" fill="{COL[sk]}"/>'
            if w >= 54:
                body += f'<text x="{x + w / 2 - 1:.1f}" y="{y + 20}" font-size="12" font-weight="600" fill="#FFFFFF" text-anchor="middle">{dur(v, lang)}</text>'
            x += w
        body += f'<text x="{x + 10:.1f}" y="{y + 20}" font-size="13" font-weight="700" fill="#111827">{dur(tot[k], lang)}</text>'
        y += 48
    h = y + 36
    body += legend([(COL["st_tr"], t["st_tr"]), (COL["st_sep"], t["st_sep"]), (COL["st_wr"], t["st_wr"])], 28, h - 14)
    return svg(780, h + 8, t["pl_title"], t["pl_sub"], body)


def chart_direct(lang):
    t = T[lang]
    x0, wmax, vmax = 200, 400, 14.0
    body, y = "", 112
    xl = x0 + wmax * 10.0 / vmax
    for k in ("rocm", "vulkan", "cpu"):
        r = next(r for r in SRV if r[0] == k)
        v = r[5]
        body += bar(x0, y, wmax * v / vmax, 30, COL[k], t[k] if k != "cpu" else t["cpu"], f"{num(v, lang, 2 if v < 1 else 1)} {t['s']}  ·  {t['lv_ok'] if v < 10 else t['lv_late']}", bold=(k == "rocm"))
        y += 48
    body += f'<line x1="{xl:.1f}" y1="92" x2="{xl:.1f}" y2="{y - 10}" stroke="#9CA3AF" stroke-width="1.5" stroke-dasharray="4 4"/>'
    body += f'<text x="{xl:.1f}" y="86" font-size="12" fill="#6B7280" text-anchor="middle">{escape(t["lv_line"])}</text>'
    return svg(780, y + 14, t["lv_title"], t["lv_sub"], body)


def chart_redaction_duree(lang):
    t = T[lang]
    x0, wmax = 230, 440
    body, y = "", 96
    for model, mk in (("4B", "m4"), ("9B", "m9"), ("27B", "m27")):
        body += f'<text x="28" y="{y + 4}" font-size="14" font-weight="700" fill="#111827">{escape(t[mk])}</text>'
        y += 14
        for r in [r for r in LLM if r[0] == model]:
            if r[2] > 160:  # barre tronquée (coupure en biais)
                body += f'<rect x="{x0}" y="{y}" width="{wmax}" height="28" rx="6" fill="{COL[r[1]]}"/><path d="M{x0 + wmax - 30} {y + 30} L{x0 + wmax - 16} {y - 2}" stroke="#FFFFFF" stroke-width="5"/>'
                body += f'<text x="{x0 + wmax + 10}" y="{y + 19}" font-size="13" font-weight="600" fill="#111827">{dur(r[2], lang)}</text>'
                body += f'<text x="{x0 - 12}" y="{y + 19}" font-size="13" fill="#374151" text-anchor="end">{escape(t[r[1]])}</text>'
                y += 40
                continue
            body += bar(x0, y, wmax * r[2] / 160.0, 28, COL[r[1]], t[r[1]], dur(r[2], lang) + (f"  ({dur(r[9], lang)})" if r[9] else ""), bold=(r[1] == "rocm"))
            y += 40
        y += 14
    return svg(780, y + 6, t["wr_title"], t["wr_sub"], body)


def chart_redaction_vitesse(lang):
    t = T[lang]
    body = ""
    panels = [(t["read"], 3, 3600.0), (t["write"], 4, 115.0)]
    for pi, (ptitle, idx, vmax) in enumerate(panels):
        px = 28 + pi * 372
        body += f'<text x="{px}" y="100" font-size="14" font-weight="700" fill="#111827">{escape(ptitle)}</text>'
        y = 116
        for model, mk in (("4B", "m4"), ("9B", "m9"), ("27B", "m27")):
            body += f'<text x="{px}" y="{y + 12}" font-size="12" fill="#6B7280">{escape(t[mk])}</text>'
            y += 20
            for r in [r for r in LLM if r[0] == model]:
                v = r[idx]
                body += bar(px + 70, y, 190 * v / vmax, 22, COL[r[1]], None, f"{num(v, lang, 0 if v >= 10 else 1)}")
                body += f'<text x="{px + 62}" y="{y + 16}" font-size="11" fill="#374151" text-anchor="end">{escape(t[r[1] + "_s"])}</text>'
                y += 30
            y += 10
    h = y + 36
    body += legend([(COL["rocm"], "ROCm"), (COL["vulkan"], "Vulkan"), (COL["cpu"], t["cpu_s"])], 28, h - 14)
    return svg(780, h + 8, t["sp_title"], t["sp_sub"], body)


def chart_memoire(lang):
    t = T[lang]
    x0, wmax = 200, 470
    scale = 14000.0
    rows = [(t["wh_" + r[0]], r[2], None, r[4]) for r in TR]
    for model, mk in (("4B", "m4"), ("9B", "m9"), ("27B", "m27")):
        for r in [r for r in LLM if r[0] == model]:
            rows.append((f"{t[mk].replace('Qwen 3.5 ', '').replace('Qwen 3.8 ', '')} · {t[r[1] + '_s']}", r[5], r[6], r[7]))
    body, y = "", 100
    for label, ram, priv, vram in rows:
        body += f'<text x="{x0 - 12}" y="{y + 28}" font-size="13" fill="#374151" text-anchor="end">{escape(label)}</text>'
        bars = [(vram, COL["vram"])] + ([(priv, COL["priv"])] if priv is not None else []) + [(ram, COL["ram"])]
        for j, (v, color) in enumerate(bars):
            if v <= 0 and j != 0:
                continue
            yy = y + j * 17
            bw = wmax * v / scale
            body += f'<rect x="{x0}" y="{yy}" width="{max(bw, 2):.1f}" height="13" rx="4" fill="{color}"/>'
            txt = "–" if v <= 0 else f"{num(v / 1000, lang, 1)} {t['gb']}"
            body += f'<text x="{x0 + max(bw, 2) + 8:.1f}" y="{yy + 11}" font-size="11" fill="#374151">{txt}</text>'
        y += 66
    h = y + 40
    body += legend([(COL["vram"], t["vram"]), (COL["ram"], t["ram"])], 28, h - 16)
    return svg(780, h + 8, t["mem_title"], t["mem_sub"], body)


def chart_cpu(lang):
    t = T[lang]
    x0, wmax = 200, 470
    rows = [(t["wh_" + r[0]], r[5], r[0]) for r in TR]
    for model, mk in (("4B", "m4"), ("9B", "m9"), ("27B", "m27")):
        for r in [r for r in LLM if r[0] == model]:
            rows.append((f"{t[mk].replace('Qwen 3.5 ', '').replace('Qwen 3.8 ', '')} · {t[r[1] + '_s']}", r[8], r[1]))
    body, y = "", 96
    for label, pct, k in rows:
        body += bar(x0, y, wmax * pct / 55.0, 24, COL[k], label, f"{pct} %")
        y += 36
    return svg(780, y + 14, t["cpu_title"], t["cpu_sub"], body)


FILES = {
    "transcription": chart_transcription, "evolution": chart_evolution, "pipeline": chart_pipeline, "direct": chart_direct, "redaction-duree": chart_redaction_duree, "redaction-vitesse": chart_redaction_vitesse,
    "memoire": chart_memoire, "processeur": chart_cpu,
}
for lang in ("fr", "en"):
    for name, fn in FILES.items():
        with open(os.path.join(OUT, f"{name}.{lang}.svg"), "w", encoding="utf-8", newline="\n") as f:
            f.write(fn(lang))

# données brutes
data = {
    "date": "2026-10-25 (transcription, direct) / 2026-10-18 (compte-rendu)", "audio_s": AUDIO_S,
    "machine": "Ryzen 7 5800X3D (8 cores / 16 threads), 48 GB RAM, Radeon RX 7800 XT 16 GB",
    "transcription": [dict(zip(("engine", "duration_s", "ram_peak_mb", "ram_private_mb", "vram_peak_mb", "cpu_pct_of_pc"), r)) for r in TR],
    "live": [dict(zip(("engine", "start_s", "ram_mb", "vram_mb", "first_10s_excerpt_s", "next_10s_excerpt_s"), r)) for r in SRV],
    "speaker_separation_s": SEP_S,
    "previous_versions_transcription_s": [dict(zip(("case", "before_0_3_x_s", "v0_4_0_s"), r)) for r in PREV],
    "writing": [{k: v for k, v in zip(("model", "engine", "duration_15min_s", "read_tok_s", "write_tok_s", "ram_peak_mb", "_", "vram_peak_mb", "cpu_pct_of_pc", "duration_1h_s"), r) if k != "_"} for r in LLM],
    "app_idle": {"working_set_mb": 360, "private_mb": 106, "process_mb": 30, "cpu_pct": 0},
}
with open(os.path.join(OUT, "results.json"), "w", encoding="utf-8", newline="\n") as f:
    json.dump(data, f, ensure_ascii=False, indent=2)
print("ok", len(FILES) * 2, "graphiques")
