# Sicherheit und Datenschutz

PDF-Dateien kommen oft von außen: von Kunden, Herstellern, aus E-Mails. Für ein Programm, das solche Dateien öffnet, sind sie die größte Angriffsfläche. Abolos PDF ist deshalb so gebaut, dass es **nichts nach außen sendet** und dass auch eine manipulierte Datei möglichst wenig anrichten kann. Den Zustand der Schutzmaßnahmen in der laufenden Sitzung zeigt das Kapitel [Über Abolos PDF](hilfe:ueber).

## 1. Keine Netzwerkfunktionen, technisch erzwungen

Abolos PDF hat keine Funktion, die das Netzwerk nutzt: keine Updates, keine Telemetrie, keine Online-Hilfe, kein Laden von Inhalten aus dem Internet.

Darauf verlässt sich das Programm nicht allein.

**Linux:** Beim Start zieht sich das Programm in einen **eigenen, leeren Netzwerk-Namensraum** des Linux-Kernels zurück (`unshare(CLONE_NEWUSER | CLONE_NEWNET)`).

- Darin gibt es keine Netzwerkkarte, nur eine abgeschaltete Loopback-Schnittstelle. Ab diesem Moment kann **nichts im Prozess** eine Verbindung aufbauen: weder Abolos PDF selbst noch Qt, PDFium oder Inhalte einer PDF-Datei.
- Das ist nötig, weil Qt Netzwerk-Bibliotheken mitbringt (Qt Network), auch wenn Abolos PDF sie nicht benutzt.
- Unix-Sockets mit Dateipfad funktionieren weiter. Darüber laufen die Anzeige (Wayland/X11) und der Dateidialog (D-Bus zum Desktop-Portal).
- Prüfen: Mit `ABOLOS_NETZTEST=1 abolos-pdf` versucht das Programm beim Start eine Verbindung und meldet im Terminal „blockiert: Network is unreachable“.
- Kann die Sperre nicht eingerichtet werden (etwa weil der Kernel unprivilegierte User-Namensräume verbietet), zeigt die Statuszeile rot „Netzsperre aus“. Das Programm selbst baut auch dann keine Verbindungen auf.

**Windows:** Windows kennt keinen gleichwertigen Mechanismus, den ein Programm ohne Adminrechte bei sich selbst einschalten kann. Die Netzsperre ist deshalb eine **Regel der Windows-Firewall** („Abolos PDF - kein Netzwerk“), die jede ein- und ausgehende Verbindung des Programms blockiert.

- Der Installer legt sie an, wenn Sie als Administrator **für alle Benutzer** installieren und die Option „Netzsperre“ angehakt lassen. Beim Deinstallieren wird sie wieder entfernt.
- Bei einer Installation nur für den eigenen Benutzer (ohne Adminrechte) gibt es keine Firewall-Regel. Das Programm hat auch dann keine Netzwerkfunktionen, nur fehlt die zusätzliche technische Absicherung.
- Selbst anlegen (PowerShell als Administrator, Pfad anpassen):
  `New-NetFirewallRule -DisplayName "Abolos PDF - kein Netzwerk" -Direction Outbound -Action Block -Program "C:\Program Files\Abolos PDF\abolos-pdf.exe"`
- Das Fenster „Über Abolos PDF“ beschreibt unter Windows, wie die Netzsperre dort umgesetzt ist. Ob die Regel besteht, zeigt die Windows-Firewall („Windows Defender Firewall mit erweiterter Sicherheit“ → Ausgehende Regeln).

**Externe Links** in PDFs (Webseiten, E-Mail-Adressen) werden nie geöffnet. Abolos PDF zeigt nur die Adresse an, Sie können sie kopieren.

## 2. Keine anderen Programme startbar (Ausführsperre)

Direkt nach dem Start schränkt sich Abolos PDF so ein, dass der Prozess **keine Programme mehr ausführen** kann, auch keine Shell:

- **Linux:** mit **Landlock**, einer Sicherheitsfunktion des Linux-Kernels (ab Version 5.13).
- **Windows:** mit der Prozessrichtlinie „keine Kindprozesse“ (`ProcessChildProcessPolicy`, Windows 10 ab 1709). Sie lässt sich danach nicht wieder aufheben.

Was das bewirkt:

- Gelänge es einer manipulierten PDF-Datei, über eine Lücke in PDFium eigenen Code im Programm auszuführen, könnte dieser keine weiteren Programme nachladen oder starten.
- Bibliotheken und Qt-Plugins lädt das Programm weiterhin normal.
- Prüfen: `ABOLOS_NETZTEST=1 abolos-pdf` meldet auch „Ausführtest: … blockiert“ (unter Windows mit `cmd.exe`).

## 3. Kein JavaScript, keine aktiven Inhalte

Die mitgelieferte PDFium-Bibliothek ist **ohne JavaScript-Engine (V8) und ohne XFA** gebaut (`pdf_enable_v8 = false`, `pdf_enable_xfa = false`, siehe `third_party/pdfium/args.gn`). Skripte in PDF-Dateien können deshalb nicht laufen, egal was die Datei enthält. Eingebettete Dateien, Formular-Aktionen und „Starte Programm“-Aktionen führt Abolos PDF ebenfalls nie aus.

## 4. Speichersicherheit

