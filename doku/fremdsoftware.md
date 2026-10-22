# Verwendete Software und Lizenzen

Abolos PDF steht unter der **GNU General Public License, Version 3 oder später** (GPL-3.0-or-later). Rechteinhaber und Maintainer ist Oliver Weber. Den vollständigen Lizenztext zeigt das Kapitel [Lizenz (GNU GPL 3)](hilfe:lizenz).

Abolos PDF baut auf freier Software anderer Projekte auf. Dieses Kapitel nennt alle Bestandteile, wie sie eingebunden sind und unter welcher Lizenz sie stehen. Die vollständigen Lizenztexte stehen im Kapitel [Lizenztexte der Fremdsoftware](hilfe:drittanbieter). Eine maschinenlesbare Liste aller Bestandteile mit Version (Software-Stückliste, CycloneDX) liegt im Quellcode unter `sbom/abolos-pdf.cdx.json`.

Alle Angaben gelten für Version 0.9.0.

## Die großen Bausteine

| Baustein | Version | Wofür | Eingebunden | Lizenz |
|---|---|---|---|---|
| **PDFium** | 151.0.7881 | PDF-Engine: Seiten darstellen, Text, Links, Anmerkungen, Speichern | Bibliothek `libpdfium.so`, liegt dem Programm bei | BSD-3-Clause |
| pdfium-binaries | chromium/7881 | vorgebautes PDFium-Paket (ohne JavaScript und XFA) | Herkunft der Bibliothek | MIT |
| **Qt 6** (Core, Gui, Qml, Quick, Quick Controls, Quick Dialogs, Wayland) | 6.12 | Oberfläche, Fenster, Dialoge | dynamisch geladen: beim eigenen Bau aus der Linux-Distribution, in AppImage, .deb und Windows-Paket mitgeliefert | LGPL-3.0 (alternativ GPL-3.0) |
| Visual-C++-Laufzeit (nur Windows) | 14.x | Laufzeitbibliothek des Compilers | im Windows-Paket mitgeliefert | Microsoft-Lizenz für weitergebbaren Code; für die GPL eine „Systembibliothek“ |
| **cxx-qt** | 0.10 | Verbindung zwischen Rust und Qt | in das Programm übersetzt | MIT oder Apache-2.0 |
| **pdfium-render** | 0.9.4 | Rust-Schnittstelle zu PDFium | in das Programm übersetzt | MIT oder Apache-2.0 |

Webseiten der Projekte (zur Information, Abolos PDF öffnet sie nicht):

- PDFium: pdfium.googlesource.com/pdfium
- pdfium-binaries: github.com/bblanchon/pdfium-binaries
- Qt: qt.io
- cxx-qt: github.com/KDAB/cxx-qt
- pdfium-render: github.com/ajrcarey/pdfium-render

### Mitgeliefertes Qt und die LGPL

Die fertigen Pakete (AppImage, .deb, Windows) enthalten Qt 6.12.0 als einzelne, dynamisch geladene Bibliotheken. Die LGPL-3.0 ist dabei so eingehalten:

- **Hinweis und Lizenztext:** in diesem Kapitel, in [Lizenztexte der Fremdsoftware](hilfe:drittanbieter) und in jedem Paket (`lizenzen/DRITTANBIETER.txt` bzw. `/usr/share/doc/abolos-pdf/`).
- **Austauschbarkeit:** Qt liegt als eigene Dateien neben dem Programm (`Qt6*.dll` bzw. `lib/libQt6*.so*`). Sie lassen sich durch eine selbst gebaute, verträgliche Qt-Version ersetzen.
- **Quellcode von Qt 6.12.0:** download.qt.io/official_releases/qt/6.12/6.12.0/single/qt-everywhere-src-6.12.0.tar.xz. Der Verweis steht auch in jedem Release. Auf Anfrage stellt der Maintainer den Quellcode selbst bereit.

## In PDFium enthaltene Bibliotheken

Diese Bibliotheken sind fest in `libpdfium.so` eingebaut. Ihre Lizenztexte liegen dem PDFium-Paket bei (`third_party/pdfium/licenses/`).

| Bibliothek | Wofür | Lizenz |
|---|---|---|
| FreeType | Schriften darstellen | FreeType License (FTL) |
| libjpeg-turbo | JPEG-Bilder in PDFs | IJG und BSD-3-Clause |
| OpenJPEG | JPEG-2000-Bilder | BSD-2-Clause |
| libpng | PNG-Bilder | libpng License 2 |
| libtiff | TIFF-Bilder | libtiff License |
| zlib | Kompression | zlib License |
| Little CMS | Farbmanagement | MIT |
| Anti-Grain Geometry 2.3 | Vektorgrafik zeichnen | AGG-Lizenz (freizügig) |
| Abseil | C++-Grundbibliothek | Apache-2.0 |
| ICU | Unicode und Textverarbeitung | Unicode License v3 |
| fast_float, simdutf | Zahlen und Zeichenkodierung | MIT |
| LLVM libc | Teile der C-Bibliothek | Apache-2.0 mit LLVM-Ausnahme |

## Rust-Bibliotheken

Das Programm enthält 84 Rust-Bibliotheken (Crates) von crates.io, einige davon nur beim Übersetzen. Direkt verwendet werden:

| Crate | Wofür | Lizenz |
|---|---|---|
| cxx, cxx-qt, cxx-qt-lib | Brücke zu Qt | MIT oder Apache-2.0 |
| pdfium-render | Brücke zu PDFium | MIT oder Apache-2.0 |
| image | Bilder einfügen (PNG, JPEG, GIF, BMP, WebP, TIFF) | MIT oder Apache-2.0 |
| serde, serde_json, toml | Einstellungen und Datenaustausch mit der Oberfläche | MIT oder Apache-2.0 |
| chrono | Datum und Uhrzeit von Anmerkungen | MIT oder Apache-2.0 |
| libc | Systemaufrufe (Netz- und Ausführsperre) | MIT oder Apache-2.0 |

