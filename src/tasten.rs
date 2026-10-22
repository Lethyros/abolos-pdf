// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Frei belegbare Tasten als QML-Singleton `Tasten`.
//! Jede Aktion kann mehrere Tastenkombinationen haben; gespeichert in ~/.config/abolos-pdf/tasten.toml.
//! Tastentexte im Format von QKeySequence ("Ctrl+Shift+K", "PgDown", "F3").

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
        /// zählt bei jeder Änderung hoch; QML bindet die Shortcuts daran
        #[qproperty(i32, version)]
        type Tasten = super::TastenRust;

        /// JSON-Liste aller Aktionen: [{id, name, gruppe, tasten, standard}]
        #[qinvokable]
        fn aktionen(self: &Tasten) -> QString;

        /// `alt` durch `neu` ersetzen (alt = "" fügt hinzu). Gehört `neu` einer anderen Aktion,
        /// wird es dort entfernt; Rückgabe ist dann deren Name, sonst "".
        #[qinvokable]
        fn belegen(self: Pin<&mut Tasten>, aktion: &QString, alt: &QString, neu: &QString) -> QString;

        #[qinvokable]
        fn entfernen(self: Pin<&mut Tasten>, aktion: &QString, taste: &QString);

        /// "" = alle Aktionen
        #[qinvokable]
        fn zuruecksetzen(self: Pin<&mut Tasten>, aktion: &QString);

        /// Name der Aktion, die `taste` belegt, sonst ""
        #[qinvokable]
        fn belegt_von(self: &Tasten, taste: &QString) -> QString;

        #[qinvokable]
        fn datei(self: &Tasten) -> QString;

        /// Taste so, wie sie auf deutschen Tastaturen heißt ("Ctrl+PgDown" -> "Strg+Bild↓")
        #[qinvokable]
        fn anzeigename(self: &Tasten, taste: &QString) -> QString;
    }
}

