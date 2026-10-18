"""Génère les graphiques de performances (SVG, français et anglais) dans docs/benchmarks/.

Données : mesures du 18/10/2026 (Ryzen 7 5800X3D, RX 7800 XT 16 Go, enregistrement réel de 15 min 16 s, Whisper Large v3 Turbo
compressé, Qwen 3.5 Q4_K_M, 3 788 tokens lus / 300 écrits). Usage : python scripts/make-benchmark-charts.py
"""
import json
import os
from xml.sax.saxutils import escape

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "docs", "benchmarks")
os.makedirs(OUT, exist_ok=True)

# ── Données ──────────────────────────────────────────────────────────────
TR = [  # moteur, durée (s), RAM pic (Mo), VRAM pic (Mo), % processeur du PC
    ("rocm", 40.7, 405, 1240, 11),
    ("cpu", 588.1, 1093, 0, 46),
]
LLM = [  # modèle, moteur, durée, lecture tok/s, écriture tok/s, RAM pic Mo, RAM privée Mo, VRAM pic Mo, % processeur
    ("4B", "rocm", 6.4, 1909, 74.3, 3117, 462, 3369, 4),
    ("4B", "vulkan", 14.9, 696, 34.1, 3798, 1164, 2765, 2),
    ("4B", "cpu", 94.2, 63, 8.9, 4565, 1948, 0, 45),
    ("9B", "rocm", 8.4, 1330, 56.3, 5392, 473, 5723, 4),
    ("9B", "vulkan", 19.0, 490, 28.6, 6403, 1504, 5029, 2),
]
AUDIO_S = 916.0

COL = {"rocm": "#E5393C", "vulkan": "#8B5CF6", "cpu": "#64748B", "vram": "#F59E0B", "priv": "#3B82F6", "ram": "#BFDBFE"}
FONT = "Segoe UI, -apple-system, Helvetica, Arial, sans-serif"

