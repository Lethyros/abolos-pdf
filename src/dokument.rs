// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Das geöffnete Dokument als QML-Singleton `Dokument`: Öffnen, Layout, Suche, Links.
//! Zusammengesetzte Antworten gehen als JSON-Text an QML (dort `JSON.parse`).

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
        #[qproperty(i32, seitenzahl)]
        #[qproperty(QString, pfad)]
        #[qproperty(QString, titel)]
        #[qproperty(QString, fehler)]
        #[qproperty(i32, generation)]
        #[qproperty(i32, treffer_anzahl)]
        #[qproperty(bool, suche_laeuft)]
        #[qproperty(f64, suche_fortschritt)]
        #[qproperty(i32, aktueller_treffer)]
        #[qproperty(i32, links_version)]
        /// zählt bei jeder Änderung der Seitenanordnung hoch (Modus, Drehung, neues Dokument)
        #[qproperty(i32, layout_version)]
        /// 0 = Einzelseiten, 1 = Doppelseiten, 2 = Doppelseiten mit einzelnem Deckblatt
        #[qproperty(i32, modus)]
        /// Drehung aller Seiten im Uhrzeigersinn (0, 90, 180, 270)
        #[qproperty(i32, drehung)]
        /// ungespeicherte Änderungen vorhanden
        #[qproperty(bool, geaendert)]
        /// zählt hoch, wenn sich Anmerkungen ändern (Liste neu laden)
        #[qproperty(i32, anmerkungen_version)]
        #[qproperty(i32, lesezeichen_version)]
        /// Ordnersuche: zählt bei jeder neuen Suche hoch (Ergebnisliste leeren)
        #[qproperty(i32, ordner_generation)]
        #[qproperty(i32, ordner_anzahl)]
        #[qproperty(bool, ordner_laeuft)]
        #[qproperty(QString, ordner_status)]
        /// 0..1 (Anteil durchsuchter Dateien)
        #[qproperty(f64, ordner_fortschritt)]
        type Dokument = super::DokumentRust;

        /// Pfad oder file://-URL
        #[qinvokable]
        fn oeffnen(self: Pin<&mut Dokument>, pfad: &QString) -> bool;

        /// Datei von der Kommandozeile
        #[qinvokable]
        fn start_pfad(self: &Dokument) -> QString;

        /// Pfad und URL umrechnen (Dateidialoge); unter Windows mit Laufwerk und "\\"
        #[qinvokable]
        fn datei_url(self: &Dokument, pfad: &QString) -> QString;
        #[qinvokable]
        fn lokaler_pfad(self: &Dokument, url: &QString) -> QString;
        /// übergeordneter Ordner bzw. Dateiname eines Pfads ("" wenn keiner)
        #[qinvokable]
        fn ordner_von(self: &Dokument, pfad: &QString) -> QString;
        #[qinvokable]
        fn dateiname(self: &Dokument, pfad: &QString) -> QString;

        /// "" = keine Warnung nötig, sonst der Grund, warum die Netzsperre nicht greift
        #[qinvokable]
        fn netzsperre(self: &Dokument) -> QString;
        /// "" = keine Warnung nötig, sonst der Grund, warum die Ausführsperre nicht greift
        #[qinvokable]
        fn ausfuehrsperre(self: &Dokument) -> QString;

        /// Nur ABOLOS_*-Variablen (Test- und Screenshot-Modus)
        #[qinvokable]
        fn umgebung(self: &Dokument, name: &QString) -> QString;

        #[qinvokable]
        fn setze_modus(self: Pin<&mut Dokument>, modus: i32);
        /// Um `grad` weiterdrehen; seite < 0 = alle Seiten, sonst nur diese
        #[qinvokable]
        fn drehen(self: Pin<&mut Dokument>, grad: i32, seite: i32);
        #[qinvokable]
        fn seiten_drehung(self: &Dokument, seite: i32) -> i32;

        /// angezeigte Größe (nach Drehung) in Punkten
        #[qinvokable]
        fn seiten_breite(self: &Dokument, seite: i32) -> f64;
        #[qinvokable]
        fn seiten_hoehe(self: &Dokument, seite: i32) -> f64;
        /// Oberkante in Punkten bei Zoom 1 (inkl. Seitenabstand)
        #[qinvokable]
        fn seiten_oben(self: &Dokument, seite: i32) -> f64;
        #[qinvokable]
        fn gesamt_hoehe(self: &Dokument) -> f64;
        #[qinvokable]
        fn max_breite(self: &Dokument) -> f64;
        #[qinvokable]
        fn seitenabstand(self: &Dokument) -> f64;
        /// linke Kante relativ zur Mittellinie der Ansicht
        #[qinvokable]
        fn seiten_x(self: &Dokument, seite: i32) -> f64;
        /// erste Seite der Zeile an Höhe y
        #[qinvokable]
        fn seite_bei(self: &Dokument, y: f64) -> i32;
        /// letzte Seite der Zeile an Höhe y
        #[qinvokable]
        fn letzte_seite_bei(self: &Dokument, y: f64) -> i32;
        /// erste Seite der nächsten/vorigen Zeile (Doppelseiten: zwei weiter)
        #[qinvokable]
        fn naechste_zeile(self: &Dokument, seite: i32) -> i32;
        #[qinvokable]
        fn vorige_zeile(self: &Dokument, seite: i32) -> i32;
        /// Breite/Höhe der Zeile, die `seite` enthält (für Seitenbreite/ganze Seite)
        #[qinvokable]
        fn zeilen_breite(self: &Dokument, seite: i32) -> f64;
        #[qinvokable]
        fn zeilen_hoehe(self: &Dokument, seite: i32) -> f64;
        /// relative Höhe y (ungedrehte Seite) in der Anzeige; -1 wenn bei 90/270 Grad nicht sinnvoll
        #[qinvokable]
        fn anzeige_y(self: &Dokument, seite: i32, y: f64) -> f64;

        #[qinvokable]
        fn suchen(self: Pin<&mut Dokument>, text: &QString, gross_klein: bool, ganze_woerter: bool, ab_seite: i32);
        #[qinvokable]
        fn suche_abbrechen(self: Pin<&mut Dokument>);
        /// richtung +1/-1; ohne ausgewählten Treffer zählt die Position (ab_seite, ab_y)
        #[qinvokable]
        fn treffer_springen(self: Pin<&mut Dokument>, richtung: i32, ab_seite: i32, ab_y: f64) -> QString;

        /// Treffer Nummer `nr` (0-basiert) der Trefferliste: JSON {seite, vor, wort, nach} oder ""
        #[qinvokable]
        fn treffer_eintrag(self: &Dokument, nr: i32) -> QString;
        /// Treffer auswählen (Klick in der Liste); Antwort wie treffer_springen
        #[qinvokable]
        fn treffer_waehlen(self: Pin<&mut Dokument>, nr: i32) -> QString;
        /// Rechteck (Anzeige) des `nr`-ten Treffers von `text` auf `seite` im geöffneten Dokument, JSON oder ""
        #[qinvokable]
        fn treffer_rechteck(
            self: &Dokument,
            seite: i32,
            text: &QString,
            gross_klein: bool,
            ganze_woerter: bool,
            nr: i32,
        ) -> QString;

        /// Alle PDFs eines Ordners durchsuchen (läuft im Hintergrund, Ergebnisse über ordner_anzahl)
        #[qinvokable]
        fn ordner_suchen(
            self: Pin<&mut Dokument>,
            ordner: &QString,
            text: &QString,
            unterordner: bool,
            gross_klein: bool,
            ganze_woerter: bool,
        );
        #[qinvokable]
        fn ordner_abbrechen(self: Pin<&mut Dokument>);
        /// Ordner-Treffer von..bis (ohne bis) als JSON-Liste
        #[qinvokable]
        fn ordner_treffer(self: &Dokument, von: i32, bis: i32) -> QString;

        #[qinvokable]
        fn links_anfordern(self: Pin<&mut Dokument>, seite: i32);
        /// Link unter der relativen Position (0..1) einer Seite, als JSON oder ""
        #[qinvokable]
        fn link_bei(self: &Dokument, seite: i32, x: f64, y: f64) -> QString;

        /// Alle bekannten Links einer Seite als JSON-Liste (für Tests und Übersichten)
        #[qinvokable]
        fn links_json(self: &Dokument, seite: i32) -> QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// Text zwischen zwei Punkten (relativ, Anzeige): JSON {rects: [...], text}
        #[qinvokable]
        fn text_auswahl(self: &Dokument, seite: i32, x1: f64, y1: f64, x2: f64, y2: f64) -> QString;
        /// Anmerkung anlegen; JSON mit "art" und Punkten in Anzeige-Koordinaten. Rückgabe: Fehler oder ""
        #[qinvokable]
        fn anlegen(self: Pin<&mut Dokument>, auftrag: &QString) -> QString;
        #[qinvokable]
        fn loeschen(self: Pin<&mut Dokument>, seite: i32, index: i32) -> QString;
        #[qinvokable]
        fn rueckgaengig(self: Pin<&mut Dokument>) -> QString;
        #[qinvokable]
        fn inhalt_setzen(self: Pin<&mut Dokument>, seite: i32, index: i32, text: &QString) -> QString;
        /// alle Anmerkungen als JSON-Liste
        #[qinvokable]
        fn anmerkungen_json(self: &Dokument) -> QString;
        /// Anmerkung unter einem Punkt (relativ, Anzeige) als JSON oder ""
        #[qinvokable]
        fn anmerkung_bei(self: &Dokument, seite: i32, x: f64, y: f64) -> QString;
        /// Gliederung (Inhaltsverzeichnis) des PDFs als JSON-Liste
        #[qinvokable]
        fn gliederung_json(self: &Dokument) -> QString;
        /// "" = in die geöffnete Datei; Rückgabe: Fehler oder ""
        #[qinvokable]
        fn speichern(self: Pin<&mut Dokument>, pfad: &QString) -> QString;

        #[qinvokable]
        fn lesezeichen_json(self: &Dokument) -> QString;
        #[qinvokable]
        fn lesezeichen_setzen(self: Pin<&mut Dokument>, seite: i32, y: f64, name: &QString);
        #[qinvokable]
        fn lesezeichen_entfernen(self: Pin<&mut Dokument>, nr: i32);
        #[qinvokable]
        fn lesezeichen_umbenennen(self: Pin<&mut Dokument>, nr: i32, name: &QString);
    }

    impl cxx_qt::Threading for Dokument {}
}

