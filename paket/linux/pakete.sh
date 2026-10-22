#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Oliver Weber
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Baut aus dem fertig übersetzten Programm die Linux-Pakete:
#   dist/Abolos-PDF-<version>-x86_64.AppImage   läuft auf fast jeder Distribution, eine Datei
#   dist/abolos-pdf_<version>_amd64.deb         Debian, Ubuntu, Mint; installiert nach /opt/abolos-pdf
# Beide bringen ihr eigenes Qt mit (keine Abhängigkeit von der Qt-Version der Distribution).
#
# Aufruf:   QMAKE=/pfad/zu/qt/bin/qmake6 paket/linux/pakete.sh [programm]
#   QMAKE   qmake des Qt, das mitgeliefert wird (z.B. von werkzeuge/qt_holen.py)
#   programm  Standard: target/release/abolos-pdf
#   NO_STRIP=1  nicht verkleinern (nötig auf sehr neuen Distributionen wie Arch: das strip in linuxdeploy
#               kennt deren Bibliotheksformat nicht; in der CI auf älterem Ubuntu nicht nötig)
# Voraussetzungen: PDFium in third_party/pdfium (werkzeuge/pdfium_holen.py linux-x64),
#   lizenzen/DRITTANBIETER.txt und sbom/abolos-pdf.cdx.json aktuell; für .deb: dpkg-deb.
set -euo pipefail
cd "$(dirname "$0")/../.."

programm=${1:-target/release/abolos-pdf}
: "${QMAKE:?QMAKE muss auf das qmake des mitzuliefernden Qt zeigen}"
qt_lib=$("$QMAKE" -query QT_INSTALL_LIBS)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
werkzeuge=build/werkzeuge
appdir=build/AppDir
mkdir -p "$werkzeuge" dist

# Werkzeuge mit fester Prüfsumme: ein geändertes Werkzeug bricht den Bau ab, statt unbemerkt mitzulaufen
holen() {
    local ziel="$werkzeuge/$1"
    if [ ! -f "$ziel" ] || ! echo "$3  $ziel" | sha256sum -c --status; then
        curl -sSL --fail -o "$ziel" "$2"
        if ! echo "$3  $ziel" | sha256sum -c --status; then
            rm -f "$ziel"
            echo "Prüfsumme von $1 stimmt nicht. Das Werkzeug wurde geändert; Version und Prüfsumme in $0 prüfen." >&2
            exit 1
        fi
    fi
    chmod +x "$ziel"
}
holen linuxdeploy-x86_64.AppImage \
    https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage \
    c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d
holen linuxdeploy-plugin-qt-x86_64.AppImage \
    https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/continuous/linuxdeploy-plugin-qt-x86_64.AppImage \
    cfc1055b2b9dbc08412b579f20990b7b41a17b61beaa5847dc9477c96c9e9617

# ---------------------------------------------------------------- AppDir: gemeinsame Grundlage beider Pakete
rm -rf "$appdir"
install -Dm755 "$programm" "$appdir/usr/bin/abolos-pdf"
install -Dm644 third_party/pdfium/lib/libpdfium.so "$appdir/usr/lib/libpdfium.so"
install -Dm644 paket/linux/abolos-pdf.desktop "$appdir/usr/share/applications/abolos-pdf.desktop"
for g in 16 24 32 48 64 128 256 512; do
    install -Dm644 "paket/symbol/abolos-pdf-$g.png" "$appdir/usr/share/icons/hicolor/${g}x${g}/apps/abolos-pdf.png"
done
for d in LICENSE lizenzen/DRITTANBIETER.txt sbom/abolos-pdf.cdx.json; do
    install -Dm644 "$d" "$appdir/usr/share/doc/abolos-pdf/$(basename "$d")"
done

# Das mitgelieferte Qt kann die Bedienelement-Stile des Desktops (z.B. KDE Breeze) nicht laden:
# fremde Stile durch Fusion ersetzen, Pfade des System-Qt nicht übernehmen
mkdir -p "$appdir/apprun-hooks"
cat > "$appdir/apprun-hooks/abolos-pdf-qt.sh" <<'HOOK'
unset QT_PLUGIN_PATH QML2_IMPORT_PATH QML_IMPORT_PATH
case "${QT_QUICK_CONTROLS_STYLE:-}" in
    "" | Basic | Fusion | Imagine | Material | Universal) ;;
    *) export QT_QUICK_CONTROLS_STYLE=Fusion ;;
esac
HOOK

