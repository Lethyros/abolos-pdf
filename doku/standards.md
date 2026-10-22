# Qualität und Firmenstandards

Abolos PDF ist ein Werkzeug zum Lesen und Kommentieren von Dokumenten. Es steuert keine Maschinen, misst nichts und verarbeitet keine Daten für andere. Deshalb ist für den Einsatz im Unternehmen **keine Norm verpflichtend**.

Damit das Programm trotzdem guten Gewissens im Firmenalltag eingesetzt und von einem Qualitäts- oder Informationssicherheits-Managementsystem freigegeben werden kann, orientieren sich Entwicklung und Dokumentation an anerkannten Normen. Dieses Kapitel nennt für jede Norm drei Dinge:

- was sie verlangt,
- was Abolos PDF dafür tut und wo das nachprüfbar ist,
- was (noch) fehlt.

> **Wichtig:** „Orientiert an“ heißt nicht „zertifiziert“. Eine Zertifizierung prüft das Managementsystem eines Unternehmens, nicht ein einzelnes Programm. Die hier beschriebenen Maßnahmen und Nachweise sind Bausteine, auf die sich ein Unternehmen bei seiner eigenen Zertifizierung stützen kann.

## Überblick

| Norm / Standard | Worum es geht | Stand bei Abolos PDF |
|---|---|---|
| ISO/IEC 25010 | Qualitätsmerkmale von Software | umgesetzt |
| ISO/IEC 27001 / 27002:2022 | Informationssicherheit, sichere Entwicklung | Maßnahmen 5.32, 5.37, 8.8, 8.25 bis 8.29, 8.32 weitgehend umgesetzt |
| ISO/IEC 5055 | messbare Schwachstellen im Quellcode | teilweise (durch Sprache und statische Analyse) |
| BSI IT-Grundschutz CON.8, BSI TR-03185 | sichere Softwareentwicklung | teilweise |
| EU Cyber Resilience Act (VO (EU) 2024/2847) | Sicherheitspflichten für Produkte auf dem EU-Markt | gilt nicht für interne Nutzung, wesentliche Unterlagen vorbereitet |
| ISO 9001:2015 (mit ISO/IEC 90003) | Qualitätsmanagement: Dokumentation, Rückverfolgbarkeit | umgesetzt für Dokumentation und Kennzeichnung |
| ISO/IEC/IEEE 29119 | Softwaretest | teilweise (automatische Tests, ohne formales Testkonzept) |
| GPL-3.0, REUSE 3.3, SPDX, CycloneDX, ISO/IEC 5230 (OpenChain) | Lizenz-Compliance für Open Source | Unterlagen vollständig |
| IEC 61508, ISO 13849, IEC 62443, ISO 26262, IEC 62304 | funktionale Sicherheit, Industrieautomation | nicht anwendbar (Begründung unten) |

## ISO/IEC 25010: Produktqualität

ISO/IEC 25010 beschreibt, was „gute Software“ ausmacht. Die Merkmale entsprechen der Ausgabe 2023; die früheren Namen stehen in Klammern.

| Merkmal | Umsetzung in Abolos PDF | Nachweis |
|---|---|---|
| Funktionale Eignung | Lesen, Suchen (Dokument und Ordner), Markieren, Zeichnen, Kommentieren, Speichern; automatische Tests für Suche, alle zwölf Anmerkungsarten, Speichern und Laden | `cargo test`, src/pruefung.rs |
| Leistungseffizienz | PDFium in eigenem Thread; Bild-Zwischenspeicher (768 MB); nur sichtbare Seiten werden gerendert; Ordnersuche in Zeitscheiben von 15 ms, damit die Oberfläche flüssig bleibt; Text-Zwischenspeicher für wiederholte Suchen | Quellcode engine.rs |
| Kompatibilität | Anmerkungen sind echte PDF-Anmerkungen nach ISO 32000 (PDF-Norm) und in anderen Programmen sichtbar; Dateidialog des Desktops über xdg-desktop-portal | Test „speichern und wieder laden“ |
| Interaktionsfähigkeit (Benutzbarkeit) | vollständig per Tastatur bedienbar; jede Taste änderbar; deutsche Oberfläche; eingebaute Hilfe (F1); Rückfrage vor Datenverlust | dieses Hilfesystem |
| Zuverlässigkeit | atomares Speichern (erst Kopie, dann Ersetzen); kaputte Dateien führen zu Meldungen, nicht zu Abstürzen; Tests prüfen die Fehlerfälle | src/bearbeiten.rs `speichern`, Tests |
| Informationssicherheit (Sicherheit) | siehe [Sicherheit und Datenschutz](hilfe:sicherheit): Netz- und Ausführsperre, kein JavaScript, speichersichere Sprache | Kapitel Sicherheit |
| Wartbarkeit | klar getrennte Module; einheitliche Formatierung (rustfmt); statische Analyse (Clippy) ohne Warnungen; Entwicklerdokumentation | `werkzeuge/pruefen.sh` |
| Flexibilität (Übertragbarkeit) | läuft auf jedem Linux mit Qt 6 und Wayland oder X11; Speicherorte nach XDG-Standard; Tastenbelegung als Textdatei | [Für Entwickler](hilfe:entwicklung) |
| Schutz vor Gefährdungen (neu 2023) | nicht relevant: Das Programm steuert nichts, was Menschen gefährden kann | – |