use core::pin::Pin;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;

use cxx_qt::Threading;
use cxx_qt_lib::{QString, QUrl};
use serde_json::json;

use crate::engine::{self, LinkZiel, OrdnerAuftrag, SuchAuftrag};
use crate::suche::Suchoptionen;
use crate::zustand::{self, ANSICHT, Ansicht, LAYOUT, LINKS, Layout, Modus, ORDNER, SEITENABSTAND, SUCHE};
use crate::{bearbeiten, lesezeichen};

pub static START_PFAD: OnceLock<String> = OnceLock::new();

/// Zustand der Schutzmaßnahmen beim Start (siehe schutz.rs)
pub static SCHUTZ: OnceLock<crate::schutz::Schutz> = OnceLock::new();

#[derive(Default)]
pub struct DokumentRust {
    seitenzahl: i32,
    pfad: QString,
    titel: QString,
    fehler: QString,
    generation: i32,
    treffer_anzahl: i32,
    suche_laeuft: bool,
    suche_fortschritt: f64,
    aktueller_treffer: i32,
    links_version: i32,
    layout_version: i32,
    modus: i32,
    drehung: i32,
    geaendert: bool,
    anmerkungen_version: i32,
    lesezeichen_version: i32,
    ordner_generation: i32,
    ordner_anzahl: i32,
    ordner_laeuft: bool,
    ordner_status: QString,
    ordner_fortschritt: f64,
}

/// Pfad aus einem Pfad oder einer file://-URL (Dateidialoge liefern URLs)
fn als_pfad(text: &QString) -> PathBuf {
    let s = String::from(text);
    if s.starts_with("file:")
        && let Some(lokal) = QUrl::from(text).to_local_file()
    {
        return systempfad(String::from(&lokal));
    }
    systempfad(s)
}

/// Qt liefert auch unter Windows "/"; für Anzeige und Vergleiche gilt dort "\\"
fn systempfad(s: String) -> PathBuf {
    #[cfg(windows)]
    let s = s.replace('/', "\\");
    PathBuf::from(s)
}

