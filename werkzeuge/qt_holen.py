#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Oliver Weber
# SPDX-License-Identifier: GPL-3.0-or-later
"""Lädt das vorgebaute Qt aus dem offiziellen Qt-Paketserver (download.qt.io) und entpackt es.

Aufruf: werkzeuge/qt_holen.py <version> <architektur> <zielordner>
  z.B.  werkzeuge/qt_holen.py 6.12.0 win64_msvc2022_64 build/qt
        werkzeuge/qt_holen.py 6.12.0 linux_gcc_64 build/qt
Ausgabe (letzte Zeile): Installationsordner mit bin/, lib/, qml/ (neuer Serveraufbau: der Zielordner
selbst; alter Aufbau: <ziel>/<version>/<compiler>)

Warum nicht aqtinstall: Qt hat den Paketserver umgebaut (Windows-Pakete liegen seit 6.12 bzw. auch
rückwirkend für 6.11 in eigenen Ordnern je Compiler); aqtinstall 3.3 findet sie nicht (aqtinstall #1047).
Dieses Skript versteht den alten und den neuen Aufbau.

Entpacken: unter Windows mit 7-Zip, unter Linux mit py7zr (pip install py7zr).

Sicherheit: Paketliste und jedes Archiv werden gegen die vom Qt-Server veröffentlichten Prüfsummen
geprüft (Updates.xml: SHA256, Archive: SHA1), alles über HTTPS. Geladen werden nur die Teile, die
Abolos PDF braucht (Qt Base, Declarative, Tools, SVG und die Laufzeit-Ergänzungen), keine Doku,
keine Übersetzungen, keine Beispiele.
"""
import hashlib
import pathlib
import shutil
import subprocess
import sys
import urllib.request
import xml.etree.ElementTree as ET

SERVER = "https://download.qt.io/online/qtsdkrepository"
# Archive, die nicht gebraucht werden (Anfang des Archivnamens)
WEGLASSEN = ("qtdoc", "qttranslations", "qtexamples")


def laden(url: str) -> bytes:
    anfrage = urllib.request.Request(url, headers={"User-Agent": "abolos-pdf-qt-holen"})
    with urllib.request.urlopen(anfrage, timeout=300) as antwort:
        return antwort.read()


def text(url: str) -> str:
    return laden(url).decode("utf-8").strip()


def paketliste(host: str, code: str, arch: str):
    """Updates.xml finden (neuer Aufbau: Ordner je Compiler; alter: ein Ordner) und prüfen"""
    kurz = arch.split("_", 1)[1] if arch.startswith("win") else arch  # win64_msvc2022_64 -> msvc2022_64
    for ordner in (f"qt6_{code}/qt6_{code}_{kurz}", f"qt6_{code}/qt6_{code}"):
        basis = f"{SERVER}/{host}/desktop/{ordner}"
        try:
            daten = laden(f"{basis}/Updates.xml")
        except OSError:
            continue
        summe = text(f"{basis}/Updates.xml.sha256").split()[0]
        if hashlib.sha256(daten).hexdigest() != summe:
            sys.exit(f"Prüfsumme der Paketliste falsch: {basis}/Updates.xml")
        return basis, ET.fromstring(daten)
    sys.exit(f"Keine Paketliste für Qt {code} ({host}, {arch}) gefunden")


def entpacken(archiv: pathlib.Path, ziel: pathlib.Path):
    """Windows: 7-Zip (schnell). Linux: py7zr, denn die Linux-Archive enthalten verkettete
    Verknüpfungen (libQt6Core.so -> .so.6 -> .so.6.12.0), die neuere 7-Zip-Versionen verweigern."""
    sieben = shutil.which("7z") or shutil.which("7za")
    if sys.platform == "win32" and sieben:
        subprocess.run([sieben, "x", "-y", f"-o{ziel}", str(archiv)], check=True, stdout=subprocess.DEVNULL)
        return
    import py7zr  # pip install py7zr

    with py7zr.SevenZipFile(archiv) as a:
        a.extractall(ziel)


def main():
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    version, arch, ziel = sys.argv[1], sys.argv[2], pathlib.Path(sys.argv[3]).resolve()
    host = "windows_x86" if arch.startswith("win") else "linux_x64"
    code = version.replace(".", "")
    basis, wurzel = paketliste(host, code, arch)
    name = f"qt.qt6.{code}.{arch}"
    paket = next((p for p in wurzel.findall("PackageUpdate") if p.findtext("Name") == name), None)
    if paket is None:
        sys.exit(f"Paket {name} nicht in der Paketliste")
    paketversion = paket.findtext("Version")
    archive = [a.strip() for a in (paket.findtext("DownloadableArchives") or "").split(",") if a.strip()]
    archive = [a for a in archive if not a.startswith(WEGLASSEN)]
    zwischen = ziel / "_archive"
    zwischen.mkdir(parents=True, exist_ok=True)

    def holen_und_pruefen(a: str) -> pathlib.Path:
        url = f"{basis}/{name}/{paketversion}{a}"
        print(f"Lade {a}", file=sys.stderr)
        daten = laden(url)
        soll = text(f"{url}.sha1").split()[0]
        if hashlib.sha1(daten).hexdigest() != soll:
            sys.exit(f"Prüfsumme falsch: {a}")
        datei = zwischen / a
        datei.write_bytes(daten)
        return datei

    # ICU (nur Linux) gehört nach lib/ des Qt-Ordners, die Archive enthalten aber nur die Dateien
    icu = [a for a in archive if a.startswith("icu-")]
    for a in archive:
        if a not in icu:
            entpacken(holen_und_pruefen(a), ziel)
    # neuer Aufbau: Archive enthalten bin/, lib/, ... direkt; alter: <version>/<ordner>/bin, ...
    kandidaten = [ziel] if list(ziel.glob("bin/qmake*")) else sorted(
        p.parent.parent for p in ziel.glob(f"{version}/*/bin/qmake*")
    )
    if not kandidaten:
        sys.exit("qmake nach dem Entpacken nicht gefunden")
    for a in icu:
        entpacken(holen_und_pruefen(a), kandidaten[0] / "lib")
    shutil.rmtree(zwischen)
    print(f"Qt {version} ({arch}): {len(archive)} Archive, Prüfsummen in Ordnung", file=sys.stderr)
    print(kandidaten[0])


if __name__ == "__main__":
    main()
