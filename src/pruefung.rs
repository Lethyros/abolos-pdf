// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Zusammenspiel-Tests über den echten PDF-Thread (PDFium) mit der Testdatei tests/daten/test.pdf.
//! Jeder Test arbeitet auf einer eigenen Kopie in einem Temp-Ordner; die Testdatei bleibt unverändert.
//! Die Engine gibt es nur einmal im Prozess, deshalb laufen diese Tests nacheinander (`SPERRE`).

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::bearbeiten::{self, Bearbeitung, Markierung};
use crate::engine::{self, OrdnerAuftrag, OrdnerTreffer};
use crate::suche::{self, Suchoptionen};

static SPERRE: Mutex<()> = Mutex::new(());

fn nacheinander() -> MutexGuard<'static, ()> {
    SPERRE.lock().unwrap_or_else(|e| e.into_inner())
}

/// eigener, leerer Temp-Ordner je Test
fn temp_ordner(name: &str) -> PathBuf {
    let ordner = std::env::temp_dir().join(format!("abolos-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&ordner);
    std::fs::create_dir_all(&ordner).unwrap();
    ordner
}

fn testkopie(ordner: &Path, name: &str) -> PathBuf {
    let ziel = ordner.join(name);
    std::fs::copy(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/daten/test.pdf"), &ziel).unwrap();
    ziel
}

fn oeffnen(pfad: &Path) {
    let (_, masse) = engine::engine().unwrap().oeffnen(pfad).unwrap();
    assert_eq!(masse.len(), 2, "Testdatei hat zwei Seiten");
    assert!((masse[0].breite - 595.0).abs() < 1.0 && (masse[0].hoehe - 842.0).abs() < 1.0, "A4");
}

fn treffer(seite: i32, text: &str, optionen: Suchoptionen) -> Vec<engine::Treffer> {
    let nadel = suche::nadel(text);
    engine::engine()
        .unwrap()
        .ausfuehren(move |stand| engine::seite_durchsuchen(stand.dok.as_ref().unwrap(), seite, &nadel, optionen))
        .unwrap()
}

fn anmerkungen() -> Vec<bearbeiten::Anmerkung> {
    engine::engine().unwrap().ausfuehren(|stand| bearbeiten::anmerkungen(stand.dok.as_ref().unwrap())).unwrap()
}

fn anlegen(b: Bearbeitung) -> Result<i32, String> {
    engine::engine().unwrap().ausfuehren(move |stand| bearbeiten::anlegen(stand, b)).unwrap()
}

#[test]
fn suche_findet_woerter_mit_lage_und_zusammenhang() {
    let _s = nacheinander();
    let ordner = temp_ordner("suche");
    oeffnen(&testkopie(&ordner, "a.pdf"));

    // "Schutz", "Schutzleiter", "Fehlerschutz"
    let alle = treffer(0, "schutz", Suchoptionen::default());
    assert_eq!(alle.len(), 3);
    for t in &alle {
        let r = t.rects[0];
        assert!(r.x > 0.1 && r.x < 0.9 && r.y > 0.05 && r.y < 0.2 && r.b > 0.0 && r.h > 0.0, "Lage {r:?}");
    }
    assert_eq!(alle[0].wort, "Schutz");
    assert!(alle[0].nach.starts_with(" und Schutzleiter"), "Zusammenhang: {:?}", alle[0].nach);

    let woerter = treffer(0, "schutz", Suchoptionen { gross_klein: false, ganze_woerter: true });
    assert_eq!(woerter.len(), 1);
    assert_eq!(treffer(0, "Schutz", Suchoptionen { gross_klein: true, ganze_woerter: false }).len(), 2);

    // Umlaute (WinAnsi-Schrift) und Wortgruppen über Leerzeichen
    assert_eq!(treffer(0, "gefährlich", Suchoptionen::default()).len(), 1);
    assert_eq!(treffer(0, "erdungs- und", Suchoptionen::default()).len(), 1);
    assert_eq!(treffer(1, "niederspannungsanlagen", Suchoptionen::default()).len(), 1);
    std::fs::remove_dir_all(&ordner).unwrap();
}

/// Ordnersuche über den PDF-Thread; Rückgabe: Treffer und Zahl unlesbarer Dateien
fn ordnersuche(dateien: Vec<PathBuf>, text: &str) -> (Vec<OrdnerTreffer>, usize) {
    let gefunden: Arc<Mutex<Vec<OrdnerTreffer>>> = Arc::default();
    let unlesbar = Arc::new(Mutex::new(0));
    let (fertig_tx, fertig_rx) = mpsc::channel();
    let (g, u) = (gefunden.clone(), unlesbar.clone());
    engine::engine().unwrap().ordner_suchen(OrdnerAuftrag {
        dateien,
        text: text.into(),
        optionen: Suchoptionen::default(),
        grenze: 1000,
        bei_treffern: Box::new(move |neu| g.lock().unwrap().extend(neu)),
        fortschritt: Box::new(move |f| *u.lock().unwrap() = f.unlesbar),
        fertig: Box::new(move |voll| {
            let _ = fertig_tx.send(voll);
        }),
    });
    let voll = fertig_rx.recv_timeout(Duration::from_secs(30)).expect("Ordnersuche endet");
    assert!(!voll, "Trefferlimit nicht erreicht");
    let treffer = gefunden.lock().unwrap().clone();
    let unlesbar = *unlesbar.lock().unwrap();
    (treffer, unlesbar)
}

#[test]
fn ordnersuche_ueber_mehrere_dateien_und_zwischenspeicher() {
    let _s = nacheinander();
    let ordner = temp_ordner("ordner");
    let a = testkopie(&ordner, "a.pdf");
    let b = testkopie(&ordner, "b.pdf");
    let kaputt = ordner.join("kaputt.pdf");
    std::fs::write(&kaputt, "kein PDF").unwrap();

    let dateien = vec![a.clone(), b.clone(), kaputt];
    let (erste, unlesbar) = ordnersuche(dateien.clone(), "schutz");
    // je Datei: drei auf Seite 1, einer auf Seite 2
    assert_eq!(erste.len(), 8);
    assert_eq!(unlesbar, 1);
    assert_eq!(erste.iter().filter(|t| *t.datei == a).count(), 4);
    assert_eq!(erste.iter().filter(|t| t.seite == 1).count(), 2);
    assert_eq!(erste[1].nr, 1, "Treffer werden je Seite durchgezählt");

    // zweiter Lauf kommt aus dem Textspeicher und muss dasselbe liefern
    let (zweite, _) = ordnersuche(dateien, "schutz");
    let kurz =
        |v: &[OrdnerTreffer]| v.iter().map(|t| (t.datei.clone(), t.seite, t.nr, t.wort.clone())).collect::<Vec<_>>();
    assert_eq!(kurz(&erste), kurz(&zweite));
    std::fs::remove_dir_all(&ordner).unwrap();
}

#[test]
fn alle_anmerkungsarten_anlegen_rueckgaengig_speichern_und_wieder_laden() {
    let _s = nacheinander();
    let ordner = temp_ordner("anmerkungen");
    let pdf = testkopie(&ordner, "a.pdf");
    let bild = ordner.join("bild.png");
    image::RgbaImage::from_pixel(8, 4, image::Rgba([200, 30, 30, 255])).save(&bild).unwrap();
    oeffnen(&pdf);

    let gelb = [255, 212, 0, 255];
    // erste Textzeile: Grundlinie 770 pt von unten, ab 72 pt vom linken Rand
    let zeile = |art| Bearbeitung::Markieren { seite: 0, art, von: (0.125, 0.08), bis: (0.45, 0.08), farbe: gelb };
    let auftraege = vec![
        ("Hervorhebung", zeile(Markierung::Hervorheben)),
        ("Unterstreichung", zeile(Markierung::Unterstreichen)),
        ("Durchstreichung", zeile(Markierung::Durchstreichen)),
        ("Wellenlinie", zeile(Markierung::Wellenlinie)),
        (
            "Freihand",
            Bearbeitung::Freihand {
                seite: 0,
                striche: vec![vec![(0.1, 0.7), (0.2, 0.72), (0.3, 0.7)]],
                farbe: gelb,
                breite: 2.0,
            },
        ),
        (
            "Rechteck",
            Bearbeitung::Form { seite: 0, ellipse: false, a: (0.1, 0.8), b: (0.3, 0.9), farbe: gelb, breite: 2.0 },
        ),
        (
            "Ellipse",
            Bearbeitung::Form { seite: 0, ellipse: true, a: (0.35, 0.8), b: (0.5, 0.9), farbe: gelb, breite: 2.0 },
        ),
        (
            "Linie",
            Bearbeitung::Linie { seite: 0, a: (0.55, 0.8), b: (0.8, 0.8), pfeil: false, farbe: gelb, breite: 2.0 },
        ),
        (
            "Pfeil",
            Bearbeitung::Linie { seite: 0, a: (0.55, 0.85), b: (0.8, 0.9), pfeil: true, farbe: gelb, breite: 3.0 },
        ),
        ("Notiz", Bearbeitung::Notiz { seite: 1, punkt: (0.8, 0.3), text: "Bitte prüfen".into(), farbe: gelb }),
        (
            "Textfeld",
            Bearbeitung::Textfeld {
                seite: 1,
                a: (0.1, 0.5),
                b: (0.5, 0.6),
                text: "Hinweis".into(),
                farbe: gelb,
                groesse: 11.0,
            },
        ),
        ("Bild", Bearbeitung::Bild { seite: 1, a: (0.6, 0.5), b: (0.8, 0.6), pfad: bild }),
    ];
    let erwartet: Vec<&str> = auftraege.iter().map(|(typ, _)| *typ).collect();
    for (typ, b) in auftraege {
        anlegen(b).unwrap_or_else(|e| panic!("{typ}: {e}"));
    }
    let liste = anmerkungen();
    assert_eq!(liste.iter().map(|a| a.typ.as_str()).collect::<Vec<_>>(), erwartet);
    assert!(liste[0].inhalt.contains("Schutz"), "markierter Text wird Kommentar: {:?}", liste[0].inhalt);
    assert_eq!(liste[0].autor, bearbeiten::autor());

    // Fehlbedienung meldet sich, statt etwas Kaputtes anzulegen
    assert!(
        anlegen(Bearbeitung::Linie { seite: 0, a: (0.5, 0.5), b: (0.5, 0.5), pfeil: true, farbe: gelb, breite: 1.0 })
            .is_err()
    );

    // Rückgängig entfernt die zuletzt angelegte (Bild), Kommentar ändern
    let geaendert =
        engine::engine().unwrap().ausfuehren(|stand| bearbeiten::rueckgaengig(stand).map(|_| stand.geaendert)).unwrap();
    assert_eq!(geaendert, Ok(true));
    let notiz = anmerkungen().into_iter().find(|a| a.typ == "Notiz").unwrap();
    let (s, i) = (notiz.seite, notiz.index);
    engine::engine()
        .unwrap()
        .ausfuehren(move |stand| bearbeiten::inhalt_setzen(stand, s, i, "Erledigt"))
        .unwrap()
        .unwrap();

    // speichern unter neuem Namen und neu laden: alles noch da
    let ziel = ordner.join("gespeichert.pdf");
    let z = ziel.clone();
    let nach_speichern = engine::engine()
        .unwrap()
        .ausfuehren(move |stand| bearbeiten::speichern(stand, &z).map(|_| stand.geaendert))
        .unwrap();
    assert_eq!(nach_speichern, Ok(false));
    assert!(!ordner.join("gespeichert.pdf.abolos-tmp").exists(), "Zwischendatei aufgeräumt");
    oeffnen(&ziel);
    let geladen = anmerkungen();
    assert_eq!(geladen.iter().map(|a| a.typ.as_str()).collect::<Vec<_>>(), erwartet[..erwartet.len() - 1]);
    assert_eq!(geladen.iter().find(|a| a.typ == "Notiz").unwrap().inhalt, "Erledigt");
    // die Testdatei selbst ist unverändert
    oeffnen(&pdf);
    assert!(anmerkungen().is_empty());
    std::fs::remove_dir_all(&ordner).unwrap();
}

#[test]
fn textauswahl_liefert_text_und_rechtecke() {
    let _s = nacheinander();
    let ordner = temp_ordner("auswahl");
    oeffnen(&testkopie(&ordner, "a.pdf"));
    let (rects, text) = engine::engine()
        .unwrap()
        .ausfuehren(|stand| bearbeiten::text_auswahl(stand.dok.as_ref().unwrap(), 0, (0.125, 0.08), (0.45, 0.08)))
        .unwrap();
    assert!(!rects.is_empty());
    assert!(text.starts_with("Schutz und Schutzleiter"), "{text:?}");
    std::fs::remove_dir_all(&ordner).unwrap();
}