fn als_text(pfad: &std::path::Path) -> QString {
    QString::from(&pfad.to_string_lossy().into_owned())
}

impl qobject::Dokument {
    fn oeffnen(mut self: Pin<&mut Self>, pfad: &QString) -> bool {
        let pfad = als_pfad(pfad);
        let engine = match engine::engine() {
            Ok(e) => e,
            Err(e) => {
                self.as_mut().set_fehler(QString::from(&e));
                return false;
            }
        };
        SUCHE.lock().unwrap().leeren();
        engine.suche_abbrechen();
        let ergebnis = engine.oeffnen(&pfad);
        let (nr, masse) = match ergebnis {
            Ok(x) => x,
            Err(e) => {
                *LAYOUT.write().unwrap() = Layout::default();
                self.as_mut().set_seitenzahl(0);
                self.as_mut().set_fehler(QString::from(&format!("Konnte nicht geöffnet werden: {e}")));
                let g = *self.generation() + 1;
                self.as_mut().set_generation(g);
                return false;
            }
        };
        let anzahl = masse.len() as i32;
        ANSICHT.write().unwrap().get_or_insert_with(Ansicht::default).seiten_drehung.clear();
        *LAYOUT.write().unwrap() = Layout::neu(masse);
        *LINKS.lock().unwrap() = Some(HashMap::new());
        zustand::DOK_NR.store(nr, Ordering::Release);

        let titel = pfad.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        self.as_mut().set_fehler(QString::default());
        self.as_mut().set_pfad(QString::from(&pfad.to_string_lossy().into_owned()));
        self.as_mut().set_titel(QString::from(&titel));
        self.as_mut().set_treffer_anzahl(0);
        self.as_mut().set_aktueller_treffer(-1);
        self.as_mut().set_suche_laeuft(false);
        self.as_mut().set_seitenzahl(anzahl);
        self.as_mut().set_geaendert(false);
        *zustand::ANMERKUNGEN.lock().unwrap() = None;
        let a = *self.anmerkungen_version() + 1;
        self.as_mut().set_anmerkungen_version(a);
        let l = *self.lesezeichen_version() + 1;
        self.as_mut().set_lesezeichen_version(l);
        let g = *self.generation() + 1;
        self.as_mut().set_generation(g);
        let v = *self.layout_version() + 1;
        self.as_mut().set_layout_version(v);
        true
    }

    fn start_pfad(&self) -> QString {
        START_PFAD.get().map(QString::from).unwrap_or_default()
    }

    fn datei_url(&self, pfad: &QString) -> QString {
        if pfad.is_empty() {
            return QString::default();
        }
        QUrl::from_local_file(pfad).to_qstring()
    }

    fn lokaler_pfad(&self, url: &QString) -> QString {
        als_text(&als_pfad(url))
    }

    fn ordner_von(&self, pfad: &QString) -> QString {
        als_pfad(pfad).parent().map(als_text).unwrap_or_default()
    }

    fn dateiname(&self, pfad: &QString) -> QString {
        als_pfad(pfad).file_name().map(|n| QString::from(&n.to_string_lossy().into_owned())).unwrap_or_default()
    }

    fn netzsperre(&self) -> QString {
        SCHUTZ.get().map(|s| QString::from(s.netz.warnung())).unwrap_or_else(|| QString::from("nicht eingerichtet"))
    }

    fn ausfuehrsperre(&self) -> QString {
        SCHUTZ
            .get()
            .map(|s| QString::from(s.ausfuehren.warnung()))
            .unwrap_or_else(|| QString::from("nicht eingerichtet"))
    }

    fn umgebung(&self, name: &QString) -> QString {
        let name = String::from(name);
        if !name.starts_with("ABOLOS_") {
            return QString::default();
        }
        std::env::var(&name).map(|v| QString::from(&v)).unwrap_or_default()
    }

    fn setze_modus(mut self: Pin<&mut Self>, modus: i32) {
        ANSICHT.write().unwrap().get_or_insert_with(Ansicht::default).modus = Modus::aus_zahl(modus);
        self.as_mut().set_modus(Modus::aus_zahl(modus).zahl());
        self.neu_anordnen();
    }

    fn drehen(mut self: Pin<&mut Self>, grad: i32, seite: i32) {
        let gesamt = {
            let mut a = ANSICHT.write().unwrap();
            let a = a.get_or_insert_with(Ansicht::default);
            if seite < 0 {
                a.drehung = (a.drehung + grad).rem_euclid(360);
            } else {
                let d = a.seiten_drehung.entry(seite).or_insert(0);
                *d = (*d + grad).rem_euclid(360);
            }
            a.drehung
        };
        self.as_mut().set_drehung(gesamt);
        self.neu_anordnen();
    }

    fn neu_anordnen(mut self: Pin<&mut Self>) {
        LAYOUT.write().unwrap().berechnen();
        let v = *self.layout_version() + 1;
        self.as_mut().set_layout_version(v);
    }

    fn seiten_drehung(&self, seite: i32) -> i32 {
        zustand::drehung_von(seite)
    }

    fn seiten_breite(&self, seite: i32) -> f64 {
        LAYOUT.read().unwrap().anzeige.get(seite as usize).map(|a| a.0).unwrap_or(595.0)
    }

    fn seiten_hoehe(&self, seite: i32) -> f64 {
        LAYOUT.read().unwrap().anzeige.get(seite as usize).map(|a| a.1).unwrap_or(842.0)
    }

    fn seiten_x(&self, seite: i32) -> f64 {
        LAYOUT.read().unwrap().x.get(seite.max(0) as usize).copied().unwrap_or(0.0)
    }

    fn letzte_seite_bei(&self, y: f64) -> i32 {
        LAYOUT.read().unwrap().letzte_seite_bei(y)
    }

    fn naechste_zeile(&self, seite: i32) -> i32 {
        let l = LAYOUT.read().unwrap();
        match l.zeile_von(seite) {
            Some(z) if z + 1 < l.zeilen.len() => l.zeilen[z + 1].0 as i32,
            Some(z) => l.zeilen[z].0 as i32,
            None => 0,
        }
    }