T = {
    "fr": {
        "rocm": "ROCm (RX 7800 XT)", "vulkan": "Vulkan", "cpu": "Processeur", "cpu8": "Processeur (8 threads)", "rocm_s": "ROCm", "vulkan_s": "Vulkan", "cpu_s": "Processeur",
        "m4": "Qwen 3.5 4B", "m9": "Qwen 3.5 9B",
        "tr_title": "Transcription d'un enregistrement de 15 min 16 s", "tr_sub": "Whisper Large v3 Turbo compressé · plus c'est court, mieux c'est",
        "tr_fast": "14× plus rapide", "rt": "temps réel",
        "wr_title": "Rédaction du compte-rendu : durée", "wr_sub": "3 788 tokens lus, 300 écrits · plus c'est court, mieux c'est",
        "sp_title": "Rédaction du compte-rendu : vitesse", "sp_sub": "tokens par seconde · plus c'est long, mieux c'est",
        "read": "Lecture du texte", "write": "Écriture",
        "mem_title": "Mémoire utilisée au pic", "mem_sub": "VRAM de la carte graphique et RAM du PC",
        "vram": "VRAM", "priv": "RAM réservée par Verbatim", "ram": "RAM totale (modèle mappé inclus)",
        "wh_rocm": "Whisper · ROCm", "wh_cpu": "Whisper · Processeur",
        "cpu_title": "Charge du processeur pendant le traitement", "cpu_sub": "part du processeur total du PC (16 threads) · plus c'est bas, mieux c'est",
        "s": "s", "min": "min", "gb": "Go", "mb": "Mo",
    },
    "en": {
        "rocm": "ROCm (RX 7800 XT)", "vulkan": "Vulkan", "cpu": "Processor", "cpu8": "Processor (8 threads)", "rocm_s": "ROCm", "vulkan_s": "Vulkan", "cpu_s": "Processor",
        "m4": "Qwen 3.5 4B", "m9": "Qwen 3.5 9B",
        "tr_title": "Transcribing a 15 min 16 s recording", "tr_sub": "Whisper Large v3 Turbo compressed · shorter is better",
        "tr_fast": "14× faster", "rt": "real time",
        "wr_title": "Report writing: duration", "wr_sub": "3,788 tokens read, 300 written · shorter is better",
        "sp_title": "Report writing: speed", "sp_sub": "tokens per second · longer is better",
        "read": "Reading the text", "write": "Writing",
        "mem_title": "Memory used at peak", "mem_sub": "Graphics card VRAM and PC RAM",
        "vram": "VRAM", "priv": "RAM reserved by Verbatim", "ram": "Total RAM (mapped model included)",
        "wh_rocm": "Whisper · ROCm", "wh_cpu": "Whisper · Processor",
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
    return f"{num(v, lang, 1)} {t['s']}"


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
    x0, wmax = 230, 320
    body = ""
    rows = [("rocm", t["rocm"]), ("cpu", t["cpu8"])]
    for i, (k, label) in enumerate(rows):
        d = next(r[1] for r in TR if r[0] == k)
        y = 100 + i * 62
        rt = AUDIO_S / d
        body += bar(x0, y, wmax * d / 600.0, 34, COL[k], label, f"{dur(d, lang)}  ·  ×{num(rt, lang, 1)} {t['rt']}", bold=(k == "rocm"))
    body += f'<rect x="{x0 + 250}" y="108" width="132" height="26" rx="13" fill="#DCFCE7"/><text x="{x0 + 316}" y="126" font-size="13" font-weight="700" fill="#166534" text-anchor="middle">{escape(t["tr_fast"])}</text>'
    return svg(780, 232, t["tr_title"], t["tr_sub"], body)


def chart_redaction_duree(lang):
    t = T[lang]
    x0, wmax = 230, 440
    body, y = "", 96
    for model, mk in (("4B", "m4"), ("9B", "m9")):
        body += f'<text x="28" y="{y + 4}" font-size="14" font-weight="700" fill="#111827">{escape(t[mk])}</text>'
        y += 14
        for r in [r for r in LLM if r[0] == model]:
            body += bar(x0, y, wmax * r[2] / 100.0, 28, COL[r[1]], t[r[1]], dur(r[2], lang), bold=(r[1] == "rocm"))
            y += 40
        y += 14
    return svg(780, y + 6, t["wr_title"], t["wr_sub"], body)


def chart_redaction_vitesse(lang):
    t = T[lang]
    body = ""
    panels = [(t["read"], 3, 2200.0), (t["write"], 4, 80.0)]
    for pi, (ptitle, idx, vmax) in enumerate(panels):
        px = 28 + pi * 372
        body += f'<text x="{px}" y="100" font-size="14" font-weight="700" fill="#111827">{escape(ptitle)}</text>'
        y = 116
        for model, mk in (("4B", "m4"), ("9B", "m9")):
            body += f'<text x="{px}" y="{y + 12}" font-size="12" fill="#6B7280">{escape(t[mk])}</text>'
            y += 20
            for r in [r for r in LLM if r[0] == model]:
                v = r[idx]
                body += bar(px + 70, y, 190 * v / vmax, 22, COL[r[1]], None, f"{num(v, lang, 0 if v >= 100 else 1)}")
                body += f'<text x="{px + 62}" y="{y + 16}" font-size="11" fill="#374151" text-anchor="end">{escape(t[r[1] + "_s"])}</text>'
                y += 30
            y += 10
    h = y + 36
    body += legend([(COL["rocm"], "ROCm"), (COL["vulkan"], "Vulkan"), (COL["cpu"], t["cpu_s"])], 28, h - 14)
    return svg(780, h + 8, t["sp_title"], t["sp_sub"], body)


def chart_memoire(lang):
    t = T[lang]
    x0, wmax = 200, 470
    scale = 7000.0
    rows = [(t["wh_rocm"], 405, None, 1240), (t["wh_cpu"], 1093, None, 0)]
    for model, mk in (("4B", "m4"), ("9B", "m9")):
        for r in [r for r in LLM if r[0] == model]:
            rows.append((f"{t[mk].replace('Qwen 3.5 ', '')} · {t[r[1] + '_s']}", r[5], r[6], r[7]))
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
    body += legend([(COL["vram"], t["vram"]), (COL["priv"], t["priv"]), (COL["ram"], t["ram"])], 28, h - 16)
    return svg(780, h + 8, t["mem_title"], t["mem_sub"], body)


def chart_cpu(lang):
    t = T[lang]
    x0, wmax = 200, 470
    rows = [(t["wh_rocm"], 11, "rocm"), (t["wh_cpu"], 46, "cpu")]
    for model, mk in (("4B", "m4"), ("9B", "m9")):
        for r in [r for r in LLM if r[0] == model]:
            rows.append((f"{t[mk].replace('Qwen 3.5 ', '')} · {t[r[1] + '_s']}", r[8], r[1]))
    body, y = "", 96
    for label, pct, k in rows:
        body += bar(x0, y, wmax * pct / 50.0, 24, COL[k], label, f"{pct} %")
        y += 36
    return svg(780, y + 14, t["cpu_title"], t["cpu_sub"], body)


FILES = {
    "transcription": chart_transcription, "redaction-duree": chart_redaction_duree, "redaction-vitesse": chart_redaction_vitesse,
    "memoire": chart_memoire, "processeur": chart_cpu,
}
for lang in ("fr", "en"):
    for name, fn in FILES.items():
        with open(os.path.join(OUT, f"{name}.{lang}.svg"), "w", encoding="utf-8", newline="\n") as f:
            f.write(fn(lang))

# données brutes
data = {
    "date": "2026-10-18", "audio_s": AUDIO_S,
    "machine": "Ryzen 7 5800X3D (8 cores / 16 threads), 48 GB RAM, Radeon RX 7800 XT 16 GB",
    "transcription": [dict(zip(("engine", "duration_s", "ram_peak_mb", "vram_peak_mb", "cpu_pct_of_pc"), r)) for r in TR],
    "writing": [dict(zip(("model", "engine", "duration_s", "read_tok_s", "write_tok_s", "ram_peak_mb", "ram_private_mb", "vram_peak_mb", "cpu_pct_of_pc"), r)) for r in LLM],
    "app_idle": {"working_set_mb": 360, "private_mb": 106, "process_mb": 30, "cpu_pct": 0},
}
with open(os.path.join(OUT, "results.json"), "w", encoding="utf-8", newline="\n") as f:
    json.dump(data, f, ensure_ascii=False, indent=2)
print("ok", len(FILES) * 2, "graphiques")
