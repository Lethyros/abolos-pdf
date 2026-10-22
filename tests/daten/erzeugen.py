#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Oliver Weber
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt tests/daten/test.pdf: zwei A4-Seiten mit bekanntem Text für die automatischen Tests.

Seite 1, Zeilen ab 72 pt vom linken Rand, Grundlinien bei 770, 750, 730, 710 pt (von unten):
  "Schutz und Schutzleiter im Betrieb"
  "Arbeiten unter Spannung sind gefährlich."
  "Prüfung der Erdungs- und Kurzschließvorrichtung"
  "Fehlerschutz durch Abschaltung"
Seite 2: "Schutz gegen elektrischen Schlag" und "Niederspannungsanlagen"
"""
import pathlib

seiten = [
    ["Schutz und Schutzleiter im Betrieb", "Arbeiten unter Spannung sind gefährlich.",
     "Prüfung der Erdungs- und Kurzschließvorrichtung", "Fehlerschutz durch Abschaltung"],
    ["Schutz gegen elektrischen Schlag", "Niederspannungsanlagen"],
]


def inhalt(zeilen):
    """Inhaltsstrom einer Seite; Text in cp1252 (= WinAnsiEncoding der Schrift)"""
    aus = b"BT /F1 14 Tf\n"
    for nr, zeile in enumerate(zeilen):
        text = zeile.encode("cp1252").replace(b"\\", b"\\\\").replace(b"(", b"\\(").replace(b")", b"\\)")
        aus += f"1 0 0 1 72 {770 - 20 * nr} Tm (".encode() + text + b") Tj\n"
    return aus + b"ET\n"


objekte = []
n_seiten = len(seiten)
# 1 Katalog, 2 Seitenbaum, 3 Schrift, dann je Seite: Seite + Inhalt
kinder = " ".join(f"{4 + 2 * i} 0 R" for i in range(n_seiten))
objekte.append(b"<< /Type /Catalog /Pages 2 0 R >>")
objekte.append(f"<< /Type /Pages /Kids [{kinder}] /Count {n_seiten} >>".encode())
objekte.append(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>")
for i, zeilen in enumerate(seiten):
    strom = inhalt(zeilen)
    objekte.append(f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R >> >> /Contents {5 + 2 * i} 0 R >>".encode())
    objekte.append(f"<< /Length {len(strom)} >>\nstream\n".encode() + strom + b"endstream")

pdf = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"
versatz = []
for nr, obj in enumerate(objekte, start=1):
    versatz.append(len(pdf))
    pdf += f"{nr} 0 obj\n".encode() + obj + b"\nendobj\n"
xref = len(pdf)
pdf += f"xref\n0 {len(objekte) + 1}\n0000000000 65535 f \n".encode()
for v in versatz:
    pdf += f"{v:010d} 00000 n \n".encode()
pdf += f"trailer\n<< /Size {len(objekte) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
ziel = pathlib.Path(__file__).with_name("test.pdf")
ziel.write_bytes(pdf)
print(f"{ziel}: {len(pdf)} Bytes")