use core::pin::Pin;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use cxx_qt_lib::QString;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// (id, Anzeigename, Gruppe, Standardtasten)
pub const AKTIONEN: &[(&str, &str, &str, &[&str])] = &[
    ("datei_oeffnen", "Datei öffnen", "Datei", &["Ctrl+O"]),
    ("beenden", "Beenden", "Datei", &["Ctrl+Q"]),
    ("bildschirm_runter", "Einen Bildschirm weiter", "Blättern", &["PgDown", "Space"]),
    ("bildschirm_hoch", "Einen Bildschirm zurück", "Blättern", &["PgUp", "Shift+Space"]),
    ("naechste_seite", "Nächste Seite", "Blättern", &["Ctrl+PgDown", "N"]),
    ("vorige_seite", "Vorige Seite", "Blättern", &["Ctrl+PgUp", "P"]),
    ("erste_seite", "Erste Seite", "Blättern", &["Home", "Ctrl+Home"]),
    ("letzte_seite", "Letzte Seite", "Blättern", &["End", "Ctrl+End"]),
    ("runter", "Ein Stück runter", "Blättern", &["Down", "J"]),
    ("hoch", "Ein Stück hoch", "Blättern", &["Up", "K"]),
    ("links", "Ein Stück nach links", "Blättern", &["Left", "H"]),
    ("rechts", "Ein Stück nach rechts", "Blättern", &["Right", "L"]),
    ("gehe_zu_seite", "Gehe zu Seite …", "Blättern", &["Ctrl+G"]),
    ("zurueck", "Zurück (nach Link-Sprung)", "Blättern", &["Alt+Left", "Backspace"]),
    ("vorwaerts", "Vorwärts", "Blättern", &["Alt+Right"]),
    ("zoom_rein", "Vergrößern", "Ansicht", &["Ctrl++", "Ctrl+="]),
    ("zoom_raus", "Verkleinern", "Ansicht", &["Ctrl+-"]),
    ("zoom_100", "Originalgröße (100 %)", "Ansicht", &["Ctrl+0"]),
    ("ganze_seite", "Ganze Seite", "Ansicht", &["Ctrl+1"]),
    ("seitenbreite", "Seitenbreite", "Ansicht", &["Ctrl+2"]),
    ("vollbild", "Vollbild", "Ansicht", &["F11"]),
    ("einzelseite", "Einzelseiten", "Ansicht", &["Ctrl+3"]),
    ("doppelseite", "Doppelseiten ein/aus", "Ansicht", &["Ctrl+4", "D"]),
    ("deckblatt", "Doppelseiten mit einzelnem Deckblatt", "Ansicht", &["Ctrl+5"]),
    ("drehen_rechts", "Alle Seiten im Uhrzeigersinn drehen", "Drehen", &["Ctrl+R"]),
    ("drehen_links", "Alle Seiten gegen den Uhrzeigersinn drehen", "Drehen", &["Ctrl+Shift+R"]),
    ("seite_drehen_rechts", "Nur diese Seite im Uhrzeigersinn drehen", "Drehen", &["Ctrl+Alt+R"]),
    ("seite_drehen_links", "Nur diese Seite gegen den Uhrzeigersinn drehen", "Drehen", &["Ctrl+Alt+Shift+R"]),
    ("suchen", "Suchen", "Suche", &["Ctrl+F", "/"]),
    ("naechster_treffer", "Nächster Treffer", "Suche", &["F3"]),
    ("voriger_treffer", "Voriger Treffer", "Suche", &["Shift+F3"]),
    ("suche_schliessen", "Suche schließen / Link-Markierung entfernen", "Suche", &["Esc"]),
    ("ordner_suchen", "In allen PDFs eines Ordners suchen …", "Suche", &["Ctrl+Shift+F"]),
    ("suche_gross_klein", "Groß-/Kleinschreibung beachten ein/aus", "Suche", &["Alt+C"]),
    ("suche_ganze_woerter", "Nur ganze Wörter ein/aus", "Suche", &["Alt+W"]),
    ("speichern", "Speichern", "Datei", &["Ctrl+S"]),
    ("speichern_unter", "Speichern unter …", "Datei", &["Ctrl+Shift+S"]),
    ("rueckgaengig", "Rückgängig (letzte Anmerkung)", "Bearbeiten", &["Ctrl+Z"]),
    ("werkzeug_hand", "Werkzeug: Hand / Auswahl", "Werkzeuge", &["Alt+1"]),
    ("werkzeug_hervorheben", "Werkzeug: Hervorheben", "Werkzeuge", &["Alt+2"]),
    ("werkzeug_unterstreichen", "Werkzeug: Unterstreichen", "Werkzeuge", &["Alt+3"]),
    ("werkzeug_durchstreichen", "Werkzeug: Durchstreichen", "Werkzeuge", &["Alt+4"]),
    ("werkzeug_freihand", "Werkzeug: Stift (Freihand)", "Werkzeuge", &["Alt+5"]),
    ("werkzeug_rechteck", "Werkzeug: Rechteck", "Werkzeuge", &["Alt+6"]),
    ("werkzeug_ellipse", "Werkzeug: Ellipse", "Werkzeuge", &["Alt+7"]),
    ("werkzeug_linie", "Werkzeug: Linie", "Werkzeuge", &["Alt+L"]),
    ("werkzeug_pfeil", "Werkzeug: Pfeil", "Werkzeuge", &["Alt+P"]),
    ("werkzeug_notiz", "Werkzeug: Notiz", "Werkzeuge", &["Alt+8"]),
    ("werkzeug_textfeld", "Werkzeug: Textfeld", "Werkzeuge", &["Alt+9"]),
    ("werkzeug_bild", "Werkzeug: Bild einfügen", "Werkzeuge", &["Alt+0"]),
    ("werkzeug_radierer", "Werkzeug: Anmerkung löschen", "Werkzeuge", &["Alt+E"]),
    ("seitenleiste", "Seitenleiste ein/aus", "Ansicht", &["F4"]),
    ("lesezeichen_setzen", "Lesezeichen hier setzen", "Lesezeichen", &["Ctrl+B"]),
    ("tastenbelegung", "Tastenbelegung …", "Einstellungen", &["Ctrl+,"]),
    ("hilfe", "Hilfe", "Hilfe", &["F1"]),
    ("ueber", "Über Abolos PDF", "Hilfe", &[]),
];