# OpenGL unter Wayland braucht die EGL-Anbindung; linuxdeploy-plugin-qt liefert diesen Ordner nicht mit
qt_plugins=$("$QMAKE" -query QT_INSTALL_PLUGINS)
mkdir -p "$appdir/usr/plugins"
cp -r "$qt_plugins/wayland-graphics-integration-client" "$appdir/usr/plugins/"

export QMAKE
export QML_SOURCES_PATHS="$PWD/qml"
export EXTRA_PLATFORM_PLUGINS="libqwayland.so"
export LD_LIBRARY_PATH="$qt_lib:$PWD/$appdir/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export APPIMAGE_EXTRACT_AND_RUN=1 # Werkzeuge auch ohne FUSE starten (CI)
export LDAI_OUTPUT="dist/Abolos-PDF-$version-x86_64.AppImage"
"$werkzeuge/linuxdeploy-x86_64.AppImage" --appdir "$appdir" --plugin qt --output appimage \
    --desktop-file "$appdir/usr/share/applications/abolos-pdf.desktop" \
    --deploy-deps-only "$appdir/usr/plugins/wayland-graphics-integration-client"
echo "AppImage: $LDAI_OUTPUT"

# ---------------------------------------------------------------- .deb aus demselben AppDir
deb=build/deb
rm -rf "$deb"
mkdir -p "$deb/opt/abolos-pdf" "$deb/usr/bin" "$deb/DEBIAN"
cp -a "$appdir/usr/." "$deb/opt/abolos-pdf/"
mv "$deb/opt/abolos-pdf/share" "$deb/usr/share"
cat > "$deb/usr/bin/abolos-pdf" <<'STARTER'
#!/bin/sh
# Abolos PDF bringt sein eigenes Qt mit (/opt/abolos-pdf); Einstellungen des System-Qt nicht übernehmen
unset QT_PLUGIN_PATH QML2_IMPORT_PATH QML_IMPORT_PATH
case "${QT_QUICK_CONTROLS_STYLE:-}" in
    "" | Basic | Fusion | Imagine | Material | Universal) ;;
    *) export QT_QUICK_CONTROLS_STYLE=Fusion ;;
esac
exec /opt/abolos-pdf/bin/abolos-pdf "$@"
STARTER
chmod 755 "$deb/usr/bin/abolos-pdf"
cat > "$deb/usr/share/doc/abolos-pdf/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: Abolos PDF
Source: https://github.com/

Files: *
Copyright: 2026 Oliver Weber
License: GPL-3.0-or-later
 Der vollständige Lizenztext steht in /usr/share/doc/abolos-pdf/LICENSE
 und in /usr/share/common-licenses/GPL-3.

Files: opt/abolos-pdf/lib/* opt/abolos-pdf/plugins/* opt/abolos-pdf/qml/*
Copyright: Qt, PDFium und weitere, siehe DRITTANBIETER.txt
License: siehe /usr/share/doc/abolos-pdf/DRITTANBIETER.txt
EOF

# Abhängigkeiten aus den Bibliotheken ermitteln (Debian/Ubuntu); sonst eine Grundliste
if command -v dpkg-shlibdeps >/dev/null; then
    mkdir -p "$deb/debian" && touch "$deb/debian/control"
    abh=$(cd "$deb" && dpkg-shlibdeps -O --ignore-missing-info -l"opt/abolos-pdf/lib" \
        $(find opt/abolos-pdf -type f \( -name '*.so*' -o -path '*/bin/abolos-pdf' \)) 2>/dev/null |
        sed -n 's/^shlibs:Depends=//p')
    rm -rf "$deb/debian"
else
    abh="libc6, libstdc++6, libgl1, libegl1, libfontconfig1, libfreetype6, libxkbcommon0, libdbus-1-3, libglib2.0-0"
fi
groesse=$(du -sk "$deb" | cut -f1)
cat > "$deb/DEBIAN/control" <<EOF
Package: abolos-pdf
Version: $version
Section: text
Priority: optional
Architecture: amd64
Maintainer: ${ABOLOS_MAINTAINER:-Oliver Weber}
Installed-Size: $groesse
Depends: $abh
Homepage: ${ABOLOS_HOMEPAGE:-https://github.com/}
Description: Offline-PDF-Reader: lesen, suchen, markieren, kommentieren – ohne Abo
 Abolos PDF zeigt, durchsucht und kommentiert PDF-Dateien, auch sehr große.
 Kein Netzwerkzugriff, keine Telemetrie; vollständig per Tastatur bedienbar.
 Suche über ganze Ordner, Anmerkungen als echte PDF-Anmerkungen.
EOF
dpkg-deb --root-owner-group --build "$deb" "dist/abolos-pdf_${version}_amd64.deb" >/dev/null
echo "Debian-Paket: dist/abolos-pdf_${version}_amd64.deb"
