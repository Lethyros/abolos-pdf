// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Gemeinsamer Zustand zwischen Dokument-Objekt, Seiten-Elementen und PDF-Thread.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};

use crate::engine::{Link, RelRect, Seitenmass, Treffer};

/// Abstand zwischen den Seiten in PDF-Punkten (wird mit dem Zoom skaliert)
pub const SEITENABSTAND: f64 = 12.0;

/// Nummer des geöffneten Dokuments; jedes Öffnen zählt hoch, alte Aufträge verfallen damit
pub static DOK_NR: AtomicU64 = AtomicU64::new(0);

pub fn dok_nr() -> u64 {
    DOK_NR.load(Ordering::Acquire)
}

/// Seitenanordnung: einzeln untereinander oder je zwei nebeneinander
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Modus {
    #[default]
    Einzeln,
    Doppelt,
    /// Doppelseiten, die erste Seite (Deckblatt) allein, wie in einem Buch
    DoppeltDeckblatt,
}

impl Modus {
    pub fn aus_zahl(n: i32) -> Modus {
        match n {
            1 => Modus::Doppelt,
            2 => Modus::DoppeltDeckblatt,
            _ => Modus::Einzeln,
        }
    }

    pub fn zahl(self) -> i32 {
        match self {
            Modus::Einzeln => 0,
            Modus::Doppelt => 1,
            Modus::DoppeltDeckblatt => 2,
        }
    }
}

/// Ansichtseinstellungen (ändern nichts am Dokument)
#[derive(Default)]
pub struct Ansicht {
    pub modus: Modus,
    /// Drehung aller Seiten im Uhrzeigersinn: 0, 90, 180, 270
    pub drehung: i32,
    /// zusätzliche Drehung einzelner Seiten
    pub seiten_drehung: HashMap<i32, i32>,
}

impl Ansicht {
    pub fn drehung_von(&self, seite: i32) -> i32 {
        (self.drehung + self.seiten_drehung.get(&seite).copied().unwrap_or(0)).rem_euclid(360)
    }
}

pub static ANSICHT: RwLock<Option<Ansicht>> = RwLock::new(None);

pub fn drehung_von(seite: i32) -> i32 {
    ANSICHT.read().unwrap().as_ref().map(|a| a.drehung_von(seite)).unwrap_or(0)
}

/// Seitenpositionen in Punkten bei Zoom 1. x zählt von der Mittellinie der Ansicht.
#[derive(Default)]
pub struct Layout {
    /// Seitengrößen ohne Ansichtsdrehung
    pub masse: Vec<Seitenmass>,
    /// angezeigte Größe (Breite, Höhe) nach Drehung
    pub anzeige: Vec<(f64, f64)>,
    pub x: Vec<f64>,
    pub oben: Vec<f64>,
    /// Zeilen: erste Seite, letzte Seite, Oberkante, Höhe
    pub zeilen: Vec<(usize, usize, f64, f64)>,
    pub gesamt: f64,
    /// breiteste Zeile
    pub max_breite: f64,
}

impl Layout {
    pub fn neu(masse: Vec<Seitenmass>) -> Layout {
        let mut l = Layout { masse, ..Layout::default() };
        l.berechnen();
        l
    }

