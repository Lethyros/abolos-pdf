# Abolos PDF

Offline-PDF-Reader für **Linux und Windows**: PDFs lesen, durchsuchen, markieren und kommentieren, auch sehr große. Ohne Abo, ohne Netzwerkzugriff, vollständig per Tastatur bedienbar.

- **Lesen:** durchgehend scrollen, Zoom, Doppelseiten und Buchansicht, Drehen (alle oder einzelne Seiten), Inhaltsverzeichnis, eigene Lesezeichen, Links mit Rücksprung
- **Suchen:** Trefferliste mit Seite und Zusammenhang, ganze Wörter, Groß-/Kleinschreibung, Silbentrennung am Zeilenende; Suche über alle PDFs eines Ordners
- **Bearbeiten:** Hervorheben, Unterstreichen, Durchstreichen, Stift, Rechteck, Ellipse, Linie, Pfeil, Notiz, Textfeld, Bild; echte PDF-Anmerkungen, sichtbar in anderen Programmen
- **Sicher:** Netzsperre und Ausführsperre beim Start, PDF-Engine ohne JavaScript, in Rust geschrieben

## Herunterladen und installieren

Die fertigen Pakete stehen unter **Releases** (rechts auf der GitHub-Seite).

| System | Datei | Installation |
|---|---|---|
| Windows 10/11 | `Abolos-PDF-…-Setup-x64.exe` | doppelklicken; braucht **keine Adminrechte** |
| Windows ohne Installation | `Abolos-PDF-…-Windows-x64-ohne-Installation.zip` | entpacken, `abolos-pdf.exe` starten |
| Linux (fast alle) | `Abolos-PDF-…-x86_64.AppImage` | ausführbar machen (`chmod +x`), starten |
| Debian, Ubuntu, Mint | `abolos-pdf_…_amd64.deb` | `sudo apt install ./abolos-pdf_…_amd64.deb` |

Wer für alle Benutzer installiert (Windows als Administrator), kann im Installer zusätzlich eine Firewall-Regel anlegen lassen, die dem Programm jeden Netzwerkzugriff verbietet. Für die IT-Verteilung geht der Installer auch stumm: `…-Setup-x64.exe /VERYSILENT /SUPPRESSMSGBOXES` (für alle Benutzer zusätzlich `/ALLUSERS`).

Prüfsummen stehen in `SHA256SUMS.txt`, alle Bestandteile in den Software-Stücklisten `*.cdx.json`.

> **Hinweis zu Windows SmartScreen:** Solange die Installer noch nicht code-signiert sind, zeigt Windows beim ersten Start „Der Computer wurde durch Windows geschützt“. Unter „Weitere Informationen“ → „Trotzdem ausführen“ geht es weiter. Die Signierung ist vorbereitet (SignPath) und wird eingeschaltet, sobald sie bewilligt ist.

Autor und Maintainer: **Oliver Weber**. Lizenz: **GNU GPL, Version 3 oder später** (`LICENSE`).

## Bauen und starten

Voraussetzungen: Rust (Edition 2024), Qt 6 mit Entwicklungsdateien (`qmake6`), C++-Compiler (unter Windows: Visual Studio Build Tools). PDFium lädt ein Skript mit geprüfter Prüfsumme:

```
python3 werkzeuge/pdfium_holen.py linux-x64      # Windows: win-x64
cargo build --release
target/release/abolos-pdf datei.pdf
```

Die Pakete baut GitHub bei jedem Versions-Tag automatisch (`.github/workflows/release.yml`); wie das abläuft und wie man sie selbst baut, steht in [doku/entwicklung.md](doku/entwicklung.md).

## Dokumentation

Die Dokumentation ist ins Programm eingebettet (**F1**) und liegt außerdem hier:

| Datei | Inhalt |
|---|---|
| [doku/handbuch.md](doku/handbuch.md) | Benutzerhandbuch |
| [doku/sicherheit.md](doku/sicherheit.md) | Sicherheit und Datenschutz |
| [doku/standards.md](doku/standards.md) | Qualität und Firmenstandards: welche Normen, wie, mit welchem Nachweis |
| [doku/fremdsoftware.md](doku/fremdsoftware.md) | verwendete Software und Lizenzen |
| [doku/entwicklung.md](doku/entwicklung.md) | Aufbau, Prüfen, Freigeben, Abnahmetest |
| [CHANGELOG.md](CHANGELOG.md) | Änderungen je Version |
| [SECURITY.md](SECURITY.md) | Sicherheitslücken melden |

## Prüfen

```
werkzeuge/pruefen.sh
```

Das Skript führt Formatierung, Clippy, Tests, die Abhängigkeitsprüfung (Schwachstellen, Lizenzen), Lizenztexte, Software-Stückliste und REUSE-Prüfung aus.

## Lizenz

Copyright © 2026 Oliver Weber

Dieses Programm ist freie Software: Sie können es unter den Bedingungen der GNU General Public License, wie von der Free Software Foundation veröffentlicht, weitergeben und/oder verändern, entweder gemäß Version 3 der Lizenz oder (nach Ihrer Wahl) jeder späteren Version. Es wird OHNE JEDE GEWÄHRLEISTUNG bereitgestellt; Einzelheiten in `LICENSE`.

Die Lizenzen der verwendeten Fremdsoftware stehen in [doku/fremdsoftware.md](doku/fremdsoftware.md), die vollständigen Texte in `lizenzen/DRITTANBIETER.txt`.
