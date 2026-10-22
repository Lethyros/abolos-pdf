# Für Entwickler

Dieses Kapitel beschreibt Aufbau, Bauen, Prüfen und Freigeben von Abolos PDF. Maintainer: Oliver Weber.

## Aufbau

Abolos PDF ist in **Rust** geschrieben, die Oberfläche in **QML** (Qt Quick). Die Verbindung zu Qt stellt **cxx-qt** her, PDFs verarbeitet **PDFium** über **pdfium-render**.

| Datei | Aufgabe |
|---|---|
| `src/main.rs` | Start: Kommandozeile, Schutzmaßnahmen, Qt starten |
| `src/schutz.rs` | Netz- und Ausführsperre je Betriebssystem (Linux: Namensraum, Landlock; Windows: Prozessrichtlinie) |
| `src/engine.rs` | PDF-Thread: besitzt PDFium allein; Öffnen, Rendern, Links, Suche, Ordnersuche, Bild- und Text-Zwischenspeicher |
| `src/suche.rs` | Textsuche: Aufbereitung des Seitentexts, Vergleich, Zusammenhang, Optionen |
| `src/bearbeiten.rs` | Anmerkungen anlegen, auflisten, ändern, löschen; Gliederung; Speichern |
| `src/dokument.rs` | QML-Singleton `Dokument`: alles, was die Oberfläche vom Dokument braucht |
| `src/seite.rs` | QML-Element `Seite`: zeichnet eine Seite mit Suchtreffern |
| `src/zustand.rs` | gemeinsamer Zustand: Seitenanordnung, Drehung, Suchtreffer, Links |
| `src/tasten.rs` | QML-Singleton `Tasten`: frei belegbare Tasten |
| `src/hilfe.rs` | QML-Singleton `Hilfe`: eingebettete Dokumentation, „Über“ |
| `src/lesezeichen.rs`, `src/pfade.rs` | eigene Lesezeichen; Speicherorte nach XDG |
| `src/pruefung.rs` | Zusammenspiel-Tests über die echte PDF-Engine |
| `qml/*.qml` | Oberfläche: Hauptfenster, Seitenleiste, Tastendialog, Hilfefenster |
| `doku/*.md` | diese Dokumentation (wird ins Programm eingebettet) |

### Threads

- **Oberfläche** (Qt-Hauptthread): QML und alle Qt-Objekte.
- **PDF-Thread** („pdfium“): besitzt PDFium allein, denn PDFium ist nicht threadsicher. Er arbeitet Aufträge aus einer Warteschlange ab:
  - Öffnen und Bearbeiten der Reihe nach.
  - Rendern zuerst das Neueste (das, was gerade auf dem Bildschirm steht).
  - Suche und Ordnersuche laufen stückweise zwischendurch weiter, damit Scrollen flüssig bleibt.
- **Dateisuche** der Ordnersuche: ein kurzlebiger Thread, der den Ordnerbaum nach PDFs durchsucht.

Ergebnisse gehen über `qt_thread().queue(...)` zurück in den Hauptthread. Gemeinsame Daten liegen in `zustand.rs` hinter `Mutex` bzw. `RwLock`.

## Voraussetzungen

- Rust mit Edition 2024 (getestet mit Rust 1.99.1)
- Qt 6.12 mit Entwicklungsdateien: Qt Base, Qt Declarative. `qmake6` (oder `qmake`) muss im Pfad liegen oder in `QMAKE` stehen.
- C++-Compiler: unter Linux gcc oder clang, unter Windows die Visual Studio Build Tools (MSVC 2022)
- Python 3 (Hilfsskripte)
- für den Prüfablauf: `cargo install --locked cargo-deny cargo-cyclonedx` und `cargo install --locked cargo-about --features cli`; optional `reuse` (Arch: `sudo pacman -S reuse`)

PDFium ist nicht im Repository. `werkzeuge/pdfium_holen.py` lädt das vorgebaute Paket „chromium/7881“ von bblanchon/pdfium-binaries (ohne V8 und XFA) und **akzeptiert nur die fest eingetragene SHA256-Prüfsumme**:

```
python3 werkzeuge/pdfium_holen.py linux-x64     # bzw. win-x64
```

## Bauen und starten