    pub fn berechnen(&mut self) {
        let ansicht = ANSICHT.read().unwrap();
        let standard = Ansicht::default();
        let ansicht = ansicht.as_ref().unwrap_or(&standard);
        let n = self.masse.len();
        self.anzeige = (0..n)
            .map(|i| {
                let m = self.masse[i];
                if ansicht.drehung_von(i as i32) % 180 == 0 {
                    (m.breite as f64, m.hoehe as f64)
                } else {
                    (m.hoehe as f64, m.breite as f64)
                }
            })
            .collect();
        // Zeilen bilden
        let mut gruppen: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;
        while i < n {
            let paar = match ansicht.modus {
                Modus::Einzeln => false,
                Modus::Doppelt => true,
                Modus::DoppeltDeckblatt => i > 0,
            };
            let ende = if paar && i + 1 < n { i + 1 } else { i };
            gruppen.push((i, ende));
            i = ende + 1;
        }
        self.x = vec![0.0; n];
        self.oben = vec![0.0; n];
        self.zeilen.clear();
        self.max_breite = 0.0;
        let mut y = SEITENABSTAND;
        for (a, b) in gruppen {
            let hoehe = (a..=b).map(|i| self.anzeige[i].1).fold(0.0, f64::max);
            if a == b {
                self.x[a] = -self.anzeige[a].0 / 2.0;
                self.max_breite = self.max_breite.max(self.anzeige[a].0);
            } else {
                self.x[a] = -SEITENABSTAND / 2.0 - self.anzeige[a].0;
                self.x[b] = SEITENABSTAND / 2.0;
                let breite = 2.0 * self.anzeige[a].0.max(self.anzeige[b].0) + SEITENABSTAND;
                self.max_breite = self.max_breite.max(breite);
            }
            for i in a..=b {
                self.oben[i] = y + (hoehe - self.anzeige[i].1) / 2.0; // in der Zeile senkrecht mittig
            }
            self.zeilen.push((a, b, y, hoehe));
            y += hoehe + SEITENABSTAND;
        }
        self.gesamt = y;
    }

    /// Zeile, die die Höhe `y` (Punkte) enthält; zwischen zwei Zeilen zählt die folgende
    pub fn zeile_bei(&self, y: f64) -> Option<usize> {
        if self.zeilen.is_empty() {
            return None;
        }
        let i = self.zeilen.partition_point(|z| z.2 <= y);
        Some(i.saturating_sub(1).min(self.zeilen.len() - 1))
    }

    pub fn zeile_von(&self, seite: i32) -> Option<usize> {
        let s = seite.max(0) as usize;
        let i = self.zeilen.partition_point(|z| z.1 < s);
        (i < self.zeilen.len()).then_some(i)
    }

    /// erste Seite der Zeile an Höhe `y`
    pub fn seite_bei(&self, y: f64) -> i32 {
        self.zeile_bei(y).map(|z| self.zeilen[z].0 as i32).unwrap_or(-1)
    }

    /// letzte Seite der Zeile an Höhe `y`
    pub fn letzte_seite_bei(&self, y: f64) -> i32 {
        self.zeile_bei(y).map(|z| self.zeilen[z].1 as i32).unwrap_or(-1)
    }
}

pub static LAYOUT: RwLock<Layout> = RwLock::new(Layout {
    masse: Vec::new(),
    anzeige: Vec::new(),
    x: Vec::new(),
    oben: Vec::new(),
    zeilen: Vec::new(),
    gesamt: 0.0,
    max_breite: 0.0,
});

/// Rechteck einer Seite (relativ 0..1, ungedreht) in die gedrehte Anzeige umrechnen
pub fn gedreht(r: RelRect, grad: i32) -> RelRect {
    match grad.rem_euclid(360) {
        90 => RelRect { x: 1.0 - r.y - r.h, y: r.x, b: r.h, h: r.b },
        180 => RelRect { x: 1.0 - r.x - r.b, y: 1.0 - r.y - r.h, b: r.b, h: r.h },
        270 => RelRect { x: r.y, y: 1.0 - r.x - r.b, b: r.h, h: r.b },
        _ => r,
    }
}

/// Punkt der gedrehten Anzeige (relativ) zurück auf die ungedrehte Seite
pub fn ungedreht(x: f64, y: f64, grad: i32) -> (f64, f64) {
    match grad.rem_euclid(360) {
        90 => (y, 1.0 - x),
        180 => (1.0 - x, 1.0 - y),
        270 => (1.0 - y, x),
        _ => (x, y),
    }
}

