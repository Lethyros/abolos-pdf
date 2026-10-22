# Benutzerhandbuch

Abolos PDF ist ein PDF-Reader für Linux und Windows. Mit ihm lesen, durchsuchen, markieren und kommentieren Sie PDFs, auch sehr große. Abolos PDF kostet nichts und braucht kein Abo. Das Programm arbeitet **vollständig offline**: Es baut keine Netzwerkverbindungen auf, öffnet keine Webseiten und startet keine anderen Programme (siehe [Sicherheit und Datenschutz](hilfe:sicherheit)).

Fast alles geht wahlweise mit der Maus oder über Tasten, und jede Taste lässt sich ändern. Die vollständige, aktuelle Liste steht im Kapitel [Tastenbelegung](hilfe:tasten). Dort zeigt Abolos PDF die Belegung so, wie Sie sie eingestellt haben. Die Tasten in diesem Handbuch sind die Standardbelegung.

## Installieren

| System | Paket | So geht es |
|---|---|---|
| Windows 10/11 | `Abolos-PDF-…-Setup-x64.exe` | doppelklicken; braucht keine Adminrechte |
| Windows ohne Installation | `…-Windows-x64-ohne-Installation.zip` | entpacken, `abolos-pdf.exe` starten |
| Linux, fast jede Distribution | `Abolos-PDF-…-x86_64.AppImage` | ausführbar machen (Eigenschaften oder `chmod +x`), starten |
| Debian, Ubuntu, Mint | `abolos-pdf_…_amd64.deb` | `sudo apt install ./abolos-pdf_…_amd64.deb` |

**Windows:**

- Der Installer legt Abolos PDF für den angemeldeten Benutzer an. Er braucht keine Adminrechte, Windows fragt nicht nach.
- Als Administrator können Sie „für alle Benutzer“ wählen. Dann bietet der Installer zusätzlich die **Netzsperre** an: eine Windows-Firewall-Regel „Abolos PDF - kein Netzwerk“.
- Auf Wunsch erscheint Abolos PDF unter „Öffnen mit“ bei PDF-Dateien. Als Standardprogramm legen Sie es in den Windows-Einstellungen unter „Standard-Apps“ fest; das darf ein Installer unter Windows 10/11 nicht selbst tun.
- Solange die Installer nicht code-signiert sind, zeigt Windows beim ersten Start die Meldung „Der Computer wurde durch Windows geschützt“. Unter „Weitere Informationen“ → „Trotzdem ausführen“ geht es weiter.
- Deinstallieren: Einstellungen → Apps → Abolos PDF.

**Linux:** Beide Pakete bringen alles mit, was sie brauchen, auch Qt. Weil das mitgelieferte Qt die Bedienelemente des Desktops nicht übernehmen kann, sehen Knöpfe und Felder etwas neutraler aus als in anderen Programmen.

## Öffnen

- **Strg+O** oder der Knopf **Öffnen** zeigt den gewohnten Dateidialog des Desktops (unter KDE mit Vorschau). Er beginnt im Ordner der gerade geöffneten Datei.
- Aus dem Terminal: `abolos-pdf datei.pdf`
- Enthält das offene Dokument ungespeicherte Änderungen, fragt Abolos PDF vor dem Öffnen einer anderen Datei nach.

## Blättern und Ansicht

| Was | Tasten |
|---|---|
| Bildschirmweise weiter / zurück | Bild↓ / Bild↑, Leertaste / Shift+Leertaste |
| Nächste / vorige Seite | Strg+Bild↓ / Strg+Bild↑, N / P |
| Erste / letzte Seite | Pos1 / Ende |
| Ein Stück scrollen | Pfeiltasten oder H J K L |
| Gehe zu Seite | Strg+G (oder Seitenzahl oben eintippen) |
| Vergrößern / verkleinern | Strg++ / Strg+-, oder Strg+Mausrad |
| 100 % / ganze Seite / Seitenbreite | Strg+0 / Strg+1 / Strg+2 |
| Vollbild | F11 |
| Einzelseiten / Doppelseiten / Buchansicht | Strg+3 / Strg+4 (oder D) / Strg+5 |

Die **Buchansicht** zeigt Doppelseiten und das Deckblatt allein, wie in einem gedruckten Buch. Der gewählte Zoom („Seitenbreite“ oder „Ganze Seite“) bleibt erhalten, auch wenn Sie die Fenstergröße ändern oder die Ansicht drehen.

### Drehen

- **Strg+R** dreht alle Seiten im Uhrzeigersinn, **Strg+Shift+R** gegen den Uhrzeigersinn.
- **Strg+Alt+R** dreht nur die aktuelle Seite, ebenso der Rechtsklick auf eine Seite → „Diese Seite drehen“.
- Das Drehen ändert nur die Ansicht, nicht die Datei. Suchtreffer, Links und Anmerkungen drehen mit.

