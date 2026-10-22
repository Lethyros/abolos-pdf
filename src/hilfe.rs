// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hilfe als QML-Singleton `Hilfe`: Die Dokumentation ist ins Programm eingebettet (offline lesbar,
//! passt immer zur Version), dazu „Über“ mit den rechtlichen Hinweisen, die die GPLv3 für Programme
//! mit Oberfläche verlangt (Abschnitt 0, „Appropriate Legal Notices“).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type Hilfe = super::HilfeRust;

        /// Kapitel als JSON-Liste [{id, titel, art}]; art: "markdown", "text" oder "tasten"
        #[qinvokable]
        fn kapitel(self: &Hilfe) -> QString;

        /// Inhalt eines Kapitels (Markdown oder reiner Text, siehe `art`)
        #[qinvokable]
        fn inhalt(self: &Hilfe, id: &QString) -> QString;

        /// Name, Version, Autor, Lizenz und Schutzstatus als JSON (für Titelzeilen)
        #[qinvokable]
        fn info(self: &Hilfe) -> QString;
    }
}

use cxx_qt_lib::QString;
use serde_json::json;

pub const PROGRAMMNAME: &str = "Abolos PDF";
pub const AUTOR: &str = "Oliver Weber";
pub const LIZENZ: &str = "GPL-3.0-or-later";

/// (id, Titel, Art, Inhalt); "ueber" und "tasten" entstehen zur Laufzeit
const KAPITEL: &[(&str, &str, &str, &str)] = &[
    ("handbuch", "Benutzerhandbuch", "markdown", include_str!("../doku/handbuch.md")),
    ("tasten", "Tastenbelegung", "tasten", ""),
    ("sicherheit", "Sicherheit und Datenschutz", "markdown", include_str!("../doku/sicherheit.md")),
    ("standards", "Qualität und Firmenstandards", "markdown", include_str!("../doku/standards.md")),
    ("fremdsoftware", "Verwendete Software und Lizenzen", "markdown", include_str!("../doku/fremdsoftware.md")),
    ("entwicklung", "Für Entwickler", "markdown", include_str!("../doku/entwicklung.md")),
    ("aenderungen", "Änderungen", "markdown", include_str!("../CHANGELOG.md")),
    ("lizenz", "Lizenz (GNU GPL 3)", "text", include_str!("../LICENSE")),
    ("drittanbieter", "Lizenztexte der Fremdsoftware", "text", include_str!("../lizenzen/DRITTANBIETER.txt")),
    ("ueber", "Über Abolos PDF", "markdown", ""),
];

#[derive(Default)]
pub struct HilfeRust {}

/// Schutzstatus in Worten (Netz- und Ausführsperre)
fn schutz_text() -> (String, String) {
    match crate::dokument::SCHUTZ.get() {
        Some(s) => (s.netz.beschreibung(), s.ausfuehren.beschreibung()),
        None => ("unbekannt".into(), "unbekannt".into()),
    }
}

