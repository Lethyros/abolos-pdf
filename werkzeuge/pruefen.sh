#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Oliver Weber
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Prüfablauf vor jeder Version (siehe doku/standards.md, „Der Prüfablauf vor jeder Version“).
# Bricht beim ersten Fehler ab. Aufruf: werkzeuge/pruefen.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

nr=0
schritt() {
    nr=$((nr + 1))
    local titel=$1
    shift
    printf '\n\033[1m[%d] %s\033[0m\n' "$nr" "$titel"
    if ! "$@"; then
        printf '\n\033[1;31mFEHLER in Schritt %d: %s\033[0m\n' "$nr" "$titel"
        exit 1
    fi
}

fehlt() {
    printf 'Werkzeug fehlt: %s\n  Installieren: %s\n' "$1" "$2"
    return 1
}

pruefe_werkzeuge() {
    command -v cargo-deny >/dev/null || fehlt cargo-deny "cargo install --locked cargo-deny"
    command -v cargo-about >/dev/null || fehlt cargo-about "cargo install --locked cargo-about --features cli"
    command -v cargo-cyclonedx >/dev/null || fehlt cargo-cyclonedx "cargo install --locked cargo-cyclonedx"
}

schritt "Werkzeuge vorhanden" pruefe_werkzeuge
schritt "Formatierung (rustfmt)" cargo fmt --check
schritt "Statische Analyse (Clippy, keine Warnungen)" cargo clippy --all-targets -- -D warnings
schritt "Tests" cargo test
schritt "Abhängigkeiten: Schwachstellen, Lizenzen, Herkunft (cargo-deny)" cargo deny check
schritt "Lizenztexte der Fremdsoftware" werkzeuge/lizenzen.sh
schritt "Software-Stückliste (CycloneDX)" python3 werkzeuge/sbom.py
if command -v reuse >/dev/null; then
    schritt "SPDX-Kennzeichnung (REUSE)" reuse lint
else
    printf '\n(REUSE-Prüfung übersprungen: „reuse“ ist nicht installiert, Arch: sudo pacman -S reuse)\n'
fi

printf '\n\033[1;32mAlle Prüfungen bestanden.\033[0m Version %s\n' "$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