Die übrigen Crates sind deren Abhängigkeiten. Ihre Lizenzen:

| Lizenz | Anzahl |
|---|---|
| MIT oder Apache-2.0 | 55 |
| MIT | 11 |
| Unlicense oder MIT | 4 |
| Zlib (allein oder wahlweise mit MIT/Apache-2.0) | 6 |
| BSD-2-Clause oder BSD-3-Clause, wahlweise mit MIT/Apache-2.0 | 4 |
| ISC | 1 |
| Apache-2.0 | 1 |
| 0BSD oder MIT oder Apache-2.0 | 1 |
| Unicode-3.0 (nur beim Übersetzen verwendet) | 1 |

Keine dieser Lizenzen ist ein „Copyleft“, alle sind freizügig. Welche Crate unter welcher Lizenz steht, steht im Kapitel [Lizenztexte der Fremdsoftware](hilfe:drittanbieter) und in der SBOM.

## Zur Laufzeit vom System geladen

Diese Bestandteile gehören nicht zu Abolos PDF. Sie kommen mit der Linux-Distribution und werden nur benutzt:

| Bestandteil | Wofür | Lizenz |
|---|---|---|
| Breeze-Stil für Qt Quick (KDE) | Aussehen der Bedienelemente | LGPL-2.0 oder später |
| Plasma-Integration, Kirigami (KDE) | Anbindung an den KDE-Desktop | LGPL |
| xdg-desktop-portal, xdg-desktop-portal-kde | Dateidialog des Desktops (eigener Prozess) | LGPL-2.1 oder später / LGPL-2.0 oder später |
| Schriftarten, Mesa, Wayland, D-Bus | Darstellung und Desktop | je nach Paket |

## Verträglichkeit mit der GPLv3

Damit Abolos PDF unter der GPLv3 stehen darf, müssen alle eingebundenen Lizenzen mit ihr verträglich sein. Das ist der Fall:

| Lizenz | verträglich mit GPLv3 | Hinweis |
|---|---|---|
| MIT, BSD-2/3-Clause, ISC, Zlib, Unlicense, 0BSD, libpng, libtiff, IJG, AGG, Unicode | ja | freizügig, nur Hinweise beibehalten |
| Apache-2.0 (auch mit LLVM-Ausnahme) | ja | mit GPLv3 verträglich, mit GPLv2 nicht; ein Grund für die Wahl der Version 3 |
| FreeType License | ja | mit GPLv3 verträglich, mit GPLv2 nicht |
| LGPL-3.0 (Qt) | ja | Qt bleibt dynamisch eingebunden und austauschbar |

`cargo deny check` prüft die Lizenzen aller Rust-Crates bei jedem Prüflauf gegen eine Liste erlaubter Lizenzen (Datei `deny.toml`). Eine neue Abhängigkeit mit unverträglicher Lizenz lässt die Prüfung fehlschlagen.

## Was bei einer Weitergabe zu beachten ist

Für die **Nutzung im eigenen Unternehmen** gibt es keine Auflagen. Auch beliebig viele Installationen und eigene Änderungen gelten nicht als Weitergabe.

Wird Abolos PDF an **Dritte** weitergegeben (Kunden, Partner, andere Firmen, auch rechtlich selbstständige Tochterfirmen), gilt:

1. Den vollständigen Quellcode mitgeben oder schriftlich anbieten (GPLv3, Abschnitt 6).
2. Die Datei `LICENSE` (GPLv3) und `lizenzen/DRITTANBIETER.txt` beilegen. Beide zeigt das Programm auch selbst in der Hilfe an.
3. Bei mitgelieferter PDFium-Bibliothek die Hinweise aus `third_party/pdfium/licenses/` beilegen; sie sind in `DRITTANBIETER.txt` enthalten.
4. Liefert man Qt selbst mit (statt es aus der Distribution zu nehmen), gelten zusätzlich die Pflichten der LGPL-3.0: Quellcode von Qt anbieten und das Austauschen von Qt ermöglichen.

## Werkzeuge, die nur bei der Entwicklung benutzt werden

Diese Werkzeuge werden nicht Teil des Programms und haben deshalb keinen Einfluss auf seine Lizenz:

| Werkzeug | Wofür | Lizenz |
|---|---|---|
| Rust-Compiler, Cargo, Clippy, rustfmt | übersetzen, prüfen, formatieren | MIT oder Apache-2.0 |
| cxx-qt-build | Qt-Brücken beim Übersetzen erzeugen | MIT oder Apache-2.0 |
| cargo-deny | Schwachstellen- und Lizenzprüfung | MIT oder Apache-2.0 |
| cargo-about | Lizenztexte sammeln | MIT oder Apache-2.0 |
| cargo-cyclonedx | Software-Stückliste erzeugen | Apache-2.0 |
| reuse (optional) | SPDX-Kennzeichnung prüfen | GPL-3.0-or-later |
| Python 3 | Hilfsskripte | PSF-Lizenz |
| linuxdeploy, linuxdeploy-plugin-qt | AppImage bauen | MIT |
| windeployqt | Qt für Windows zusammenstellen | Teil von Qt |
| Inno Setup | Windows-Installer bauen | Inno Setup License (frei) |
| aqtinstall | Qt in der CI installieren | MIT |
| stable-diffusion.cpp mit Z-Image Turbo | Programmsymbol erzeugen (einmalig) | MIT; Modell Apache-2.0 |
