"""Dessine le logo de Verbatim (tuile d'application + forme d'onde) et exporte icônes et favicon.

    python scripts/make-logo.py

Rendu raster supersamplé : dégradés, lueur, ombre portée, reflet. Les petites tailles (<= 48 px)
utilisent un dessin simplifié (barres plus épaisses, sans liseré) pour rester lisibles.
"""
import os
import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICONS = os.path.join(ROOT, "src-tauri", "icons")
UI = os.path.join(ROOT, "ui")


def hexrgb(h):
    h = h.lstrip("#")
    return np.array([int(h[i:i + 2], 16) for i in (0, 2, 4)], dtype=np.float32)


def lerp_stops(t, stops):
    """Dégradé multi-arrêts ; t : tableau [0..1], stops : [(pos, '#rrggbb')]."""
    out = np.zeros(t.shape + (3,), dtype=np.float32)
    for (p0, c0), (p1, c1) in zip(stops, stops[1:]):
        m = (t >= p0) & (t <= p1)
        u = ((t - p0) / (p1 - p0))[..., None]
        out = np.where(m[..., None], hexrgb(c0) * (1 - u) + hexrgb(c1) * u, out)
    out = np.where((t < stops[0][0])[..., None], hexrgb(stops[0][1]), out)
    out = np.where((t > stops[-1][0])[..., None], hexrgb(stops[-1][1]), out)
    return out


def squircle_mask(size, margin, n=5.0):
    """Tuile « squircle » (superellipse) antialiasée."""
    s = size * 4
    y, x = np.mgrid[0:s, 0:s].astype(np.float32)
    c = (s - 1) / 2
    r = (s / 2) * (1 - 2 * margin)
    d = (np.abs(x - c) / r) ** n + (np.abs(y - c) / r) ** n
    m = Image.fromarray(((d <= 1.0) * 255).astype(np.uint8), "L")
    return m.resize((size, size), Image.LANCZOS)


def radial(size, cx, cy, radius, power=1.6):
    y, x = np.mgrid[0:size, 0:size].astype(np.float32)
    d = np.sqrt((x - cx * size) ** 2 + (y - cy * size) ** 2) / (radius * size)
    return np.clip(1 - d, 0, 1) ** power