    fn vorige_zeile(&self, seite: i32) -> i32 {
        let l = LAYOUT.read().unwrap();
        match l.zeile_von(seite) {
            Some(z) if z > 0 => l.zeilen[z - 1].0 as i32,
            _ => 0,
        }
    }

    fn zeilen_breite(&self, seite: i32) -> f64 {
        let l = LAYOUT.read().unwrap();
        let Some(z) = l.zeile_von(seite) else { return 595.0 };
        let (a, b, ..) = l.zeilen[z];
        if a == b { l.anzeige[a].0 } else { 2.0 * l.anzeige[a].0.max(l.anzeige[b].0) + SEITENABSTAND }
    }

    fn zeilen_hoehe(&self, seite: i32) -> f64 {
        let l = LAYOUT.read().unwrap();
        l.zeile_von(seite).map(|z| l.zeilen[z].3).unwrap_or(842.0)
    }

    fn anzeige_y(&self, seite: i32, y: f64) -> f64 {
        match zustand::drehung_von(seite) {
            0 => y,
            180 => 1.0 - y,
            _ => -1.0,
        }
    }

    fn seiten_oben(&self, seite: i32) -> f64 {
        let l = LAYOUT.read().unwrap();
        l.oben.get(seite.max(0) as usize).copied().unwrap_or(0.0)
    }

    fn gesamt_hoehe(&self) -> f64 {
        LAYOUT.read().unwrap().gesamt
    }

    fn max_breite(&self) -> f64 {
        LAYOUT.read().unwrap().max_breite
    }

    fn seitenabstand(&self) -> f64 {
        SEITENABSTAND
    }

    fn seite_bei(&self, y: f64) -> i32 {
        LAYOUT.read().unwrap().seite_bei(y)
    }

    fn suchen(mut self: Pin<&mut Self>, text: &QString, gross_klein: bool, ganze_woerter: bool, ab_seite: i32) {
        let text = String::from(text);
        let such_nr = SUCHE.lock().unwrap().leeren();
        self.as_mut().set_treffer_anzahl(0);
        self.as_mut().set_aktueller_treffer(-1);
        self.as_mut().set_suche_fortschritt(0.0);
        let Ok(engine) = engine::engine() else { return };
        if text.trim().is_empty() || *self.seitenzahl() == 0 {
            engine.suche_abbrechen();
            self.as_mut().set_suche_laeuft(false);
            return;
        }
        self.as_mut().set_suche_laeuft(true);
        let (t1, t2, t3) = (self.qt_thread(), self.qt_thread(), self.qt_thread());
        let aktuell = move || SUCHE.lock().unwrap().generation == such_nr;
        engine.suchen(SuchAuftrag {
            text,
            optionen: Suchoptionen { gross_klein, ganze_woerter },
            ab_seite,
            bei_treffern: Box::new(move |treffer| {
                let (anzahl, aktuell) = {
                    let mut s = SUCHE.lock().unwrap();
                    if s.generation != such_nr {
                        return;
                    }
                    s.einfuegen(treffer);
                    (s.treffer.len() as i32, s.aktuell as i32)
                };
                // aktuell kann sich verschieben, wenn davor Treffer dazukommen
                let _ = t1.queue(move |mut d| {
                    d.as_mut().set_aktueller_treffer(aktuell);
                    d.as_mut().set_treffer_anzahl(anzahl);
                });
            }),
            fortschritt: Box::new(move |f| {
                if aktuell() {
                    let _ = t2.queue(move |d| d.set_suche_fortschritt(f as f64));
                }
            }),
            fertig: Box::new(move |()| {
                let _ = t3.queue(move |d| {
                    if SUCHE.lock().unwrap().generation == such_nr {
                        d.set_suche_laeuft(false);
                    }
                });
            }),
        });
    }

    fn suche_abbrechen(mut self: Pin<&mut Self>) {
        SUCHE.lock().unwrap().leeren();
        if let Ok(engine) = engine::engine() {
            engine.suche_abbrechen();
        }
        self.as_mut().set_suche_laeuft(false);
        self.as_mut().set_treffer_anzahl(0);
        self.as_mut().set_aktueller_treffer(-1);
    }

    fn treffer_springen(mut self: Pin<&mut Self>, richtung: i32, ab_seite: i32, ab_y: f64) -> QString {
        let (idx, antwort) = {
            let mut s = SUCHE.lock().unwrap();
            let n = s.treffer.len() as i64;
            if n == 0 {
                return QString::default();
            }
            let idx = if s.aktuell >= 0 {
                (s.aktuell + richtung.signum() as i64).rem_euclid(n)
            } else {
                let lage = |t: &engine::Treffer| (t.seite, t.rects.first().map(|r| r.y as f64).unwrap_or(0.0));
                // ab_y kommt aus der gedrehten Anzeige; Treffer sind ungedreht gespeichert
                let ab_y = match zustand::drehung_von(ab_seite) {
                    0 => ab_y,
                    180 => 1.0 - ab_y,
                    _ => 0.0,
                };
                let ab = (ab_seite, ab_y);
                if richtung >= 0 {
                    s.treffer.iter().position(|t| lage(t) >= ab).map(|i| i as i64).unwrap_or(0)
                } else {
                    s.treffer.iter().rposition(|t| lage(t) < ab).map(|i| i as i64).unwrap_or(n - 1)
                }
            };
            s.aktuell = idx;
            (idx, sprung_json(&s, idx))
        };
        self.as_mut().set_aktueller_treffer(idx as i32);
        QString::from(&antwort.to_string())
    }

    fn treffer_eintrag(&self, nr: i32) -> QString {
        let s = SUCHE.lock().unwrap();
        let Some(t) = s.treffer.get(nr.max(0) as usize) else { return QString::default() };
        QString::from(&json!({"seite": t.seite, "vor": t.vor, "wort": t.wort, "nach": t.nach}).to_string())
    }

    fn treffer_waehlen(mut self: Pin<&mut Self>, nr: i32) -> QString {
        let antwort = {
            let mut s = SUCHE.lock().unwrap();
            if nr < 0 || nr as usize >= s.treffer.len() {
                return QString::default();
            }
            s.aktuell = nr as i64;
            sprung_json(&s, nr as i64)
        };
        self.as_mut().set_aktueller_treffer(nr);
        QString::from(&antwort.to_string())
    }