## ISO/IEC 27001 und 27002:2022: Informationssicherheit

ISO/IEC 27001 verlangt ein Managementsystem für Informationssicherheit. Die Maßnahmen dazu stehen in Anhang A, erläutert in ISO/IEC 27002. Für Software, die ein Unternehmen selbst entwickelt, sind diese Maßnahmen wichtig:

| Maßnahme (27002:2022) | Was sie verlangt | Umsetzung bei Abolos PDF |
|---|---|---|
| 5.32 Geistiges Eigentum | Lizenzen der verwendeten Software einhalten | alle Lizenzen erfasst und auf Verträglichkeit geprüft (cargo-deny), Lizenztexte im Programm, SPDX-Kennzeichnung jeder Datei; siehe [Verwendete Software und Lizenzen](hilfe:fremdsoftware) |
| 5.21 Informationssicherheit in der IKT-Lieferkette | fremde Bestandteile kontrolliert beziehen | PDFium und die AppImage-Werkzeuge nur mit fest eingetragener SHA256-Prüfsumme; alle GitHub-Aktionen auf feste Commits festgelegt; Rust-Bibliotheken nur von crates.io (`deny.toml`); Pakete mit Prüfsummen (`SHA256SUMS.txt`) |
| 5.37 Dokumentierte Betriebsabläufe | Bedienung und Betrieb dokumentiert | dieses Handbuch, im Programm und als Dateien in `doku/` |
| 8.8 Umgang mit technischen Schwachstellen | Schwachstellen erkennen und beheben | RustSec-Abgleich (`cargo deny check`) bei jedem Push, monatliche Update-Vorschläge (Dependabot), Software-Stückliste (SBOM) je Plattform, Meldeweg in `SECURITY.md` |
| 8.25 Sicherer Entwicklungslebenszyklus | Sicherheit in jedem Entwicklungsschritt | fester Prüfablauf vor jeder Version (`werkzeuge/pruefen.sh`, unten); Freigabe-Checkliste in [Für Entwickler](hilfe:entwicklung) |
| 8.26 Anforderungen an Anwendungssicherheit | Sicherheitsanforderungen festlegen | festgelegt und umgesetzt: offline, keine Skripte, keine fremden Programme, keine Telemetrie (Kapitel Sicherheit) |
| 8.27 Sichere Systemarchitektur | Prinzipien wie „geringste Rechte“, „Verteidigung in der Tiefe“ | mehrere Schutzschichten: speichersichere Sprache, PDFium ohne Skripte, Netzsperre, Ausführsperre |
| 8.28 Sichere Programmierung | Regeln für sicheren Code, Werkzeuge zur Prüfung | Rust; Clippy mit Fehlerabbruch bei Warnungen; jede `unsafe`-Stelle begründet (`undocumented_unsafe_blocks`); keine ungeprüften Eingaben in Systemaufrufe |
| 8.29 Sicherheitstests | Tests in Entwicklung und Abnahme | automatische Tests unter Linux und Windows bei jedem Push (GitHub Actions), Abhängigkeitsprüfung, Selbsttest der Sperren (`ABOLOS_NETZTEST=1`) |
| 8.32 Änderungssteuerung | Änderungen nachvollziehbar | Versionsnummer, `CHANGELOG.md`, Versionsverwaltung mit Git |