/// Suchtreffer, sortiert nach Seite und Lage auf der Seite
pub struct Suche {
    pub generation: u64,
    pub treffer: Vec<Treffer>,
    /// ausgewählter Treffer (Index in `treffer`), -1 = keiner
    pub aktuell: i64,
}

impl Suche {
    pub fn leeren(&mut self) -> u64 {
        self.generation += 1;
        self.treffer.clear();
        self.aktuell = -1;
        self.generation
    }

    /// Die Suche läuft ab der aktuellen Seite im Kreis, Treffer kommen also nicht der Reihe nach
    pub fn einfuegen(&mut self, neu: Vec<Treffer>) {
        for t in neu {
            let schluessel = (t.seite, ordnung(&t));
            let pos = self.treffer.partition_point(|x| (x.seite, ordnung(x)) < schluessel);
            self.treffer.insert(pos, t);
            if self.aktuell >= pos as i64 {
                self.aktuell += 1; // ausgewählter Treffer bleibt derselbe
            }
        }
    }

    /// Bereich der Treffer einer Seite in `treffer`
    pub fn bereich(&self, seite: i32) -> std::ops::Range<usize> {
        let a = self.treffer.partition_point(|t| t.seite < seite);
        let b = self.treffer.partition_point(|t| t.seite <= seite);
        a..b
    }
}

fn ordnung(t: &Treffer) -> (i64, i64) {
    let r = t.rects.first().copied().unwrap_or(RelRect { x: 0.0, y: 0.0, b: 0.0, h: 0.0 });
    ((r.y * 1e6) as i64, (r.x * 1e6) as i64)
}

pub static SUCHE: Mutex<Suche> = Mutex::new(Suche { generation: 0, treffer: Vec::new(), aktuell: -1 });

/// Links je (Dokument, Seite)
pub type LinkSpeicher = HashMap<(u64, i32), Vec<Link>>;

/// Links aller angeforderten Seiten; fehlt ein Eintrag, ist er noch nicht angefordert
pub static LINKS: Mutex<Option<LinkSpeicher>> = Mutex::new(None);

pub fn links_von(dok: u64, seite: i32) -> Option<Vec<Link>> {
    LINKS.lock().unwrap().as_ref().and_then(|m| m.get(&(dok, seite)).cloned())
}

/// zuletzt eingesammelte Anmerkungen (Dokumentnummer, Liste); None = neu einsammeln
pub static ANMERKUNGEN: Mutex<Option<(u64, Vec<crate::bearbeiten::Anmerkung>)>> = Mutex::new(None);

/// Ergebnisse der Ordnersuche (in Datei- und Seitenreihenfolge, nur angehängt)
pub struct Ordnersuche {
    pub generation: u64,
    pub wurzel: std::path::PathBuf,
    pub text: String,
    pub optionen: crate::suche::Suchoptionen,
    pub treffer: Vec<crate::engine::OrdnerTreffer>,
    /// Zahl der Dateien mit Treffern
    pub dateien: usize,
    /// beschädigte oder mit Passwort geschützte PDFs
    pub unlesbar: usize,
}

pub static ORDNER: Mutex<Ordnersuche> = Mutex::new(Ordnersuche {
    generation: 0,
    wurzel: std::path::PathBuf::new(),
    text: String::new(),
    optionen: crate::suche::Suchoptionen { gross_klein: false, ganze_woerter: false },
    treffer: Vec::new(),
    dateien: 0,
    unlesbar: 0,
});

#[cfg(test)]
mod tests {
    use super::*;

    /// ANSICHT ist global: Tests, die sie setzen, laufen nacheinander
    static SPERRE: Mutex<()> = Mutex::new(());

    fn a4(n: usize) -> Vec<Seitenmass> {
        (0..n).map(|_| Seitenmass { breite: 595.0, hoehe: 842.0 }).collect()
    }

