#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Oliver Weber
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt alle Programmsymbole aus einem Bild (z.B. aus Z-Image):

    werkzeuge/symbol.py vorlage.png

- stellt die runde Kachel frei (Ecken außerhalb durchsichtig), erkannt an den kräftigen Farben
- schreibt paket/symbol/abolos-pdf.png (512 px, Vorlage für alles), abolos-pdf-<g>.png für
  16 bis 512 px (Linux-Symbolthema) und abolos-pdf.ico (Windows: Programm, Installer, Taskleiste)

Braucht Pillow und NumPy.
"""
import pathlib
import sys

import numpy as np
from PIL import Image, ImageDraw

GROESSEN = [16, 24, 32, 48, 64, 128, 256, 512]
ICO_GROESSEN = [16, 24, 32, 48, 64, 128, 256]


def freistellen(bild: Image.Image) -> Image.Image:
    """Kachel = Bereich kräftiger Farben; außen herum durchsichtig, Ecken gerundet."""
    rgb = np.asarray(bild.convert("RGB")).astype(int)
    hell, dunkel = rgb.max(axis=2), rgb.min(axis=2)
    saettigung = (hell - dunkel) / np.maximum(hell, 1)
    farbig = (saettigung > 0.45) & (hell > 90)
    ys, xs = np.where(farbig)
    if len(xs) == 0:
        raise SystemExit("keine farbige Kachel gefunden")
    x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
    b, h = bild.size
    if x0 <= 2 and y0 <= 2 and x1 >= b - 3 and y1 >= h - 3:  # Farbe bis zum Rand: kleinen Rand lassen
        x0, y0, x1, y1 = 8, 8, b - 9, h - 9
    fein = 4  # Maske vierfach zeichnen und verkleinern: weiche Kanten
    maske = Image.new("L", (b * fein, h * fein), 0)
    radius = int(0.215 * (x1 - x0) * fein)
    ImageDraw.Draw(maske).rounded_rectangle(
        (x0 * fein, y0 * fein, (x1 + 1) * fein, (y1 + 1) * fein), radius=radius, fill=255
    )
    rgba = bild.convert("RGBA")
    rgba.putalpha(maske.resize((b, h), Image.LANCZOS))
    rgba = rgba.crop((x0, y0, x1 + 1, y1 + 1))
    # quadratisch, Kachel füllt ~94 % (übliches Symbolraster)
    seite = int(max(rgba.size) / 0.94)
    quadrat = Image.new("RGBA", (seite, seite), (0, 0, 0, 0))
    quadrat.paste(rgba, ((seite - rgba.width) // 2, (seite - rgba.height) // 2), rgba)
    return quadrat


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    ziel = pathlib.Path(__file__).resolve().parent.parent / "paket" / "symbol"
    ziel.mkdir(parents=True, exist_ok=True)
    symbol = freistellen(Image.open(sys.argv[1]))
    symbol.resize((512, 512), Image.LANCZOS).save(ziel / "abolos-pdf.png")
    for g in GROESSEN:
        symbol.resize((g, g), Image.LANCZOS).save(ziel / f"abolos-pdf-{g}.png")
    symbol.resize((256, 256), Image.LANCZOS).save(
        ziel / "abolos-pdf.ico", sizes=[(g, g) for g in ICO_GROESSEN]
    )
    print(f"Symbole in {ziel}: {', '.join(map(str, GROESSEN))} px und abolos-pdf.ico", file=sys.stderr)


if __name__ == "__main__":
    main()