    fn treffer_rechteck(&self, seite: i32, text: &QString, gross_klein: bool, ganze_woerter: bool, nr: i32) -> QString {
        let optionen = Suchoptionen { gross_klein, ganze_woerter };
        let nadel = crate::suche::nadel(&String::from(text));
        let Ok(engine) = engine::engine() else { return QString::default() };
        let rects = engine
            .ausfuehren(move |stand| {
                stand
                    .dok
                    .as_ref()
                    .and_then(|d| {
                        engine::seite_durchsuchen(d, seite, &nadel, optionen).into_iter().nth(nr.max(0) as usize)
                    })
                    .map(|t| t.rects)
            })
            .ok()
            .flatten();
        let Some(rects) = rects.filter(|r| !r.is_empty()) else { return QString::default() };
        // mehrzeilige Treffer: umschließendes Rechteck
        let (x0, y0) = rects.iter().fold((f32::MAX, f32::MAX), |(x, y), r| (x.min(r.x), y.min(r.y)));
        let (x1, y1) = rects.iter().fold((f32::MIN, f32::MIN), |(x, y), r| (x.max(r.x + r.b), y.max(r.y + r.h)));
        let r = zustand::gedreht(engine::RelRect { x: x0, y: y0, b: x1 - x0, h: y1 - y0 }, zustand::drehung_von(seite));
        QString::from(&rect_json(r).to_string())
    }

    fn ordner_suchen(
        mut self: Pin<&mut Self>,
        ordner: &QString,
        text: &QString,
        unterordner: bool,
        gross_klein: bool,
        ganze_woerter: bool,
    ) {
        let optionen = Suchoptionen { gross_klein, ganze_woerter };
        let wurzel = als_pfad(ordner);
        let text = String::from(text);
        let generation = {
            let mut o = ORDNER.lock().unwrap();
            o.generation += 1;
            o.treffer.clear();
            o.dateien = 0;
            o.unlesbar = 0;
            o.wurzel = wurzel.clone();
            o.text = text.clone();
            o.optionen = optionen;
            o.generation
        };
        let Ok(engine) = engine::engine() else { return };
        engine.ordner_abbrechen();
        self.as_mut().set_ordner_generation(generation as i32);
        self.as_mut().set_ordner_anzahl(0);
        if text.trim().is_empty() {
            self.as_mut().set_ordner_laeuft(false);
            self.as_mut().set_ordner_status(QString::default());
            return;
        }
        self.as_mut().set_ordner_laeuft(true);
        self.as_mut().set_ordner_fortschritt(0.0);
        self.as_mut().set_ordner_status(QString::from("PDF-Dateien werden gesucht …"));
        let aktuell = move || ORDNER.lock().unwrap().generation == generation;
        let (t0, t1, t2, t3) = (self.qt_thread(), self.qt_thread(), self.qt_thread(), self.qt_thread());
        // Dateien im eigenen Thread zusammensuchen: große Ordnerbäume dürfen die Oberfläche nicht aufhalten
        std::thread::spawn(move || {
            let dateien = pdfs_finden(&wurzel, unterordner, &aktuell);
            if !aktuell() {
                return;
            }
            if dateien.is_empty() {
                let meldung = format!("Keine PDF-Dateien in {}", wurzel.display());
                let _ = t0.queue(move |mut d| {
                    d.as_mut().set_ordner_laeuft(false);
                    d.as_mut().set_ordner_status(QString::from(&meldung));
                });
                return;
            }
            let gesamt = dateien.len();
            engine.ordner_suchen(OrdnerAuftrag {
                dateien,
                text,
                optionen,
                grenze: ORDNER_GRENZE,
                bei_treffern: Box::new(move |neu| {
                    let anzahl = {
                        let mut o = ORDNER.lock().unwrap();
                        if o.generation != generation {
                            return;
                        }
                        for t in neu {
                            if o.treffer.last().is_none_or(|l| l.datei != t.datei) {
                                o.dateien += 1;
                            }
                            o.treffer.push(t);
                        }
                        o.treffer.len() as i32
                    };
                    let _ = t1.queue(move |d| d.set_ordner_anzahl(anzahl));
                }),
                fortschritt: Box::new(move |f| {
                    if !aktuell() {
                        return;
                    }
                    let (anzahl, mit) = {
                        let mut o = ORDNER.lock().unwrap();
                        o.unlesbar = f.unlesbar;
                        (o.treffer.len(), o.dateien)
                    };
                    let mut text = format!(
                        "Datei {} von {}: {} · {} Treffer in {} Dateien",
                        f.fertig, f.gesamt, f.datei, anzahl, mit
                    );
                    if f.unlesbar > 0 {
                        text += &format!(" · {} nicht lesbar", f.unlesbar);
                    }
                    let anteil = f.fertig as f64 / f.gesamt.max(1) as f64;
                    let _ = t2.queue(move |mut d| {
                        d.as_mut().set_ordner_status(QString::from(&text));
                        d.as_mut().set_ordner_fortschritt(anteil);
                    });
                }),
                fertig: Box::new(move |voll| {
                    if !aktuell() {
                        return;
                    }
                    let (anzahl, mit, unlesbar) = {
                        let o = ORDNER.lock().unwrap();
                        (o.treffer.len(), o.dateien, o.unlesbar)
                    };
                    let mut text = format!("{anzahl} Treffer in {mit} von {gesamt} PDF-Dateien");
                    if unlesbar > 0 {
                        text += &format!(" · {unlesbar} nicht lesbar (beschädigt oder mit Passwort)");
                    }
                    if voll {
                        text += &format!(" (Suche nach {ORDNER_GRENZE} Treffern beendet)");
                    }
                    let _ = t3.queue(move |mut d| {
                        d.as_mut().set_ordner_laeuft(false);
                        d.as_mut().set_ordner_status(QString::from(&text));
                    });
                }),
            });
        });
    }

    fn ordner_abbrechen(mut self: Pin<&mut Self>) {
        ORDNER.lock().unwrap().generation += 1; // laufende Rückmeldungen verfallen
        if let Ok(engine) = engine::engine() {
            engine.ordner_abbrechen();
        }
        if *self.ordner_laeuft() {
            let anzahl = ORDNER.lock().unwrap().treffer.len();
            self.as_mut().set_ordner_laeuft(false);
            self.as_mut().set_ordner_status(QString::from(&format!("Abgebrochen · {anzahl} Treffer")));
        }
    }