**Noch offen:**

- **Vier-Augen-Prinzip:** 8.28 empfiehlt die Durchsicht von Änderungen durch eine zweite Person. Bei einem Ein-Personen-Projekt ersetzt das die automatische Prüfung nur teilweise.
- **Abnahmetest:** Eine protokollierte Abnahme durch Anwender vor dem Einsatz fehlt noch (Vorschlag in [Für Entwickler](hilfe:entwicklung)).
- **Unternehmensaufgaben:** Aufnahme ins Software-Inventar und Freigabe für die Installation (8.19) regelt das Unternehmen.

## ISO/IEC 5055: Messbare Codequalität

ISO/IEC 5055 zählt typische Schwachstellen im Quellcode (auf Grundlage der CWE-Liste), gruppiert nach Sicherheit, Zuverlässigkeit, Leistungseffizienz und Wartbarkeit.

- **Umgesetzt:** Rust schließt die größten Gruppen schon beim Übersetzen aus: Pufferüberläufe, Zugriffe auf freigegebenen Speicher, Nullzeiger, Datenrennen. Die statische Analyse mit Clippy prüft zusätzlich Hunderte typischer Fehlermuster und schlägt bei jeder Warnung fehl.
- **Nicht umgesetzt:** eine formale Messung nach ISO/IEC 5055 mit einem dafür zugelassenen Analysewerkzeug.

## BSI IT-Grundschutz CON.8 und BSI TR-03185

Der Baustein CON.8 („Software-Entwicklung“) des IT-Grundschutz-Kompendiums und die Technische Richtlinie TR-03185 („Sicherer Software-Lebenszyklus“) beschreiben sichere Entwicklung aus deutscher Sicht. Abolos PDF erfüllt davon:

- dokumentierte Anforderungen und Architektur,
- Versionsverwaltung,
- feste Prüfschritte vor jeder Version,
- Prüfung der Fremdkomponenten,
- Software-Stückliste,
- Meldeweg für Schwachstellen,
- sichere Grundeinstellungen („secure by default“: offline ohne Einstellmöglichkeit, nichts abzuschalten).

Nicht umgesetzt sind formale Bedrohungsmodellierung, Penetrationstests und Fuzzing durch Dritte.

## EU Cyber Resilience Act

Die Verordnung (EU) 2024/2847 gilt für „Produkte mit digitalen Elementen“, die **auf dem EU-Markt bereitgestellt** werden. Die Meldepflichten gelten seit September 2026, die übrigen Pflichten ab Dezember 2027. Wer Abolos PDF nur im eigenen Unternehmen nutzt, stellt es nicht auf dem Markt bereit, die Verordnung gilt dann nicht.

Sollte Abolos PDF einmal verkauft oder im Rahmen einer Geschäftstätigkeit an Kunden weitergegeben werden, sind die wichtigsten Unterlagen schon vorhanden:

- Software-Stückliste (SBOM) im CycloneDX-Format
- dokumentierter Meldeweg und Umgang mit Schwachstellen (`SECURITY.md`)
- sichere Grundeinstellungen
- Benutzer- und Sicherheitsdokumentation

Dann noch nötig wären unter anderem eine Konformitätsbewertung, eine EU-Konformitätserklärung, ein festgelegter Unterstützungszeitraum und die Meldung aktiv ausgenutzter Lücken an die ENISA.

## ISO 9001: Qualitätsmanagement

Für Software, die im Unternehmen eingesetzt wird, sind vor allem diese Punkte aus ISO 9001 (Auslegung für Software in ISO/IEC 90003) wichtig:

- **7.5 Dokumentierte Information:** Handbuch, Sicherheits- und Entwicklerdokumentation liegen versioniert beim Quellcode und im Programm. Die Dokumentation passt deshalb immer zur installierten Version.
- **8.5.2 Kennzeichnung und Rückverfolgbarkeit:** eindeutige Versionsnummer (`abolos-pdf --version`, Fenster „Über“); `CHANGELOG.md` beschreibt jede Version; die SBOM hält alle Bestandteile mit Version fest.
- **7.1.6 Wissen der Organisation:** Aufbau, Prüfablauf und Entscheidungen sind dokumentiert, nicht nur im Kopf des Entwicklers.