#[derive(Serialize, Deserialize, Default)]
struct Datei {
    #[serde(default)]
    tasten: BTreeMap<String, Vec<String>>,
}

struct Belegung {
    tasten: BTreeMap<String, Vec<String>>,
}

static BELEGUNG: Mutex<Option<Belegung>> = Mutex::new(None);

fn dateipfad() -> PathBuf {
    crate::pfade::konfig("tasten.toml")
}

fn standard(id: &str) -> Vec<String> {
    AKTIONEN.iter().find(|a| a.0 == id).map(|a| a.3.iter().map(|s| s.to_string()).collect()).unwrap_or_default()
}

fn gleich(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

/// Belegung laden (beim ersten Zugriff); fehlende Aktionen bekommen ihre Standardtasten
fn belegung() -> MutexGuard<'static, Option<Belegung>> {
    let mut b = BELEGUNG.lock().unwrap();
    if b.is_none() {
        let gespeichert: Datei = std::fs::read_to_string(dateipfad())
            .ok()
            .and_then(|text| toml::from_str(&text).map_err(|e| eprintln!("tasten.toml ungültig: {e}")).ok())
            .unwrap_or_default();
        let mut tasten = BTreeMap::new();
        for (id, ..) in AKTIONEN {
            let liste = gespeichert.tasten.get(*id).cloned().unwrap_or_else(|| standard(id));
            tasten.insert(id.to_string(), liste);
        }
        *b = Some(Belegung { tasten });
    }
    b
}

fn speichern(b: &Belegung) {
    let pfad = dateipfad();
    let text = format!(
        "# Abolos PDF: Tastenbelegung. Mehrere Tasten je Aktion möglich, [] = keine.\n\
         # Schreibweise wie \"Ctrl+Shift+K\", \"PgDown\", \"F3\".\n\n{}",
        toml::to_string(&Datei { tasten: b.tasten.clone() }).unwrap_or_default()
    );
    if let Some(dir) = pfad.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&pfad, text) {
        eprintln!("{}: {e}", pfad.display());
    }
}

fn name_von(id: &str) -> &'static str {
    AKTIONEN.iter().find(|a| a.0 == id).map(|a| a.1).unwrap_or("")
}

#[derive(Default)]
pub struct TastenRust {
    version: i32,
}

impl qobject::Tasten {
    fn geaendert(mut self: Pin<&mut Self>) {
        let v = *self.version() + 1;
        self.as_mut().set_version(v);
    }

    fn aktionen(&self) -> QString {
        let b = belegung();
        let b = b.as_ref().unwrap();
        let liste: Vec<_> = AKTIONEN
            .iter()
            .map(|(id, name, gruppe, std)| {
                json!({"id": id, "name": name, "gruppe": gruppe, "tasten": b.tasten.get(*id), "standard": std})
            })
            .collect();
        QString::from(&serde_json::Value::Array(liste).to_string())
    }