/// „Über“: Version, Bausteine, Schutzstatus und die Hinweise nach GPLv3
pub fn ueber() -> String {
    let (netz, ausfuehren) = schutz_text();
    let ordner = |p: std::path::PathBuf| p.parent().map(|p| p.display().to_string()).unwrap_or_default();
    format!(
        "# {name} {version}\n\n\
         Offline-PDF-Reader für Linux und Windows: lesen, suchen, markieren, kommentieren – ohne Abo.\n\n\
         **Autor und Maintainer:** {autor}  \n\
         **Lizenz:** GNU General Public License, Version 3 oder später ({lizenz})\n\n\
         ## Rechtliche Hinweise\n\n\
         Copyright © 2026 {autor}\n\n\
         Dieses Programm ist freie Software: Sie können es unter den Bedingungen der GNU General Public \
         License, wie von der Free Software Foundation veröffentlicht, weitergeben und/oder verändern, \
         entweder gemäß Version 3 der Lizenz oder (nach Ihrer Wahl) jeder späteren Version.\n\n\
         Dieses Programm wird in der Hoffnung bereitgestellt, dass es nützlich ist, aber **OHNE JEDE \
         GEWÄHRLEISTUNG**, sogar ohne die implizite Gewährleistung der MARKTFÄHIGKEIT oder EIGNUNG FÜR EINEN \
         BESTIMMTEN ZWECK. Einzelheiten finden Sie im Kapitel [Lizenz (GNU GPL 3)](hilfe:lizenz).\n\n\
         Verwendete Fremdsoftware und deren Lizenzen: [Verwendete Software und Lizenzen](hilfe:fremdsoftware), \
         vollständige Texte unter [Lizenztexte der Fremdsoftware](hilfe:drittanbieter).\n\n\
         ## Bausteine dieser Version\n\n\
         | Baustein | Version |\n|---|---|\n\
         | Abolos PDF | {version} |\n\
         | PDFium | {pdfium} |\n\
         | Qt (beim Bauen) | {qt} |\n\
         | Rust | {rust} |\n\n\
         ## Schutz in dieser Sitzung\n\n\
         | Maßnahme | Zustand |\n|---|---|\n\
         | Netzsperre (kein Netzwerkzugriff) | {netz} |\n\
         | Ausführsperre (keine Programme startbar) | {ausfuehren} |\n\n\
         Mehr dazu unter [Sicherheit und Datenschutz](hilfe:sicherheit).\n\n\
         ## Speicherorte\n\n\
         - Einstellungen (Tastenbelegung): `{konfig}`\n\
         - Daten (eigene Lesezeichen): `{daten}`\n",
        name = PROGRAMMNAME,
        version = env!("CARGO_PKG_VERSION"),
        autor = AUTOR,
        lizenz = LIZENZ,
        pdfium = env!("ABOLOS_PDFIUM_VERSION"),
        qt = env!("ABOLOS_QT_VERSION"),
        rust = env!("ABOLOS_RUST_VERSION"),
        konfig = ordner(crate::pfade::konfig("tasten.toml")),
        daten = ordner(crate::pfade::daten("lesezeichen.json")),
    )
}

/// Text für `abolos-pdf --help`
pub fn kommandozeile() -> String {
    format!(
        "{PROGRAMMNAME} {version} – Offline-PDF-Reader\n\n\
         Aufruf:\n  abolos-pdf [DATEI.pdf]\n  abolos-pdf --version\n  abolos-pdf --help\n\n\
         Copyright © 2026 {AUTOR}. Lizenz: GNU GPL, Version 3 oder später.\n\
         Ohne jede Gewährleistung. Hilfe im Programm: F1.",
        version = env!("CARGO_PKG_VERSION"),
    )
}

impl qobject::Hilfe {
    fn kapitel(&self) -> QString {
        let liste: Vec<_> =
            KAPITEL.iter().map(|(id, titel, art, _)| json!({"id": id, "titel": titel, "art": art})).collect();
        QString::from(&serde_json::Value::Array(liste).to_string())
    }

    fn inhalt(&self, id: &QString) -> QString {
        let id = String::from(id);
        if id == "ueber" {
            return QString::from(&ueber());
        }
        KAPITEL.iter().find(|k| k.0 == id).map(|k| QString::from(k.3)).unwrap_or_default()
    }

    fn info(&self) -> QString {
        let (netz, ausfuehren) = schutz_text();
        QString::from(
            &json!({
                "name": PROGRAMMNAME,
                "version": env!("CARGO_PKG_VERSION"),
                "autor": AUTOR,
                "lizenz": LIZENZ,
                "netzsperre": netz,
                "ausfuehrsperre": ausfuehren,
            })
            .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jedes_kapitel_hat_inhalt() {
        for (id, titel, art, text) in KAPITEL {
            assert!(!titel.is_empty());
            assert!(matches!(*art, "markdown" | "text" | "tasten"), "{id}: Art {art}");
            if !matches!(*id, "ueber" | "tasten") {
                assert!(text.len() > 200, "Kapitel {id} ist (fast) leer");
            }
        }
    }

    #[test]
    fn interne_verweise_zeigen_auf_vorhandene_kapitel() {
        let ids: Vec<&str> = KAPITEL.iter().map(|k| k.0).collect();
        let ueber = ueber();
        let texte = KAPITEL.iter().map(|k| k.3).chain(std::iter::once(ueber.as_str()));
        for text in texte {
            for teil in text.split("(hilfe:").skip(1) {
                let ziel = teil.split(')').next().unwrap_or("");
                assert!(ids.contains(&ziel), "Verweis auf unbekanntes Kapitel: hilfe:{ziel}");
            }
        }
    }

    #[test]
    fn ueber_enthaelt_gpl_hinweise() {
        let u = ueber();
        assert!(u.contains("Copyright © 2026 Oliver Weber"));
        assert!(u.contains("OHNE JEDE"));
        assert!(u.contains("Version 3"));
        assert!(u.contains(env!("CARGO_PKG_VERSION")));
    }
}
