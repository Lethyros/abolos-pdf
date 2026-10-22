#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Oliver Weber
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt die Software-Stückliste (SBOM, CycloneDX 1.5) sbom/abolos-pdf.cdx.json.

Aufruf: werkzeuge/sbom.py [ziel-triple]   (Standard: die eigene Plattform)

Die Rust-Bibliotheken liefert cargo-cyclonedx aus Cargo.lock; PDFium mit seinen Bibliotheken und
Qt stehen in sbom/fremdkomponenten.json (von Hand gepflegt, Versionen bei Updates anpassen).
"""
import json
import os
import pathlib
import subprocess
import sys

wurzel = pathlib.Path(__file__).resolve().parent.parent
umgebung = dict(os.environ, PATH=str(pathlib.Path.home() / ".cargo/bin") + os.pathsep + os.environ.get("PATH", ""))
# Zielplattform: Argument, sonst die eigene (Windows hat z.B. windows-sys statt libc)
if len(sys.argv) > 1:
    ziel_triple = sys.argv[1]
else:
    rustc = subprocess.run(["rustc", "-vV"], capture_output=True, text=True, env=umgebung, check=True).stdout
    ziel_triple = next(z.split(": ", 1)[1] for z in rustc.splitlines() if z.startswith("host: "))
roh = wurzel / "abolos-pdf-roh.cdx.json"
subprocess.run(
    ["cargo", "cyclonedx", "--format", "json", "--spec-version", "1.5", "--no-build-deps",
     "--target", ziel_triple, "--override-filename", "abolos-pdf-roh.cdx"],
    cwd=wurzel, env=umgebung, check=True, stdout=subprocess.DEVNULL)
sbom = json.loads(roh.read_text(encoding="utf-8"))
roh.unlink()

fremd = json.loads((wurzel / "sbom/fremdkomponenten.json").read_text(encoding="utf-8"))
sbom.setdefault("components", []).extend(fremd["components"])
haupt = sbom["metadata"]["component"]
haupt["name"] = "Abolos PDF"
haupt["authors"] = [{"name": "Oliver Weber"}]
haupt["supplier"] = {"name": "Oliver Weber"}
haupt["licenses"] = [{"expression": "GPL-3.0-or-later"}]
abhaengig = next((d for d in sbom.get("dependencies", []) if d.get("ref") == haupt.get("bom-ref")), None)
if abhaengig is not None:
    abhaengig.setdefault("dependsOn", []).extend(c["bom-ref"] for c in fremd["components"] if c.get("direkt", True))
for c in fremd["components"]:
    c.pop("direkt", None)

ziel = wurzel / "sbom/abolos-pdf.cdx.json"
text = json.dumps(sbom, indent=2, ensure_ascii=False) + "\n"
# cargo-cyclonedx nennt das eigene Paket nach seinem Ordner; der lokale Pfad gehört nicht in die SBOM
text = text.replace(f"path+file://{wurzel.as_posix()}", "abolos-pdf").replace(wurzel.as_posix(), "abolos-pdf")
ziel.write_text(text, encoding="utf-8")
print(f"{ziel.relative_to(wurzel)}: {len(sbom['components'])} Komponenten ({ziel_triple})", file=sys.stderr)