    fn ordner_treffer(&self, von: i32, bis: i32) -> QString {
        let o = ORDNER.lock().unwrap();
        let von = (von.max(0) as usize).min(o.treffer.len());
        let bis = (bis.max(0) as usize).clamp(von, o.treffer.len());
        let liste: Vec<_> = o.treffer[von..bis]
            .iter()
            .map(|t| {
                let name = t.datei.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                let ordner = t
                    .datei
                    .parent()
                    .map(|p| p.strip_prefix(&o.wurzel).unwrap_or(p).to_string_lossy().into_owned())
                    .unwrap_or_default();
                json!({"datei": t.datei.to_string_lossy(), "name": name, "ordner": ordner, "seite": t.seite, "nr": t.nr,
                       "vor": t.vor, "wort": t.wort, "nach": t.nach})
            })
            .collect();
        QString::from(&serde_json::Value::Array(liste).to_string())
    }

    fn links_anfordern(self: Pin<&mut Self>, seite: i32) {
        let dok = zustand::dok_nr();
        {
            let mut links = LINKS.lock().unwrap();
            let Some(map) = links.as_mut() else { return };
            if map.contains_key(&(dok, seite)) {
                return; // schon da oder unterwegs
            }
            map.insert((dok, seite), Vec::new());
        }
        let Ok(engine) = engine::engine() else { return };
        let thread = self.qt_thread();
        engine.links(
            dok,
            seite,
            Box::new(move |gefunden| {
                if let Some(map) = LINKS.lock().unwrap().as_mut() {
                    map.insert((dok, seite), gefunden);
                }
                let _ = thread.queue(|mut d| {
                    let v = *d.links_version() + 1;
                    d.as_mut().set_links_version(v);
                });
            }),
        );
    }

    fn links_json(&self, seite: i32) -> QString {
        let links = zustand::links_von(zustand::dok_nr(), seite).unwrap_or_default();
        let drehung = zustand::drehung_von(seite);
        let liste: Vec<_> = links.iter().map(|l| link_json(l, drehung)).collect();
        QString::from(&serde_json::Value::Array(liste).to_string())
    }

    fn link_bei(&self, seite: i32, x: f64, y: f64) -> QString {
        let Some(links) = zustand::links_von(zustand::dok_nr(), seite) else { return QString::default() };
        let (x, y) = zustand::ungedreht(x, y, zustand::drehung_von(seite));
        let (x, y) = (x as f32, y as f32);
        let Some(link) = links
            .iter()
            .find(|l| x >= l.rect.x && x <= l.rect.x + l.rect.b && y >= l.rect.y && y <= l.rect.y + l.rect.h)
        else {
            return QString::default();
        };
        QString::from(&link_json(link, zustand::drehung_von(seite)).to_string())
    }
}

/// Höchstzahl Treffer der Ordnersuche
const ORDNER_GRENZE: usize = 20_000;

/// Sprungziel eines Treffers: Seite und Rechteck in der (gedrehten) Anzeige
fn sprung_json(s: &zustand::Suche, idx: i64) -> serde_json::Value {
    let t = &s.treffer[idx as usize];
    let r = t.rects.first().copied().unwrap_or(engine::RelRect { x: 0.0, y: 0.0, b: 0.0, h: 0.0 });
    let r = zustand::gedreht(r, zustand::drehung_von(t.seite));
    json!({"seite": t.seite, "x": r.x, "y": r.y, "b": r.b, "h": r.h, "nr": idx + 1, "gesamt": s.treffer.len()})
}

/// PDF-Dateien eines Ordners, sortiert; versteckte Ordner und Verknüpfungen werden übersprungen
fn pdfs_finden(wurzel: &std::path::Path, unterordner: bool, aktuell: &dyn Fn() -> bool) -> Vec<PathBuf> {
    const HOECHSTENS: usize = 100_000;
    let mut dateien = Vec::new();
    let mut offen = vec![wurzel.to_path_buf()];
    while let Some(ordner) = offen.pop() {
        if !aktuell() || dateien.len() >= HOECHSTENS {
            break;
        }
        let Ok(eintraege) = std::fs::read_dir(&ordner) else { continue };
        let mut unter = Vec::new();
        for e in eintraege.flatten() {
            let Ok(art) = e.file_type() else { continue };
            let pfad = e.path();
            let versteckt = e.file_name().to_string_lossy().starts_with('.');
            if art.is_dir() {
                if unterordner && !versteckt {
                    unter.push(pfad);
                }
            } else if art.is_file() && pfad.extension().is_some_and(|x| x.eq_ignore_ascii_case("pdf")) {
                dateien.push(pfad);
            }
        }
        unter.sort();
        offen.extend(unter.into_iter().rev());
    }
    dateien.sort();
    dateien
}

/// Link als JSON; `rect` in Anzeige-Koordinaten der (gedrehten) Seite, Ziel-`y` ungedreht
fn link_json(link: &engine::Link, drehung: i32) -> serde_json::Value {
    let r = zustand::gedreht(link.rect, drehung);
    let rect = json!({"x": r.x, "y": r.y, "b": r.b, "h": r.h});
    match &link.ziel {
        LinkZiel::Intern { seite, y } => json!({"typ": "intern", "seite": seite, "y": y, "rect": rect}),
        LinkZiel::Extern { uri } => json!({"typ": "extern", "uri": uri, "rect": rect}),
    }
}

// ------------------------------------------------------------------------------------------------
// Bearbeiten, Anmerkungsliste, Gliederung, Speichern, Lesezeichen

/// Punkt der Anzeige (mit Ansichtsdrehung) -> relativ zur gerenderten Seite
fn seitenpunkt(seite: i32, x: f64, y: f64) -> (f32, f32) {
    let (x, y) = zustand::ungedreht(x, y, zustand::drehung_von(seite));
    (x as f32, y as f32)
}

fn json_punkt(seite: i32, v: &serde_json::Value) -> (f32, f32) {
    seitenpunkt(seite, v["x"].as_f64().unwrap_or(0.0), v["y"].as_f64().unwrap_or(0.0))
}