```
cargo build --release
target/release/abolos-pdf datei.pdf            # Windows: target\release\abolos-pdf.exe
```

Das Programm sucht `libpdfium.so` bzw. `pdfium.dll` in dieser Reihenfolge:

1. `$ABOLOS_PDFIUM`
2. neben dem Programm, in `lib/` daneben und in `../lib` (Aufbau der Linux-Pakete)
3. im Projektordner unter `third_party/pdfium/lib` bzw. `bin`
4. in den Systembibliotheken

## Pakete

| Paket | Skript | Inhalt |
|---|---|---|
| AppImage, .deb | `paket/linux/pakete.sh` | Programm, PDFium, mitgeliefertes Qt (aus aqtinstall), Desktop-Datei, Symbole, Lizenzen |
| Windows-Ordner | `paket/windows/vorbereiten.ps1` | Programm, PDFium, Qt (windeployqt), Visual-C++-Laufzeit, Lizenzen |
| Windows-Installer | `paket/windows/abolos-pdf.iss` | Inno Setup: pro Benutzer ohne Adminrechte, optional für alle mit Firewall-Regel, „Öffnen mit“, Deinstallation |

- **Linux-Pakete:** Sie bringen ihr eigenes Qt mit und hängen so nicht von der Qt-Version der Distribution ab. Sie entstehen auf Ubuntu 22.04 und laufen deshalb auf Systemen ab glibc 2.35 (Ubuntu 22.04, Debian 12, Fedora 36 und neuer).
- **Werkzeuge:** Die AppImage-Werkzeuge (linuxdeploy und dessen Qt-Plugin) sind mit Version und SHA256 festgelegt.
- **Lokal bauen:** `QMAKE=/pfad/zu/qt/bin/qmake6 paket/linux/pakete.sh` baut die Linux-Pakete. Auf sehr neuen Distributionen wie Arch braucht es zusätzlich `NO_STRIP=1`; das `.deb` entsteht nur, wo `dpkg-deb` vorhanden ist.
- **Windows-Programm:** Die `.exe` enthält Symbol, Versionsangaben und ein Manifest (`build.rs`). Das Manifest legt fest:
  - Start ohne Adminrechte (`asInvoker`)
  - Windows 10/11
  - hohe Auflösung
  - lange Pfade
  - UTF-8 für Umlaute in Dateinamen

### Programmsymbol

Das Symbol ist mit **Z-Image Turbo** entstanden (lokal mit stable-diffusion.cpp; Modell unter Apache-2.0). `werkzeuge/symbol.py vorlage.png` stellt die Kachel frei und erzeugt `paket/symbol/abolos-pdf-<16…512>.png` und `abolos-pdf.ico`.

## Prüfen

```
werkzeuge/pruefen.sh
```

Das Skript führt alle Prüfschritte aus und bricht beim ersten Fehler ab: Formatierung, Clippy ohne Warnungen, Tests, Abhängigkeiten (Schwachstellen, Lizenzen, Herkunft), Lizenztexte, Software-Stückliste und, falls installiert, REUSE. Welche Norm welcher Schritt bedient, steht in [Qualität und Firmenstandards](hilfe:standards).

### Tests

- `cargo test` führt Einzel- und Zusammenspieltests aus. Diese Tests brauchen keine Anzeige, wohl aber `libpdfium.so`.
- Die Testdatei `tests/daten/test.pdf` erzeugt `tests/daten/erzeugen.py`. Der Inhalt ist dort beschrieben.
- Jeder Test arbeitet auf einer Kopie in einem eigenen Temp-Ordner.

### Oberfläche automatisch bedienen

Für Bildschirmfotos und Abläufe in der Oberfläche liest das Programm die Variable `ABOLOS_SKRIPT`. Sie enthält Befehle, getrennt durch Semikolon:

```
ABOLOS_SKRIPT="warte:1000;suche:Schutz;aktion:naechster_treffer;screenshot:/tmp/a.png;beenden" abolos-pdf test.pdf
```

Befehle:

| Gruppe | Befehle |
|---|---|
| Ablauf | `warte:ms`, `beenden` (ohne Rückfrage) |
| Bedienung | `aktion:<id>` (jede Aktion der Tastenbelegung), `seite:N`, `zoom:f`, `modus:N` |
| Suche | `suche:text`, `tippen:text`, `treffer:i`, `trefferliste` |
| Ordnersuche | `ordnersuche:pfad\|text[\|w]` (w = nur ganze Wörter), `ordnerstatus`, `ordneroeffnen:i`, `ordnerdialog` |
| Links | `links:S`, `link:S,x,y` |
| Anmerkungen | `anlegen:<json>`, `auswahl:S,x1,y1,x2,y2`, `anmerkungen`, `rueckgaengig` |
| Dokument | `gliederung`, `speichern:pfad`, `oeffnen:pfad` |
| Oberfläche | `leiste:N`, `hilfe:id`, `markierung`, `status`, `screenshot:pfad` |

Das Ergebnis erscheint im Protokoll (mit `QT_FORCE_STDERR_LOGGING=1` im Terminal).

## Regeln für den Code

- Namen und Kommentare auf Deutsch, wie im bestehenden Code. Kommentare erklären das **Warum**.
- `cargo fmt` (Einstellungen in `rustfmt.toml`) und Clippy ohne Warnungen.
- Jede `unsafe`-Stelle bekommt einen Kommentar `// SAFETY: …`, der begründet, warum sie korrekt ist. Clippy erzwingt das.
- **Keine Netzwerkfunktionen** und **keine Programmaufrufe**. Beides ist zur Laufzeit ohnehin gesperrt; der Code soll es gar nicht erst versuchen.
- Neue Abhängigkeiten nur von crates.io und nur mit Lizenzen aus `deny.toml`. Danach `werkzeuge/lizenzen.sh` und `werkzeuge/sbom.py` ausführen.
- Jede neue Datei bekommt den SPDX-Kopf (siehe bestehende Dateien) oder einen Eintrag in `REUSE.toml`.
- Jede Änderung, die Anwender bemerken, kommt in `CHANGELOG.md`.

## Neue Version freigeben

1. Versionsnummer in `Cargo.toml` erhöhen (Schema `Haupt.Neben.Fehlerbehebung`).
2. `CHANGELOG.md` ergänzen: Überschrift `## X.Y.Z – Datum`, darunter Neues, Geändert, Behoben. Dieser Abschnitt wird zum Text des Releases.
3. `werkzeuge/pruefen.sh` muss fehlerfrei durchlaufen.
4. Abnahmetest (unten) durchführen und das Ergebnis festhalten.
5. Übernehmen und markieren: `git commit`, `git tag vX.Y.Z`, `git push --follow-tags`.
6. GitHub baut dann automatisch (`.github/workflows/release.yml`):
   - Linux: AppImage, .deb
   - Windows: Installer und ZIP ohne Installation
   - dazu `SHA256SUMS.txt`, eine Software-Stückliste je System und die Lizenztexte
   - Daraus entsteht das Release. Passt der Tag nicht zur Version in `Cargo.toml`, bricht der Bau ab.

Bei jedem Push und Pull Request prüft `.github/workflows/pruefen.yml` unter Linux und Windows: Formatierung, Clippy, Tests, cargo-deny, REUSE.

Alle GitHub-Aktionen sind auf feste Commits festgelegt. Dependabot (`.github/dependabot.yml`) schlägt monatlich Updates für Aktionen und Rust-Bibliotheken vor.

### Windows-Signatur (SignPath)

Ohne Code-Signatur warnt Windows SmartScreen beim ersten Start heruntergeladener Installer. Die SignPath Foundation signiert Open-Source-Projekte kostenlos. Einrichtung:

1. Auf signpath.org das Projekt anmelden (öffentliches Repository, OSI-Lizenz GPL-3.0). Nach der Zusage legt SignPath Organisation und Projekt an.
2. Dort zwei Artefakt-Konfigurationen anlegen, mit dem Inhalt aus `.signpath/artifact-configuration/`:
   - `programm`: signiert `abolos-pdf.exe`
   - `installer`: signiert den Setup
3. Im GitHub-Repository unter Settings → Secrets and variables → Actions eintragen:
   - Secret `SIGNPATH_API_TOKEN`
   - Variablen `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG` (z.B. `abolos-pdf`) und `SIGNPATH_SIGNING_POLICY_SLUG` (z.B. `release-signing`)
