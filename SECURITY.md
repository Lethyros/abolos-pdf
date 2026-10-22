# Sicherheitslücken melden

## Ansprechpartner

Sicherheitsfragen und vermutete Lücken bitte **vertraulich** an den Maintainer **Oliver Weber** melden: auf GitHub im Reiter **Security → Report a vulnerability** (private Meldung). Bitte nicht als öffentliches Issue. Hilfreich sind:

- Version (`abolos-pdf --version` oder Hilfe → Über Abolos PDF)
- Beschreibung und Auswirkung
- wenn möglich eine Datei, mit der sich das Verhalten nachstellen lässt (bei vertraulichen Dokumenten bitte nur nach Absprache)

## Ablauf

1. **Eingang bestätigen:** innerhalb von 3 Arbeitstagen.
2. **Bewerten:** Ist die Lücke echt, wie schwer wiegt sie, welche Versionen sind betroffen? Dafür die Software-Stückliste `sbom/abolos-pdf.cdx.json` heranziehen.
3. **Beheben:** Bei schweren Lücken bekommt die nächste Version Vorrang. Lücken in Fremdkomponenten werden durch deren Aktualisierung behoben:
   - PDFium: siehe „PDFium aktualisieren“ in `doku/entwicklung.md`
   - Rust-Crates: `cargo update`, dann `werkzeuge/pruefen.sh`
   - Qt: Update der Linux-Distribution
4. **Informieren:** Anwender über die neue Version und den Grund informieren; Eintrag in `CHANGELOG.md`.

## Laufende Überwachung

`werkzeuge/pruefen.sh` gleicht alle Rust-Abhängigkeiten mit der RustSec-Datenbank ab (`cargo deny check`). Es sollte vor jeder Version und zusätzlich regelmäßig laufen (z.B. monatlich), auch ohne Codeänderung, weil neue Lücken in bestehenden Abhängigkeiten bekannt werden können.

## Unterstützte Versionen

Sicherheitskorrekturen gibt es für die jeweils neueste Version.