    fn belegen(self: Pin<&mut Self>, aktion: &QString, alt: &QString, neu: &QString) -> QString {
        let (aktion, alt, neu) = (String::from(aktion), String::from(alt), String::from(neu));
        if neu.trim().is_empty() || name_von(&aktion).is_empty() {
            return QString::default();
        }
        let mut vorbesitzer = String::new();
        {
            let mut g = belegung();
            let b = g.as_mut().unwrap();
            for (id, liste) in b.tasten.iter_mut() {
                if *id != aktion && liste.iter().any(|t| gleich(t, &neu)) {
                    liste.retain(|t| !gleich(t, &neu));
                    vorbesitzer = name_von(id).to_string();
                }
            }
            let liste = b.tasten.entry(aktion).or_default();
            match liste.iter().position(|t| gleich(t, &alt)).filter(|_| !alt.is_empty()) {
                Some(i) => liste[i] = neu.clone(),
                None if !liste.iter().any(|t| gleich(t, &neu)) => liste.push(neu.clone()),
                None => {}
            }
            speichern(b);
        }
        self.geaendert();
        QString::from(&vorbesitzer)
    }

    fn entfernen(self: Pin<&mut Self>, aktion: &QString, taste: &QString) {
        let (aktion, taste) = (String::from(aktion), String::from(taste));
        {
            let mut g = belegung();
            let b = g.as_mut().unwrap();
            if let Some(liste) = b.tasten.get_mut(&aktion) {
                liste.retain(|t| !gleich(t, &taste));
            }
            speichern(b);
        }
        self.geaendert();
    }

    fn zuruecksetzen(self: Pin<&mut Self>, aktion: &QString) {
        let aktion = String::from(aktion);
        {
            let mut g = belegung();
            let b = g.as_mut().unwrap();
            for (id, ..) in AKTIONEN {
                if aktion.is_empty() || aktion == *id {
                    b.tasten.insert(id.to_string(), standard(id));
                }
            }
            speichern(b);
        }
        self.geaendert();
    }

    fn belegt_von(&self, taste: &QString) -> QString {
        let taste = String::from(taste);
        let g = belegung();
        let b = g.as_ref().unwrap();
        b.tasten
            .iter()
            .find(|(_, liste)| liste.iter().any(|t| gleich(t, &taste)))
            .map(|(id, _)| QString::from(name_von(id)))
            .unwrap_or_default()
    }

    fn datei(&self) -> QString {
        QString::from(&dateipfad().to_string_lossy().into_owned())
    }

    fn anzeigename(&self, taste: &QString) -> QString {
        QString::from(&anzeigename(&String::from(taste)))
    }
}

/// Tastenname für Anzeige und Hilfe; gespeichert bleibt die Schreibweise von QKeySequence
pub fn anzeigename(taste: &str) -> String {
    let mut teile: Vec<&str> = taste.split('+').collect();
    // "Ctrl++": die Taste selbst ist das Plus
    if taste.ends_with("++") || taste == "+" {
        teile.truncate(teile.len().saturating_sub(2));
        teile.push("+");
    }
    teile
        .iter()
        .map(|t| match *t {
            "Ctrl" => "Strg",
            "PgDown" => "Bild↓",
            "PgUp" => "Bild↑",
            "Home" => "Pos1",
            "End" => "Ende",
            "Space" => "Leertaste",
            "Backspace" => "Rücktaste",
            "Del" | "Delete" => "Entf",
            "Ins" | "Insert" => "Einfg",
            "Return" | "Enter" => "Eingabe",
            "Left" => "←",
            "Right" => "→",
            "Up" => "↑",
            "Down" => "↓",
            andere => andere,
        })
        .collect::<Vec<_>>()
        .join("+")
}

#[cfg(test)]
mod tests {
    use super::anzeigename;

    #[test]
    fn tastennamen_auf_deutsch() {
        assert_eq!(anzeigename("Ctrl+O"), "Strg+O");
        assert_eq!(anzeigename("Ctrl+PgDown"), "Strg+Bild↓");
        assert_eq!(anzeigename("Ctrl++"), "Strg++");
        assert_eq!(anzeigename("Ctrl+-"), "Strg+-");
        assert_eq!(anzeigename("Shift+Space"), "Shift+Leertaste");
        assert_eq!(anzeigename("Alt+Left"), "Alt+←");
        assert_eq!(anzeigename("F3"), "F3");
        assert_eq!(anzeigename("+"), "+");
    }
}
