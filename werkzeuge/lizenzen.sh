#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Oliver Weber
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Erzeugt lizenzen/DRITTANBIETER.txt: die Lizenztexte aller Fremdsoftware, die in Abolos PDF steckt
# (PDFium mit seinen Bibliotheken, Qt, alle Rust-Bibliotheken). Die Datei wird ins Programm
# eingebettet (Hilfe → Lizenztexte der Fremdsoftware) und muss bei jeder Weitergabe beiliegen.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

ziel=lizenzen/DRITTANBIETER.txt
crates=$(mktemp)
trap 'rm -f "$crates"' EXIT
cargo about generate --offline --fail about.hbs > "$crates"

# als UTF-8 ausgeben (einige Lizenzdateien sind Latin-1, die Hilfe braucht UTF-8)
utf8() { if iconv -f utf-8 -t utf-8 "$1" >/dev/null 2>&1; then cat "$1"; else iconv -f latin1 -t utf-8 "$1"; fi; }

trenner() { printf '\n%s\n%s\n%s\n\n' "================================================================================" "$1" "================================================================================"; }

{
    echo "Abolos PDF $(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1) – Lizenztexte der verwendeten Fremdsoftware"
    echo
    echo "Abolos PDF selbst steht unter der GNU General Public License, Version 3 oder später"
    echo "(Datei LICENSE). Diese Datei enthält die Lizenztexte der Fremdsoftware, die im Programm"
    echo "enthalten ist oder von ihm geladen wird. Erzeugt mit werkzeuge/lizenzen.sh."

    trenner "1. PDFium (Binärpaket von bblanchon/pdfium-binaries) und enthaltene Bibliotheken"
    echo "Paket-Lizenz (third_party/pdfium/LICENSE):"
    echo
    utf8 third_party/pdfium/LICENSE
    for f in third_party/pdfium/licenses/*; do
        trenner "PDFium: $(basename "$f")"
        utf8 "$f"
    done

    trenner "2. Qt 6 – GNU Lesser General Public License, Version 3"
    echo "Qt wird als Systembibliothek dynamisch geladen und kann durch jede kompatible Qt-6-Version"
    echo "ersetzt werden. Quellcode: https://download.qt.io/ bzw. das Quellpaket der Linux-Distribution."
    echo
    cat lizenzen/LGPL-3.0-only.txt

    trenner "3. Rust-Bibliotheken (zusammengestellt mit cargo-about)"
    cat "$crates"
} > "$ziel"
echo "$ziel: $(wc -c < "$ziel") Bytes"