    fn mit_ansicht(modus: Modus, drehung: i32, pruefen: impl FnOnce()) {
        let _s = SPERRE.lock().unwrap_or_else(|e| e.into_inner());
        *ANSICHT.write().unwrap() = Some(Ansicht { modus, drehung, seiten_drehung: HashMap::new() });
        pruefen();
        *ANSICHT.write().unwrap() = None;
    }

    #[test]
    fn einzelseiten_stehen_untereinander() {
        mit_ansicht(Modus::Einzeln, 0, || {
            let l = Layout::neu(a4(3));
            assert_eq!(l.zeilen.len(), 3);
            assert_eq!(l.x[0], -595.0 / 2.0, "mittig");
            assert_eq!(l.oben[1], l.oben[0] + 842.0 + SEITENABSTAND);
            assert_eq!(l.gesamt, 3.0 * 842.0 + 4.0 * SEITENABSTAND);
            assert_eq!(l.seite_bei(l.oben[2] + 10.0), 2);
        });
    }

    #[test]
    fn doppelseiten_und_buchansicht() {
        mit_ansicht(Modus::Doppelt, 0, || {
            let l = Layout::neu(a4(5));
            let paare: Vec<(usize, usize)> = l.zeilen.iter().map(|z| (z.0, z.1)).collect();
            assert_eq!(paare, vec![(0, 1), (2, 3), (4, 4)]);
            assert_eq!(l.letzte_seite_bei(l.oben[2] + 1.0), 3);
        });
        mit_ansicht(Modus::DoppeltDeckblatt, 0, || {
            let l = Layout::neu(a4(5));
            let paare: Vec<(usize, usize)> = l.zeilen.iter().map(|z| (z.0, z.1)).collect();
            assert_eq!(paare, vec![(0, 0), (1, 2), (3, 4)], "Deckblatt allein");
            assert_eq!(l.zeile_von(2), Some(1));
        });
    }

    #[test]
    fn drehung_tauscht_breite_und_hoehe() {
        mit_ansicht(Modus::Einzeln, 90, || {
            let l = Layout::neu(a4(1));
            assert_eq!(l.anzeige[0], (842.0, 595.0));
            assert_eq!(l.max_breite, 842.0);
        });
    }

    #[test]
    fn gedreht_und_ungedreht_heben_sich_auf() {
        for grad in [0, 90, 180, 270] {
            let r = gedreht(RelRect { x: 0.2, y: 0.3, b: 0.0, h: 0.0 }, grad);
            let (x, y) = ungedreht(r.x as f64, r.y as f64, grad);
            assert!((x - 0.2).abs() < 1e-6 && (y - 0.3).abs() < 1e-6, "{grad}°: {x} {y}");
        }
    }

    fn treffer(seite: i32, y: f32) -> Treffer {
        Treffer {
            seite,
            rects: vec![RelRect { x: 0.1, y, b: 0.1, h: 0.02 }],
            vor: String::new(),
            wort: String::new(),
            nach: String::new(),
        }
    }

    #[test]
    fn suchtreffer_bleiben_sortiert_und_die_auswahl_wandert_mit() {
        let mut s = Suche { generation: 0, treffer: Vec::new(), aktuell: -1 };
        // die Suche beginnt mitten im Dokument und läuft im Kreis
        s.einfuegen(vec![treffer(5, 0.5), treffer(5, 0.1)]);
        s.aktuell = 1; // Seite 5, y 0.5
        s.einfuegen(vec![treffer(1, 0.3)]);
        let lage: Vec<(i32, f32)> = s.treffer.iter().map(|t| (t.seite, t.rects[0].y)).collect();
        assert_eq!(lage, vec![(1, 0.3), (5, 0.1), (5, 0.5)]);
        assert_eq!(s.aktuell, 2, "ausgewählt bleibt Seite 5, y 0.5");
        assert_eq!(s.bereich(5), 1..3);
    }
}