4. Ab dem nächsten Release werden Programm und Installer automatisch signiert. Fremde Dateien (Qt, PDFium) signiert SignPath nach seinen Regeln nicht, das ist auch nicht nötig.

Auch signierte Installer brauchen anfangs etwas Zeit, bis Windows SmartScreen ihnen „vertraut“ (Reputation). Die Warnung wird mit der Zeit seltener und verschwindet dann.

### Abnahmetest (von Hand)

Auf einer **Kopie** eines echten Dokuments prüfen und die Ergebnisse mit Datum und Version festhalten:

- [ ] Öffnen über den Dialog und über die Kommandozeile
- [ ] Scrollen, Zoomen, Doppelseiten, Buchansicht, Drehen (alle und eine Seite)
- [ ] Link anklicken: Sprung, rote Markierung, Zurück mit Alt+←
- [ ] Suche mit Trefferliste; Optionen „Aa“ und „Wort“
- [ ] Ordnersuche, Treffer anklicken öffnet die richtige Stelle
- [ ] Jedes Werkzeug einmal mit der Maus: Hervorheben, Unterstreichen, Durchstreichen, Stift, Rechteck, Ellipse, Linie, Pfeil, Notiz, Textfeld, Bild, Löschen
- [ ] Rückgängig, Kommentar per Doppelklick ändern
- [ ] Speichern, Programm schließen, Datei in einem anderen Programm (z.B. Okular) öffnen: Anmerkungen sichtbar
- [ ] Lesezeichen setzen, umbenennen, entfernen
- [ ] Eine Taste umbelegen; sie wirkt sofort
- [ ] Hilfe (F1): Kapitel öffnen; „Über“ zeigt Netz- und Ausführsperre „aktiv“ (Windows: Netzsperre über Firewall-Regel)
- [ ] Windows: Installer ohne Adminrechte, Startmenü, „Öffnen mit“ bei einer PDF, Deinstallation über Einstellungen → Apps
- [ ] Linux: AppImage und .deb (auf einem Ubuntu oder Debian) starten, Datei öffnen

## PDFium aktualisieren

1. In `werkzeuge/pdfium_holen.py` die neue Version (`VERSION`) und die SHA256-Prüfsummen für `linux-x64` und `win-x64` eintragen. Die Prüfsummen stehen auf der Release-Seite von bblanchon/pdfium-binaries („digest“).
2. `python3 werkzeuge/pdfium_holen.py linux-x64` ausführen.
3. In `Cargo.toml` das passende `pdfium_XXXX`-Merkmal von pdfium-render wählen, ggf. pdfium-render aktualisieren.
4. Version in `sbom/fremdkomponenten.json` anpassen.
5. `werkzeuge/pruefen.sh` und den Abnahmetest durchführen, auch unter Windows (GitHub-Prüflauf).

## Eigenheiten von pdfium-render 0.9.4

Diese Fallen sind im Code umgangen. Bei Updates sollte man sie erneut prüfen:

- `set_stroke_color` auf Zeichnungen und Stempeln nach dem Hinzufügen von Objekten bringt PDFium zum Absturz. Deshalb wird die Farbe vorher gesetzt.
- `stroke_color()` und `fill_color()` auf Anmerkungen mit eigener Darstellung stürzen ab. Deshalb liest `sichere_farbe()` die Farbe aus den Objekten.
- `PdfQuadPoints::from_rect` liefert die falsche Punktreihenfolge. Deshalb gibt es eine eigene Funktion `quad()`.
- Nur Zeichnungen (Ink) und Stempel können Objekte enthalten. Rechteck, Ellipse, Linie und Pfeil sind deshalb Zeichnungen, Textfeld und Bild sind Stempel. Was sie sind, steht im Betreff (/Subj).
- `PdfPageText::all()` liefert Text, dessen Positionen nicht zu den Zeichenindizes passen. Die Suche liest deshalb Zeichen für Zeichen.
- Der Betreff (/Subj) einer Anmerkung lässt sich mit pdfium-render nicht setzen. Der PDF-Thread hat dafür eine zweite Anbindung an PDFium (`engine::mit_roh`).
