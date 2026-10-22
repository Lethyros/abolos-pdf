# Änderungen

Alle Änderungen, die Anwender bemerken, je Version. Versionsnummern: Haupt.Neben.Fehlerbehebung.

## 0.9.0 – 2026-10-22

Erste Version unter dem Namen **Abolos PDF** (vorher Arbeitstitel „Seitenblick“). Vor dem Einsatz steht noch der Abnahmetest der Mausbedienung aus (siehe „Für Entwickler“).

### Neu

- **Name, Lizenz, Rechtliches:** Abolos PDF, GNU GPL Version 3 oder später, Autor und Maintainer Oliver Weber. Fenster „Über“ mit Copyright, Gewährleistungsausschluss und Versionen. Kommandozeile `--version` und `--help`.
- **Hilfe (F1)** mit eingebetteter Dokumentation:
  - Benutzerhandbuch und Tastenbelegung
  - Sicherheit und Datenschutz
  - Qualität und Firmenstandards
  - verwendete Software und Lizenzen
  - Entwickler-Dokumentation und Änderungen
  - Lizenztexte
- **Ausführsperre:** Nach dem Start kann das Programm keine anderen Programme mehr starten (Landlock). Ergänzt die bestehende Netzsperre.
- **Suche:** nur ganze Wörter („Wort“, Alt+W), Groß-/Kleinschreibung jetzt auch per Taste (Alt+C).
- **Suche:** Trefferliste mit Seite und umgebendem Text in der Seitenleiste.
- **Suche** über alle PDFs eines Ordners (Strg+Shift+F) mit Zwischenspeicher für wiederholte Suchen.
- Am Zeilenende getrennte Wörter werden gefunden („Nieder-⏎spannung“).
- **Werkzeuge Linie (Alt+L) und Pfeil (Alt+P).** Mit Shift waagrecht, senkrecht oder 45°; Rechteck und Ellipse mit Shift quadratisch bzw. rund.
- Anmerkungen tragen einen Betreff (Linie, Pfeil, Rechteck, Ellipse, Freihand, Textfeld, Bild), auch in anderen Programmen sichtbar.
- Dateidialoge des Desktops (KDE/GNOME) statt Qts Ersatzdialog; Ordner und PDFs sind jetzt klar zu unterscheiden.
- Seitenleiste in der Breite verstellbar.
- **Windows-Version** mit Installer (ohne Adminrechte; für alle Benutzer optional mit Firewall-Regel als Netzsperre) und als ZIP ohne Installation. Ausführsperre unter Windows über die Prozessrichtlinie „keine Kindprozesse“.
- **Linux-Pakete:** AppImage und .deb, beide mit mitgeliefertem Qt.
- **Programmsymbol** (erzeugt mit Z-Image Turbo), Desktop-Datei für Startmenü und „Öffnen mit“.
- **Veröffentlichung auf GitHub:** automatische Prüfung unter Linux und Windows, Releases mit Prüfsummen und Software-Stücklisten; Code-Signatur über SignPath vorbereitet.

### Geändert

- Einstellungen und Lesezeichen liegen jetzt in `~/.config/abolos-pdf` und `~/.local/share/abolos-pdf`. Vorhandene Dateien aus `seitenblick` werden beim ersten Start übernommen.
- „Doppelseiten + Deckblatt“ heißt jetzt „Buchansicht“.
- Die Werkzeugleiste passt auch in schmale Fenster; der Knopf „Tastenbelegung“ sitzt in der unteren Leiste.

### Behoben

- Datum und Uhrzeit von Anmerkungen wurden in UTC statt in Ortszeit angezeigt.

### Qualität und Sicherheit

- Prüfablauf `werkzeuge/pruefen.sh`: Formatierung, Clippy ohne Warnungen, Tests, cargo-deny (RustSec, Lizenzen, Herkunft), Lizenztexte, Software-Stückliste (CycloneDX), REUSE.
- Zusammenspieltests über die echte PDF-Engine: Suche, Ordnersuche, alle Anmerkungsarten, Rückgängig, Speichern und Wiederladen.
- Jede `unsafe`-Stelle begründet (Clippy-Regel `undocumented_unsafe_blocks`).
- SPDX-Kennzeichnung aller Dateien (REUSE 3.3).

## Vorversionen (Arbeitstitel „Seitenblick“)

- **Bearbeiten:** Hervorheben, Unterstreichen, Durchstreichen, Stift, Rechteck, Ellipse, Notiz, Textfeld, Bild, Löschen, Rückgängig, Speichern; Seitenleiste mit Inhalt, Lesezeichen und Anmerkungen.
- **Ansicht:** Doppelseiten, Drehen (alle oder einzelne Seiten); Link-Ziele bleiben rot markiert.
- **Behoben:** Suchtreffer lagen bei beschnittenen PDFs (CropBox) daneben.
- **Grundfunktionen:** PDF-Reader mit Zoom, Suche, Links mit Rücksprung, frei belegbaren Tasten und Netzsperre.
