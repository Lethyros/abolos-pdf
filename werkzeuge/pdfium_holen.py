#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Oliver Weber
# SPDX-License-Identifier: GPL-3.0-or-later
"""Lädt das vorgebaute PDFium (bblanchon/pdfium-binaries, ohne V8/XFA) nach third_party/pdfium.

Aufruf: werkzeuge/pdfium_holen.py linux-x64|win-x64 [zielordner]

Version und SHA256-Prüfsummen stehen fest hier im Skript: Eine andere oder veränderte Datei wird
abgelehnt (Lieferkette). Bei einem PDFium-Update Version, Prüfsummen, das pdfium_XXXX-Merkmal in
Cargo.toml und sbom/fremdkomponenten.json gemeinsam ändern.
"""
import hashlib
import io
import pathlib
import shutil
import sys
import tarfile
import urllib.request

VERSION = "chromium/7881"
PRUEFSUMMEN = {
    "linux-x64": "1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d",
    "win-x64": "73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08",
}


def main():
    if len(sys.argv) < 2 or sys.argv[1] not in PRUEFSUMMEN:
        sys.exit(f"Aufruf: {sys.argv[0]} {'|'.join(PRUEFSUMMEN)} [zielordner]")
    plattform = sys.argv[1]
    wurzel = pathlib.Path(__file__).resolve().parent.parent
    ziel = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else wurzel / "third_party" / "pdfium"
    url = f"https://github.com/bblanchon/pdfium-binaries/releases/download/{VERSION}/pdfium-{plattform}.tgz"
    print(f"Lade {url}", file=sys.stderr)
    with urllib.request.urlopen(url, timeout=120) as antwort:
        daten = antwort.read()
    summe = hashlib.sha256(daten).hexdigest()
    if summe != PRUEFSUMMEN[plattform]:
        sys.exit(f"Prüfsumme falsch: {summe}, erwartet {PRUEFSUMMEN[plattform]}. Datei verworfen.")
    if ziel.exists():
        shutil.rmtree(ziel)
    ziel.mkdir(parents=True)
    with tarfile.open(fileobj=io.BytesIO(daten), mode="r:gz") as archiv:
        archiv.extractall(ziel, filter="data")  # keine Pfade außerhalb, keine Sonderdateien
    print(f"PDFium {VERSION} ({plattform}) in {ziel}, SHA256 geprüft", file=sys.stderr)


if __name__ == "__main__":
    main()