## ISO/IEC/IEEE 29119: Softwaretest

- **Umgesetzt:** automatische Einzeltests (Suche, Seitenanordnung, Datumsanzeige, Tastennamen, Hilfe, Speicherorte) und Zusammenspieltests über die echte PDF-Engine (Suchen, Ordnersuche mit Zwischenspeicher, alle Anmerkungsarten, Rückgängig, Speichern und Wiederladen, Fehlerfälle).
  - Die Testdaten werden von einem Skript erzeugt (`tests/daten/erzeugen.py`).
  - Getestet wird nur auf Kopien, nie auf Originaldateien.
- **Nicht umgesetzt:** formales Testkonzept, Testprotokolle nach 29119-3 und automatische Tests der Mausbedienung in der Oberfläche.

## Lizenz-Compliance (GPL-3.0, REUSE, SPDX, CycloneDX, OpenChain)

- Abolos PDF steht unter der **GNU GPL, Version 3 oder später**. Alle eingesetzten Fremdkomponenten haben damit verträgliche Lizenzen; das prüft `cargo deny check` bei jedem Lauf (siehe [Verwendete Software und Lizenzen](hilfe:fremdsoftware)).
- Jede Quelldatei trägt einen **SPDX-Kopf** mit Rechteinhaber und Lizenz, nach der **REUSE-Spezifikation 3.3** der Free Software Foundation Europe. Dateien ohne Kopf erfasst `REUSE.toml`, die Lizenztexte liegen in `LICENSES/`.
- Die GPLv3 verlangt bei Programmen mit Oberfläche „angemessene rechtliche Hinweise“ (Abschnitt 0). Diese zeigt das Kapitel [Über Abolos PDF](hilfe:ueber): Copyright, Lizenz, Gewährleistungsausschluss, Verweis auf den Lizenztext.
- Die **Software-Stückliste** im CycloneDX-Format und die Sammlung aller Lizenztexte sind genau die Unterlagen, die ein Open-Source-Compliance-Programm nach **ISO/IEC 5230 (OpenChain)** für jede eingesetzte Software verlangt. Das Programm selbst einzurichten, ist Aufgabe des Unternehmens.

## Nicht anwendbare Normen und warum

| Norm | Bereich | Warum nicht anwendbar |
|---|---|---|
| IEC 61508, ISO 13849, IEC 62061 | funktionale Sicherheit von Steuerungen und Maschinen | Abolos PDF steuert nichts und hat keine Sicherheitsfunktion |
| IEC 62443 | Security von Industrieautomation | kein Bestandteil einer Automatisierungs- oder Leittechnik |
| ISO 26262, IEC 62304, EN 50716, DO-178C | Auto, Medizin, Bahn, Luftfahrt | kein Einsatz in diesen Produkten |
| EN 301 549 | Barrierefreiheit (öffentliche Stellen) | nicht vorgeschrieben; vollständig per Tastatur bedienbar, aber nicht mit Bildschirmlesern geprüft |

Ändert sich der Einsatzzweck, muss diese Bewertung neu gemacht werden. Das gilt zum Beispiel, wenn Abolos PDF Prüfberichte erzeugen oder Messwerte verarbeiten soll.

## Der Prüfablauf vor jeder Version

`werkzeuge/pruefen.sh` führt alle Prüfungen aus und bricht beim ersten Fehler ab:

| Schritt | Werkzeug | dient |
|---|---|---|
| 1. Formatierung | `cargo fmt --check` | Wartbarkeit (25010) |
| 2. Statische Analyse | `cargo clippy -- -D warnings` | sichere Programmierung (27002 8.28), ISO/IEC 5055 |
| 3. Tests | `cargo test` | Funktion und Zuverlässigkeit (25010, 29119, 27002 8.29) |
| 4. Abhängigkeiten | `cargo deny check` | Schwachstellen (27002 8.8), Lizenzen (5.32), Herkunft |
| 5. Lizenztexte | `werkzeuge/lizenzen.sh` | Lizenz-Compliance |
| 6. Software-Stückliste | `werkzeuge/sbom.py` | Schwachstellenmanagement, CRA-Vorbereitung |
| 7. REUSE (falls installiert) | `reuse lint` | SPDX-Kennzeichnung aller Dateien |