## Links

Fährt die Maus über einen Link, wird er rot umrandet und der Zeiger zur Hand.

- **Interne Links** (z.B. im Inhaltsverzeichnis) springen zum Ziel. Das Ziel blinkt rot und bleibt rot markiert, bis Sie wieder springen oder **Esc** drücken.
- **Zurück:** Alt+← oder Rücktaste, **Vorwärts:** Alt+→.
- **Externe Links** (Web- und Mailadressen) öffnet Abolos PDF bewusst nicht. Es zeigt die Adresse an, und Sie können sie kopieren.

## Suchen

### Im Dokument

- **Strg+F** oder **/** setzt den Cursor ins Suchfeld. Die Suche beginnt schon beim Tippen.
- Alle Treffer werden **gelb**, der ausgewählte **orange** markiert.
- **F3** / **Shift+F3** (oder Enter / Shift+Enter im Suchfeld) springt zum nächsten bzw. vorigen Treffer.
- In der Seitenleiste öffnet sich der Reiter **Suche**. Er listet jeden Treffer mit **Seite und umgebendem Text** auf, ein Klick springt hin.
- **Aa** (Alt+C): Groß-/Kleinschreibung beachten.
- **Wort** (Alt+W): nur ganze Wörter. Dann findet „Schutz“ zwar „Schutz“ und „Schutz-Leiter“, aber nicht „Schutzleiter“ oder „Fehlerschutz“.
- **Esc** beendet die Suche.

Die Suche fasst Leerzeichen und Zeilenumbrüche zu einem Leerzeichen zusammen und löst Ligaturen auf („ﬁ“ wird zu „fi“). Am Zeilenende **getrennte Wörter** findet sie ebenfalls: „Niederspannungsanlagen“ findet auch „Nieder-⏎spannungsanlagen“. Ergänzungsstriche wie in „Erdungs-⏎oder“ bleiben dabei erhalten.

### In einem ganzen Ordner

- **Strg+Shift+F**, oder in der Seitenleiste **Suche → In einem Ordner**. Das geht auch ohne geöffnetes PDF.
- Ordner wählen, Suchbegriff eingeben, Enter. Unterordner werden mit durchsucht (abschaltbar), versteckte Ordner nicht.
- Die Ergebnisse sind nach Dateien gruppiert. Ein Klick öffnet die Datei an der Fundstelle, rahmt sie rot ein und markiert alle weiteren Vorkommen gelb.
- Eine zweite Suche im selben Ordner ist sehr schnell, weil der Text der Dateien im Speicher bleibt, solange das Programm läuft.
- Beschädigte oder mit Passwort geschützte PDFs werden übersprungen und gezählt. Nach 20 000 Treffern endet die Suche.

## Markieren, zeichnen, kommentieren

Die untere Werkzeugleiste enthält die Werkzeuge:

| Werkzeug | Taste | Bedienung |
|---|---|---|
| Hand / Auswahl | Alt+1 (oder Esc) | normal arbeiten; Doppelklick auf eine Anmerkung bearbeitet ihren Kommentar |
| Hervorheben | Alt+2 | mit der Maus über Text ziehen |
| Unterstreichen | Alt+3 | über Text ziehen |
| Durchstreichen | Alt+4 | über Text ziehen |
| Stift (freihand) | Alt+5 | zeichnen |
| Rechteck | Alt+6 | aufziehen, mit Shift quadratisch |
| Ellipse | Alt+7 | aufziehen, mit Shift kreisrund |
| Linie | Alt+L | ziehen, mit Shift waagrecht, senkrecht oder 45° |
| Pfeil | Alt+P | ziehen, mit Shift wie Linie |
| Notiz | Alt+8 | klicken, Text eingeben |
| Textfeld | Alt+9 | Rahmen ziehen, Text eingeben |
| Bild einfügen | Alt+0 | Rahmen ziehen, Bild wählen (PNG, JPEG, GIF, BMP, WebP, TIFF); das Seitenverhältnis bleibt erhalten |
| Anmerkung löschen | Alt+E | Anmerkung anklicken; sie wird vorher rot umrandet |

- **Farbe:** eine der sieben Farben oder „…“ für eine eigene. Jedes Werkzeug merkt sich seine Farbe.
- **Stärke** (Strichbreite in Punkt) bzw. **Schrift** (Schriftgröße beim Textfeld).
- **Rückgängig:** Strg+Z entfernt die zuletzt angelegte Anmerkung.

Alle Markierungen und Zeichnungen sind **echte PDF-Anmerkungen**. Andere Programme (z.B. Okular, Acrobat) zeigen sie ebenso an. Jede Anmerkung erhält als Autor Ihren Benutzernamen sowie Datum und Uhrzeit.

## Seitenleiste

**F4** blendet die Seitenleiste ein und aus. Ihre Breite können Sie mit der Maus ziehen.

- **Inhalt:** das Inhaltsverzeichnis des PDFs. Ein Klick springt hin.
- **Lesezeichen:** eigene Lesezeichen. **Strg+B** setzt eines an der aktuellen Stelle. Rechtsklick → umbenennen oder entfernen.
- **Anmerkungen:** alle Anmerkungen des Dokuments mit Art, Seite, Kommentar, Autor und Datum, dazu ein Filterfeld.
  - Ein Klick springt hin und rahmt die Stelle rot ein.
  - Doppelklick bearbeitet den Kommentar.
  - Rechtsklick → bearbeiten oder löschen.
- **Suche:** Trefferliste im Dokument und Ordnersuche (siehe oben).

Eigene Lesezeichen werden **nicht ins PDF** geschrieben, sondern je Datei in `~/.local/share/abolos-pdf/lesezeichen.json` gespeichert. Sie hängen am Dateipfad. Verschieben oder benennen Sie die Datei um, sind sie dort nicht mehr zu sehen.

## Speichern

- **Strg+S** speichert in die geöffnete Datei, **Strg+Shift+S** unter neuem Namen.
- Ein „●“ im Fenstertitel und ein Hinweis in der Statuszeile zeigen ungespeicherte Änderungen an. Beim Schließen fragt Abolos PDF nach.
- Abolos PDF speichert zuerst eine Kopie und ersetzt die Datei erst, wenn die Kopie vollständig geschrieben ist. Bricht das Speichern ab (z.B. bei vollem Datenträger), bleibt die alte Datei unverändert.

## Tastenbelegung ändern

**Strg+,** öffnet den Dialog mit allen Aktionen. Jede Aktion kann mehrere Tasten haben. Ist eine Taste schon vergeben, wird sie dort entfernt, und der Dialog sagt wo. Die Belegung steht in `~/.config/abolos-pdf/tasten.toml` und lässt sich auch dort bearbeiten.

## Bekannte Grenzen

- Textfelder und Bilder werden immer aufrecht zur Seite eingefügt, auch in gedrehter Ansicht. Markieren, Zeichnen und Formen funktionieren gedreht richtig.
- Rückgängig entfernt nur selbst angelegte Anmerkungen. Gelöschte Anmerkungen holt es nicht zurück.
- Drucken, Formulare ausfüllen und Seiten bearbeiten (einfügen, löschen, umsortieren) gibt es noch nicht.
- JavaScript und XFA-Formulare in PDFs werden aus Sicherheitsgründen grundsätzlich nicht ausgeführt.
- Gescannte PDFs ohne Texterkennung (OCR) enthalten keinen Text. Suchen und Markieren finden dort nichts.

## Wo Abolos PDF Daten ablegt

| Datei (Linux) | Datei (Windows) | Inhalt |
|---|---|---|
| `~/.config/abolos-pdf/tasten.toml` | `%APPDATA%\abolos-pdf\tasten.toml` | Ihre Tastenbelegung |
| `~/.local/share/abolos-pdf/lesezeichen.json` | `%APPDATA%\abolos-pdf\lesezeichen.json` | eigene Lesezeichen je Datei |
| die PDF-Datei selbst | die PDF-Datei selbst | Anmerkungen, aber nur beim Speichern |

Weitere Daten speichert Abolos PDF nicht, auch keine Liste zuletzt geöffneter Dateien. Einstellungen aus der Zeit vor der Umbenennung (Ordner `seitenblick`) werden beim ersten Start übernommen. Der alte Ordner bleibt dabei als Sicherung erhalten.

## Kommandozeile

```
abolos-pdf [DATEI.pdf]    Programm starten, optional mit einer Datei
abolos-pdf --version      Version anzeigen
abolos-pdf --help         kurze Hilfe
```

Für Fachleute gibt es diese Umgebungsvariablen:

- `ABOLOS_QT_DIALOGE=1`: Qts eigene Dateidialoge statt der des Desktops.
- `ABOLOS_PDFIUM=/pfad/libpdfium.so`: eine andere PDFium-Bibliothek verwenden.
- `ABOLOS_NETZTEST=1`: Beim Start werden Netz- und Ausführsperre geprüft, das Ergebnis erscheint im Terminal. Unter Windows funktioniert das in der Eingabeaufforderung oder PowerShell.