def render(size, small=False):
    S = size
    margin = 0.045 if not small else 0.02
    tile = squircle_mask(S, margin)

    # ── fond : dégradé diagonal indigo → violet, lueur turquoise en bas à droite
    y, x = np.mgrid[0:S, 0:S].astype(np.float32)
    t = (x * 0.42 + y * 0.58) / S
    bg = lerp_stops(t, [(0.0, "#1b1466"), (0.45, "#3f33c9"), (1.0, "#7a5bff")])
    glow_t = radial(S, 0.82, 0.92, 0.62, 1.7)[..., None]
    bg = bg * (1 - glow_t * 0.55) + hexrgb("#14d1bd") * (glow_t * 0.55)
    light = radial(S, 0.22, 0.12, 0.70, 1.8)[..., None]
    bg = bg + (hexrgb("#b9b0ff") - bg) * (light * 0.16)
    img = Image.fromarray(np.clip(bg, 0, 255).astype(np.uint8), "RGB").convert("RGBA")

    # ── reflet de verre sur la moitié haute
    if not small:
        gl = Image.new("L", (S, S), 0)
        ImageDraw.Draw(gl).ellipse([-0.25 * S, -0.62 * S, 1.25 * S, 0.50 * S], fill=255)
        gl = gl.filter(ImageFilter.GaussianBlur(S * 0.004))
        fade = np.clip(1 - y / (0.52 * S), 0, 1) ** 1.3
        gla = (np.asarray(gl, dtype=np.float32) / 255 * fade * 0.13 * 255).astype(np.uint8)
        white = Image.new("RGBA", (S, S), (255, 255, 255, 0))
        white.putalpha(Image.fromarray(gla, "L"))
        img = Image.alpha_composite(img, white)

    # ── forme d'onde : 5 barres, bouts arrondis, hauteurs d'origine (courte, haute, la plus haute, moyenne, courte)
    heights = [0.26, 0.47, 0.68, 0.40, 0.21] if not small else [0.30, 0.50, 0.70, 0.44, 0.26]
    bar_w = (0.082 if not small else 0.118) * S
    gap = (0.058 if not small else 0.052) * S
    total = 5 * bar_w + 4 * gap
    x0 = (S - total) / 2
    cy = S * 0.5
    bars = Image.new("L", (S, S), 0)
    bd = ImageDraw.Draw(bars)
    for i, h in enumerate(heights):
        bx = x0 + i * (bar_w + gap)
        hh = h * S
        bd.rounded_rectangle([bx, cy - hh / 2, bx + bar_w, cy + hh / 2], radius=bar_w / 2, fill=255)

    # ombre portée douce
    sh = bars.filter(ImageFilter.GaussianBlur(S * 0.022))
    sh = ImageChops.offset(sh, 0, int(S * 0.018))
    shadow = Image.new("RGBA", (S, S), (14, 8, 70, 0))
    shadow.putalpha(sh.point(lambda v: int(v * 0.55)))
    img = Image.alpha_composite(img, shadow)

    # lueur turquoise derrière les barres
    if not small:
        gb = bars.filter(ImageFilter.GaussianBlur(S * 0.035))
        gcol = Image.new("RGBA", (S, S), (45, 230, 210, 0))
        gcol.putalpha(gb.point(lambda v: int(v * 0.60)))
        img = Image.alpha_composite(img, gcol)

    # remplissage des barres : blanc → turquoise clair (haut vers bas)
    ty = np.clip((y - (cy - 0.36 * S)) / (0.72 * S), 0, 1)
    fill = lerp_stops(ty, [(0.0, "#ffffff"), (0.55, "#e9fbff"), (1.0, "#8ff0e2")])
    fimg = Image.fromarray(fill.astype(np.uint8), "RGB").convert("RGBA")
    fimg.putalpha(bars)
    img = Image.alpha_composite(img, fimg)

    # liseré intérieur lumineux (haut) / sombre (bas)
    if not small:
        edge = ImageChops.subtract(tile, tile.filter(ImageFilter.MinFilter(max(3, int(S * 0.012)) | 1)))
        et = np.clip(y / S, 0, 1)
        ea = (np.asarray(edge, dtype=np.float32) / 255) * (0.55 * (1 - et) ** 1.2 + 0.06) * 255
        ecol = Image.new("RGBA", (S, S), (255, 255, 255, 0))
        ecol.putalpha(Image.fromarray(np.clip(ea, 0, 255).astype(np.uint8), "L"))
        img = Image.alpha_composite(img, ecol)

    img.putalpha(ImageChops.multiply(img.getchannel("A"), tile))
    return img


def main():
    master = render(1024)
    big = {s: master.resize((s, s), Image.LANCZOS) for s in (512, 256, 128, 96)}
    mid = {s: render(s) for s in (64, 48)}
    small = {s: render(s * 4, small=True).resize((s, s), Image.LANCZOS) for s in (32, 24, 16)}

    os.makedirs(ICONS, exist_ok=True)
    big[512].save(os.path.join(ICONS, "icon.png"))
    big[256].save(os.path.join(ICONS, "128x128@2x.png"))
    big[128].save(os.path.join(ICONS, "128x128.png"))
    small[32].save(os.path.join(ICONS, "32x32.png"))
    frames = {**small, **mid, 128: big[128], 256: big[256]}
    ordered = [frames[s] for s in (16, 24, 32, 48, 64, 128, 256)]
    ordered[-1].save(os.path.join(ICONS, "icon.ico"), format="ICO", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)], append_images=ordered[:-1])

    # interface : logo de la barre latérale + favicon
    big[96].save(os.path.join(UI, "logo.png"))
    small[32].save(os.path.join(UI, "favicon.png"))
    ordered[-1].save(os.path.join(UI, "favicon.ico"), format="ICO", sizes=[(s, s) for s in (16, 32, 48)], append_images=[])
    print("logo exporté :", ICONS, UI)


if __name__ == "__main__":
    main()