Abolos PDF ist in **Rust** geschrieben. Der Compiler verhindert die häufigsten Ursachen von Sicherheitslücken schon beim Übersetzen:

- Pufferüberläufe (CWE-787, CWE-120)
- Zugriff auf freigegebenen Speicher (CWE-416)
- Nullzeiger (CWE-476)
- Datenrennen zwischen Threads (CWE-362)

Speichersichere Sprachen empfehlen unter anderem die US-Behörden CISA und NSA („memory-safe languages“).

An wenigen, eng begrenzten Stellen muss Rust mit C/C++ reden. Diese Stellen sind mit `unsafe` gekennzeichnet:

| Stelle | Wozu |
|---|---|
| Qt-Brücken (`unsafe extern "C++"` in dokument.rs, hilfe.rs, seite.rs, tasten.rs) | Schnittstellenbeschreibung für cxx-qt; die Typen prüft cxx beim Übersetzen |
| seite.rs: Bildübergabe an Qt, Zeichenfläche | gerenderte Seiten ohne Kopie an Qt übergeben; Zeiger, den Qt beim Zeichnen übergibt |
| schutz.rs: Systemaufrufe | Netz- und Ausführsperre (Linux: `unshare`, `prctl`, Landlock; Windows: `SetProcessMitigationPolicy`), Konsolenausgabe unter Windows, Umgebungsvariable vor dem Start |
| bearbeiten.rs: zwei direkte PDFium-Aufrufe | Betreff (/Subj) einer Anmerkung lesen und schreiben |

Jede dieser Stellen trägt einen Kommentar `SAFETY:`, der begründet, warum sie korrekt ist. Die Codeprüfung (Clippy, Regel `undocumented_unsafe_blocks`) meldet jede Stelle ohne Begründung.

## 5. Die verbleibende Angriffsfläche: PDFium

Die PDF-Engine **PDFium** ist in C++ geschrieben. Sie stammt aus dem Chromium-Projekt (Google Chrome), wird dort ständig gepflegt und auf Sicherheitslücken getestet (Fuzzing). Trotzdem bleibt sie die Stelle, an der fremde Daten verarbeitet werden. Folgendes ist zu ihrer Absicherung umgesetzt:

- Netz- und Ausführsperre (siehe oben): Selbst bei einer Lücke gibt es keinen Weg nach außen und kein Nachladen von Programmen.
- PDFium läuft in einem eigenen Thread mit festen Schnittstellen. Die Oberfläche bekommt nur fertige Bilder und Daten.
- Keine Skripte, kein XFA (siehe oben).

**Restrisiko:** Code, der über eine PDFium-Lücke im Programm läuft, hätte die Dateirechte des angemeldeten Benutzers. Er könnte Dateien lesen und verändern, die der Benutzer lesen und verändern darf. Dateizugriffe schränkt Abolos PDF bewusst nicht ein, weil Sie PDFs von überall öffnen und überall speichern können müssen. Gegenmaßnahme: PDFium regelmäßig aktualisieren (siehe [Für Entwickler](hilfe:entwicklung)).

## 6. Abhängigkeiten und Schwachstellen

- **Bekannte Lücken:** `cargo deny check` gleicht alle Rust-Bibliotheken mit der RustSec-Datenbank ab. Gemeldete Lücken lassen die Prüfung fehlschlagen.
- **Software-Stückliste (SBOM):** `sbom/abolos-pdf.cdx.json` listet alle Bestandteile im CycloneDX-Format, einschließlich PDFium und Qt. Damit lässt sich bei einer neuen Sicherheitsmeldung sofort klären, ob Abolos PDF betroffen ist.
- **Qt und Systembibliotheken** kommen aus der Linux-Distribution und werden mit deren Updates aktualisiert.
- **Meldung von Lücken:** siehe Datei `SECURITY.md` im Quellcode.

## 7. Datenschutz

- Abolos PDF sendet **keine Daten**. Es gibt keine Telemetrie, keine Absturzberichte und keine Nutzungsstatistik.
- Gespeichert werden nur Ihre Tastenbelegung und Ihre Lesezeichen (siehe [Benutzerhandbuch](hilfe:handbuch), „Wo Abolos PDF Daten ablegt“). Eine Liste zuletzt geöffneter Dateien gibt es nicht.
- **Hinweis zu Anmerkungen:** Jede neue Anmerkung enthält als Autor Ihren **Linux-Benutzernamen** sowie Datum und Uhrzeit. Diese Angaben stehen nach dem Speichern in der PDF-Datei und sind für jeden sichtbar, der die Datei erhält. Das ist bei PDF-Anmerkungen üblich (Acrobat und Okular halten es genauso), sollte aber vor der Weitergabe von Dateien nach außen bekannt sein.
- Ungespeicherte Anmerkungen existieren nur im Arbeitsspeicher und sind nach dem Schließen ohne Speichern weg.

## 8. Zuverlässigkeit beim Speichern

Beim Speichern schreibt Abolos PDF zuerst eine vollständige Kopie (`datei.pdf.abolos-tmp`) und ersetzt die Originaldatei erst danach in einem Schritt. Ein Abbruch mittendrin (Stromausfall, voller Datenträger) hinterlässt deshalb nie eine halb geschriebene PDF-Datei. Vor dem Schließen und vor dem Öffnen einer anderen Datei fragt das Programm, ob ungespeicherte Änderungen gesichert werden sollen.