fn json_farbe(v: &serde_json::Value) -> bearbeiten::Farbe {
    // "#rrggbb" oder "#aarrggbb"
    let s = v.as_str().unwrap_or("#ffd400").trim_start_matches('#');
    let h = |i: usize| u8::from_str_radix(s.get(i..i + 2).unwrap_or("ff"), 16).unwrap_or(255);
    if s.len() == 8 { [h(2), h(4), h(6), h(0)] } else { [h(0), h(2), h(4), 255] }
}

fn rect_json(r: engine::RelRect) -> serde_json::Value {
    json!({"x": r.x, "y": r.y, "b": r.b, "h": r.h})
}

fn anmerkung_json(a: &bearbeiten::Anmerkung) -> serde_json::Value {
    let r = zustand::gedreht(a.rect, zustand::drehung_von(a.seite));
    let farbe = a.farbe.map(|f| format!("#{:02x}{:02x}{:02x}", f[0], f[1], f[2]));
    json!({"seite": a.seite, "index": a.index, "typ": a.typ, "rect": rect_json(r), "farbe": farbe,
           "inhalt": a.inhalt, "autor": a.autor, "datum": a.datum})
}

fn auftrag_lesen(text: &str) -> Result<bearbeiten::Bearbeitung, String> {
    use bearbeiten::{Bearbeitung, Markierung};
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let seite = v["seite"].as_i64().ok_or("seite fehlt")? as i32;
    let p = |name: &str| json_punkt(seite, &v[name]);
    let farbe = json_farbe(&v["farbe"]);
    let breite = v["breite"].as_f64().unwrap_or(2.0) as f32;
    let text_von = |name: &str| v[name].as_str().unwrap_or("").to_string();
    Ok(match v["art"].as_str().unwrap_or("") {
        art @ ("hervorheben" | "unterstreichen" | "durchstreichen" | "wellenlinie") => Bearbeitung::Markieren {
            seite,
            art: match art {
                "hervorheben" => Markierung::Hervorheben,
                "unterstreichen" => Markierung::Unterstreichen,
                "durchstreichen" => Markierung::Durchstreichen,
                _ => Markierung::Wellenlinie,
            },
            von: p("von"),
            bis: p("bis"),
            farbe,
        },
        "freihand" => Bearbeitung::Freihand {
            seite,
            striche: v["striche"]
                .as_array()
                .map(|s| {
                    s.iter()
                        .map(|strich| {
                            strich
                                .as_array()
                                .map(|pt| pt.iter().map(|q| json_punkt(seite, q)).collect())
                                .unwrap_or_default()
                        })
                        .collect()
                })
                .unwrap_or_default(),
            farbe,
            breite,
        },
        art @ ("rechteck" | "ellipse") => {
            Bearbeitung::Form { seite, ellipse: art == "ellipse", a: p("a"), b: p("b"), farbe, breite }
        }
        art @ ("linie" | "pfeil") => {
            Bearbeitung::Linie { seite, a: p("a"), b: p("b"), pfeil: art == "pfeil", farbe, breite }
        }
        "notiz" => Bearbeitung::Notiz { seite, punkt: p("a"), text: text_von("text"), farbe },
        "textfeld" => Bearbeitung::Textfeld {
            seite,
            a: p("a"),
            b: p("b"),
            text: text_von("text"),
            farbe,
            groesse: v["groesse"].as_f64().unwrap_or(12.0) as f32,
        },
        "bild" => Bearbeitung::Bild { seite, a: p("a"), b: p("b"), pfad: als_pfad(&QString::from(&text_von("pfad"))) },
        andere => return Err(format!("unbekannte Art: {andere}")),
    })
}

impl qobject::Dokument {
    /// nach jeder Änderung: Seite neu rendern lassen, Liste und Status aktualisieren
    fn geaendert_melden(mut self: Pin<&mut Self>, seite: Option<i32>, noch_ungespeichert: bool) {
        if let Some(seite) = seite {
            engine::cache().lock().unwrap().seite_verwerfen(zustand::dok_nr(), seite);
        }
        *zustand::ANMERKUNGEN.lock().unwrap() = None;
        self.as_mut().set_geaendert(noch_ungespeichert);
        let v = *self.anmerkungen_version() + 1;
        self.as_mut().set_anmerkungen_version(v);
    }

    fn text_auswahl(&self, seite: i32, x1: f64, y1: f64, x2: f64, y2: f64) -> QString {
        let (von, bis) = (seitenpunkt(seite, x1, y1), seitenpunkt(seite, x2, y2));
        let Ok(engine) = engine::engine() else { return QString::default() };
        let Ok((rects, text)) = engine.ausfuehren(move |stand| match stand.dok.as_ref() {
            Some(d) => bearbeiten::text_auswahl(d, seite, von, bis),
            None => (vec![], String::new()),
        }) else {
            return QString::default();
        };
        let drehung = zustand::drehung_von(seite);
        let rects: Vec<_> = rects.into_iter().map(|r| rect_json(zustand::gedreht(r, drehung))).collect();
        QString::from(&json!({"rects": rects, "text": text}).to_string())
    }

    fn anlegen(self: Pin<&mut Self>, auftrag: &QString) -> QString {
        let auftrag = match auftrag_lesen(&String::from(auftrag)) {
            Ok(a) => a,
            Err(e) => return QString::from(&e),
        };
        let ergebnis = engine::engine().and_then(|e| e.ausfuehren(move |stand| bearbeiten::anlegen(stand, auftrag)));
        match ergebnis.and_then(|r| r) {
            Ok(seite) => {
                self.geaendert_melden(Some(seite), true);
                QString::default()
            }
            Err(e) => QString::from(&e),
        }
    }

    fn loeschen(self: Pin<&mut Self>, seite: i32, index: i32) -> QString {
        let ergebnis = engine::engine()
            .and_then(|e| e.ausfuehren(move |stand| bearbeiten::loeschen(stand, seite, index.max(0) as usize)));
        match ergebnis.and_then(|r| r) {
            Ok(()) => {
                self.geaendert_melden(Some(seite), true);
                QString::default()
            }
            Err(e) => QString::from(&e),
        }
    }

    fn rueckgaengig(self: Pin<&mut Self>) -> QString {
        let ergebnis =
            engine::engine().and_then(|e| e.ausfuehren(|stand| (bearbeiten::rueckgaengig(stand), stand.geaendert)));
        match ergebnis {
            Ok((Ok(Some(seite)), geaendert)) => {
                self.geaendert_melden(Some(seite), geaendert);
                QString::default()
            }
            Ok((Ok(None), _)) => QString::from("Nichts rückgängig zu machen"),
            Ok((Err(e), _)) | Err(e) => QString::from(&e),
        }
    }

    fn inhalt_setzen(self: Pin<&mut Self>, seite: i32, index: i32, text: &QString) -> QString {
        let text = String::from(text);
        let ergebnis = engine::engine().and_then(|e| {
            e.ausfuehren(move |stand| bearbeiten::inhalt_setzen(stand, seite, index.max(0) as usize, &text))
        });
        match ergebnis.and_then(|r| r) {
            Ok(()) => {
                self.geaendert_melden(Some(seite), true);
                QString::default()
            }
            Err(e) => QString::from(&e),
        }
    }

    /// Anmerkungen aus dem Zwischenspeicher, sonst im PDF-Thread neu einsammeln
    fn mit_anmerkungen<T>(&self, f: impl FnOnce(&[bearbeiten::Anmerkung]) -> T) -> T {
        let dok = zustand::dok_nr();
        let mut zwischen = zustand::ANMERKUNGEN.lock().unwrap();
        if zwischen.as_ref().map(|(d, _)| *d) != Some(dok) {
            let liste = engine::engine()
                .and_then(|e| e.ausfuehren(|stand| stand.dok.as_ref().map(bearbeiten::anmerkungen).unwrap_or_default()))
                .unwrap_or_default();
            *zwischen = Some((dok, liste));
        }
        f(&zwischen.as_ref().unwrap().1)
    }

    fn anmerkungen_json(&self) -> QString {
        let liste = self.mit_anmerkungen(|l| l.iter().map(anmerkung_json).collect::<Vec<_>>());
        QString::from(&serde_json::Value::Array(liste).to_string())
    }

    fn anmerkung_bei(&self, seite: i32, x: f64, y: f64) -> QString {
        let (px, py) = seitenpunkt(seite, x, y);
        let treffer = self.mit_anmerkungen(|l| {
            // die oberste (zuletzt gezeichnete) Anmerkung gewinnt
            l.iter()
                .filter(|a| a.seite == seite)
                .rev()
                .find(|a| {
                    let r = a.rect;
                    let rand = 0.01;
                    px >= r.x - rand && px <= r.x + r.b + rand && py >= r.y - rand && py <= r.y + r.h + rand
                })
                .map(anmerkung_json)
        });
        treffer.map(|j| QString::from(&j.to_string())).unwrap_or_default()
    }

    fn gliederung_json(&self) -> QString {
        let liste = engine::engine()
            .and_then(|e| e.ausfuehren(|stand| stand.dok.as_ref().map(bearbeiten::gliederung).unwrap_or_default()))
            .unwrap_or_default();
        let liste: Vec<_> =
            liste.iter().map(|g| json!({"titel": g.titel, "tiefe": g.tiefe, "seite": g.seite, "y": g.y})).collect();
        QString::from(&serde_json::Value::Array(liste).to_string())
    }

    fn speichern(mut self: Pin<&mut Self>, pfad: &QString) -> QString {
        let ziel =
            if String::from(pfad).is_empty() { PathBuf::from(String::from(self.pfad())) } else { als_pfad(pfad) };
        let ziel2 = ziel.clone();
        let ergebnis = engine::engine().and_then(|e| e.ausfuehren(move |stand| bearbeiten::speichern(stand, &ziel2)));
        match ergebnis.and_then(|r| r) {
            Ok(()) => {
                self.as_mut().set_geaendert(false);
                self.as_mut().set_pfad(QString::from(&ziel.to_string_lossy().into_owned()));
                let titel = ziel.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.as_mut().set_titel(QString::from(&titel));
                QString::default()
            }
            Err(e) => QString::from(&e),
        }
    }

    fn lesezeichen_json(&self) -> QString {
        QString::from(&lesezeichen::als_json(&String::from(self.pfad())))
    }

    fn lesezeichen_setzen(mut self: Pin<&mut Self>, seite: i32, y: f64, name: &QString) {
        lesezeichen::setzen(&String::from(self.pfad()), seite, y, &String::from(name));
        let v = *self.lesezeichen_version() + 1;
        self.as_mut().set_lesezeichen_version(v);
    }

    fn lesezeichen_entfernen(mut self: Pin<&mut Self>, nr: i32) {
        lesezeichen::entfernen(&String::from(self.pfad()), nr);
        let v = *self.lesezeichen_version() + 1;
        self.as_mut().set_lesezeichen_version(v);
    }

    fn lesezeichen_umbenennen(mut self: Pin<&mut Self>, nr: i32, name: &QString) {
        lesezeichen::umbenennen(&String::from(self.pfad()), nr, &String::from(name));
        let v = *self.lesezeichen_version() + 1;
        self.as_mut().set_lesezeichen_version(v);
    }
}

#[cfg(test)]
mod tests {
    use super::pdfs_finden;

    #[test]
    fn pdfs_finden_mit_und_ohne_unterordner() {
        let wurzel = std::env::temp_dir().join(format!("abolos-pdfs-finden-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&wurzel);
        for d in ["unter", ".versteckt"] {
            std::fs::create_dir_all(wurzel.join(d)).unwrap();
        }
        for f in ["a.pdf", "B.PDF", "notiz.txt", "unter/c.pdf", ".versteckt/d.pdf"] {
            std::fs::write(wurzel.join(f), "x").unwrap();
        }
        let namen = |unterordner| {
            let mut v: Vec<String> = pdfs_finden(&wurzel, unterordner, &|| true)
                .iter()
                .map(|p| p.strip_prefix(&wurzel).unwrap().to_string_lossy().into_owned())
                .collect();
            v.sort();
            v
        };
        assert_eq!(namen(true), vec!["B.PDF", "a.pdf", "unter/c.pdf"]);
        assert_eq!(namen(false), vec!["B.PDF", "a.pdf"]);
        // Abbruch von außen
        assert!(pdfs_finden(&wurzel, true, &|| false).is_empty());
        std::fs::remove_dir_all(&wurzel).unwrap();
    }
}
