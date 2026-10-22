// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! PDF-Kern: ein Arbeits-Thread besitzt PDFium allein (PDFium ist nicht threadsicher).
//! Die Oberfläche schickt Aufträge, Ergebnisse kommen über Rückruf-Funktionen zurück.
//! Render-Aufträge haben Vorrang vor der Suche, damit Scrollen auch während einer langen
//! Suche in großen PDFs flüssig bleibt.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use pdfium_render::prelude::*;

use crate::suche::Suchoptionen;

/// Seitengröße in PDF-Punkten (1/72 Zoll).
#[derive(Clone, Copy, Debug)]
pub struct Seitenmass {
    pub breite: f32,
    pub hoehe: f32,
}

/// Rechteck relativ zur Seite: 0..1, Ursprung oben links (PDF zählt von unten).
#[derive(Clone, Copy, Debug)]
pub struct RelRect {
    pub x: f32,
    pub y: f32,
    pub b: f32,
    pub h: f32,
}

#[derive(Clone, Debug)]
pub enum LinkZiel {
    /// Sprung im Dokument; `y` = relative Höhe auf der Zielseite, falls angegeben
    Intern { seite: i32, y: Option<f32> },
    /// Web- oder Mail-Adresse: wird bewusst nie geöffnet (Programm ist offline)
    Extern { uri: String },
}

#[derive(Clone, Debug)]
pub struct Link {
    pub rect: RelRect,
    pub ziel: LinkZiel,
}

pub struct Bild {
    pub breite: i32,
    pub hoehe: i32,
    pub rgba: Arc<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct Treffer {
    pub seite: i32,
    pub rects: Vec<RelRect>,
    /// Zusammenhang für die Trefferliste
    pub vor: String,
    pub wort: String,
    pub nach: String,
}

/// Fundstelle der Ordnersuche; `nr` zählt die Treffer auf der Seite (für das Rechteck beim Öffnen)
#[derive(Clone, Debug)]
pub struct OrdnerTreffer {
    pub datei: Arc<PathBuf>,
    pub seite: i32,
    pub nr: usize,
    pub vor: String,
    pub wort: String,
    pub nach: String,
}

/// Stand der Ordnersuche nach jeder Datei
pub struct OrdnerFortschritt {
    pub fertig: usize,
    pub gesamt: usize,
    pub unlesbar: usize,
    pub datei: String,
}

type Rueckruf<T> = Box<dyn FnOnce(T) + Send>;

pub struct SuchAuftrag {
    pub text: String,
    pub optionen: Suchoptionen,
    pub ab_seite: i32,
    /// Treffer einer Seite (nur Seiten mit Treffern)
    pub bei_treffern: Box<dyn Fn(Vec<Treffer>) + Send>,
    /// Fortschritt 0..1, regelmäßig
    pub fortschritt: Box<dyn Fn(f32) + Send>,
    pub fertig: Rueckruf<()>,
}

pub struct OrdnerAuftrag {
    pub dateien: Vec<PathBuf>,
    pub text: String,
    pub optionen: Suchoptionen,
    /// höchstens so viele Treffer, dann endet die Suche
    pub grenze: usize,
    pub bei_treffern: Box<dyn Fn(Vec<OrdnerTreffer>) + Send>,
    pub fortschritt: Box<dyn Fn(OrdnerFortschritt) + Send>,
    /// true = Trefferlimit erreicht
    pub fertig: Rueckruf<bool>,
}

enum Auftrag {
    /// Antwort: neue Dokumentnummer und Seitengrößen
    Oeffnen {
        pfad: PathBuf,
        antwort: Sender<Result<(u64, Vec<Seitenmass>), String>>,
    },
    /// `gueltig` wird kurz vor dem Rendern gefragt: weggescrollte Seiten werden übersprungen
    /// `breite_px` gilt für die ungedrehte Seite; das Ergebnis wird danach um `drehung` gedreht
    Rendern {
        dok: u64,
        seite: i32,
        breite_px: i32,
        drehung: i32,
        gueltig: Box<dyn Fn() -> bool + Send>,
        fertig: Rueckruf<Option<Bild>>,
    },
    Links {
        dok: u64,
        seite: i32,
        fertig: Rueckruf<Vec<Link>>,
    },
    Suchen(SuchAuftrag),
    SucheAbbrechen,
    Ordnersuche(OrdnerAuftrag),
    OrdnerAbbrechen,
    /// beliebige Arbeit am Dokument im PDF-Thread (Bearbeiten, Listen, Speichern)
    Ausfuehren(Box<dyn for<'a> FnOnce(&mut Stand<'a>) + Send>),
}

/// Zustand des PDF-Threads, den `Engine::ausfuehren` erreicht
pub struct Stand<'a> {
    pub dok: Option<PdfDocument<'a>>,
    /// Nummer des Dokuments (siehe `zustand::DOK_NR`)
    pub nr: u64,
    /// selbst angelegte Anmerkungen (Seite, Index) für Rückgängig, neueste zuletzt
    pub rueckgaengig: Vec<(i32, usize)>,
    /// ungespeicherte Änderungen
    pub geaendert: bool,
}

pub struct Engine {
    tx: Mutex<Sender<Auftrag>>,
}

static ENGINE: OnceLock<Result<Engine, String>> = OnceLock::new();

/// Die eine Engine des Programms; startet beim ersten Zugriff.
pub fn engine() -> Result<&'static Engine, String> {
    ENGINE.get_or_init(Engine::starten).as_ref().map_err(Clone::clone)
}

impl Engine {
    fn starten() -> Result<Engine, String> {
        let (bibliothek, ort) = pdfium_bibliothek()?;
        let (tx, rx) = mpsc::channel();
        let (bereit_tx, bereit_rx) = mpsc::channel();
        thread::Builder::new()
            .name("pdfium".into())
            .spawn(move || {
                let roh = match &ort {
                    Some(p) => Pdfium::bind_to_library(p),
                    None => Pdfium::bind_to_system_library(),
                };
                if let Ok(roh) = roh {
                    ROH.with(|r| {
                        let _ = r.set(roh);
                    });
                }
                arbeiter(bibliothek, rx, bereit_tx)
            })
            .map_err(|e| format!("PDF-Thread nicht startbar: {e}"))?;
        bereit_rx.recv().map_err(|_| "PDF-Thread beendet".to_string())??;
        Ok(Engine { tx: Mutex::new(tx) })
    }

    fn senden(&self, auftrag: Auftrag) {
        let _ = self.tx.lock().unwrap().send(auftrag);
    }

    /// Öffnet ein Dokument (blockiert kurz; PDFium lädt Seiten erst bei Bedarf).
    pub fn oeffnen(&self, pfad: &Path) -> Result<(u64, Vec<Seitenmass>), String> {
        let (antwort, rx) = mpsc::channel();
        self.senden(Auftrag::Oeffnen { pfad: pfad.to_path_buf(), antwort });
        rx.recv().map_err(|_| "PDF-Thread beendet".to_string())?
    }

    #[allow(clippy::too_many_arguments)]
    pub fn rendern(
        &self,
        dok: u64,
        seite: i32,
        breite_px: i32,
        drehung: i32,
        gueltig: Box<dyn Fn() -> bool + Send>,
        fertig: Rueckruf<Option<Bild>>,
    ) {
        self.senden(Auftrag::Rendern { dok, seite, breite_px, drehung, gueltig, fertig });
    }

    pub fn links(&self, dok: u64, seite: i32, fertig: Rueckruf<Vec<Link>>) {
        self.senden(Auftrag::Links { dok, seite, fertig });
    }

    pub fn suchen(&self, auftrag: SuchAuftrag) {
        self.senden(Auftrag::Suchen(auftrag));
    }

    pub fn suche_abbrechen(&self) {
        self.senden(Auftrag::SucheAbbrechen);
    }

    pub fn ordner_suchen(&self, auftrag: OrdnerAuftrag) {
        self.senden(Auftrag::Ordnersuche(auftrag));
    }

    pub fn ordner_abbrechen(&self) {
        self.senden(Auftrag::OrdnerAbbrechen);
    }

    /// Arbeit im PDF-Thread ausführen und auf das Ergebnis warten (blockiert kurz)
    pub fn ausfuehren<T: Send + 'static>(
        &self,
        f: impl for<'a> FnOnce(&mut Stand<'a>) -> T + Send + 'static,
    ) -> Result<T, String> {
        let (tx, rx) = mpsc::channel();
        self.senden(Auftrag::Ausfuehren(Box::new(move |stand| {
            let _ = tx.send(f(stand));
        })));
        rx.recv().map_err(|_| "PDF-Thread beendet".to_string())
    }
}

/// libpdfium.so bzw. pdfium.dll suchen: $ABOLOS_PDFIUM, neben dem Programm (auch lib/ und ../lib wie
/// in den Linux-Paketen), im Projekt, im System.
/// Rückgabe: Anbindung und Fundort (None = Systembibliothek)
fn pdfium_bibliothek() -> Result<(Box<dyn PdfiumLibraryBindings>, Option<PathBuf>), String> {
    let mut orte: Vec<PathBuf> = Vec::new();
    if let Some(p) = std::env::var_os("ABOLOS_PDFIUM") {
        orte.push(PathBuf::from(p));
    }
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
        orte.push(Pdfium::pdfium_platform_library_name_at_path(&dir));
        orte.push(Pdfium::pdfium_platform_library_name_at_path(&dir.join("lib")));
        orte.push(Pdfium::pdfium_platform_library_name_at_path(&dir.join("../lib")));
    }
    // beim Entwickeln: das heruntergeladene Paket (Linux: lib/, Windows: bin/)
    let projekt = Path::new(env!("CARGO_MANIFEST_DIR")).join("third_party/pdfium");
    orte.push(Pdfium::pdfium_platform_library_name_at_path(&projekt.join("lib")));
    orte.push(Pdfium::pdfium_platform_library_name_at_path(&projekt.join("bin")));
    let mut fehler = Vec::new();
    for ort in &orte {
        if ort.exists() {
            match Pdfium::bind_to_library(ort) {
                Ok(b) => return Ok((b, Some(ort.clone()))),
                Err(e) => fehler.push(format!("{}: {e}", ort.display())),
            }
        }
    }
    Pdfium::bind_to_system_library()
        .map(|b| (b, None))
        .map_err(|e| format!("libpdfium.so nicht gefunden (gesucht: {orte:?}; {fehler:?}; System: {e})"))
}

thread_local! {
    /// Zweite Anbindung an dieselbe Bibliothek, nur im PDF-Thread: für PDFium-Aufrufe, die
    /// pdfium-render nicht anbietet (z.B. /Subj von Anmerkungen)
    static ROH: std::cell::OnceCell<Box<dyn PdfiumLibraryBindings>> = const { std::cell::OnceCell::new() };
}

/// Rohe PDFium-Aufrufe im PDF-Thread; None, wenn die zweite Anbindung fehlt
pub(crate) fn mit_roh<T>(f: impl FnOnce(&dyn PdfiumLibraryBindings) -> T) -> Option<T> {
    ROH.with(|r| r.get().map(|b| f(b.as_ref())))
}

fn arbeiter(bindings: Box<dyn PdfiumLibraryBindings>, rx: Receiver<Auftrag>, bereit: Sender<Result<(), String>>) {
    let pdfium = Pdfium::new(bindings);
    let _ = bereit.send(Ok(()));
    let mut stand = Stand { dok: None, nr: 0, rueckgaengig: Vec::new(), geaendert: false };
    let mut eingang: Vec<Auftrag> = Vec::new();
    // laufende Suche: Auftrag, aufbereiteter Suchbegriff und Zahl der schon durchsuchten Seiten
    let mut suche: Option<(SuchAuftrag, Vec<char>, i32)> = None;
    let mut ordner: Option<OrdnerLauf> = None;
    let mut textspeicher = Textspeicher::default();

    let suche_beenden = |suche: &mut Option<(SuchAuftrag, Vec<char>, i32)>| {
        if let Some((alt, ..)) = suche.take() {
            (alt.fertig)(());
        }
    };
    let ordner_beenden = |ordner: &mut Option<OrdnerLauf>, voll: bool| {
        if let Some(alt) = ordner.take() {
            (alt.auftrag.fertig)(voll);
        }
    };

    loop {
        // Ohne laufende Suche blockierend warten, sonst nur abholen, was schon da ist
        if suche.is_none() && ordner.is_none() {
            match rx.recv() {
                Ok(a) => eingang.push(a),
                Err(_) => return,
            }
        }
        loop {
            match rx.try_recv() {
                Ok(a) => eingang.push(a),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        // Öffnen und Suchen in Eingangsreihenfolge, Render-/Link-Aufträge sammeln
        let mut sichtbar = Vec::new();
        for auftrag in eingang.drain(..) {
            match auftrag {
                Auftrag::Oeffnen { pfad, antwort } => {
                    suche_beenden(&mut suche);
                    stand.dok = None;
                    stand.rueckgaengig.clear();
                    stand.geaendert = false;
                    let ergebnis = pdfium
                        .load_pdf_from_file(&pfad, None)
                        .map_err(|e| format!("{}: {e}", pfad.display()))
                        .and_then(|d| {
                            let masse = seitenmasse(&d)?;
                            stand.dok = Some(d);
                            Ok(masse)
                        });
                    stand.nr += 1;
                    let _ = antwort.send(ergebnis.map(|masse| (stand.nr, masse)));
                }
                Auftrag::Suchen(a) => {
                    suche_beenden(&mut suche);
                    let nadel = crate::suche::nadel(&a.text);
                    suche = Some((a, nadel, 0));
                }
                Auftrag::SucheAbbrechen => suche_beenden(&mut suche),
                Auftrag::Ordnersuche(a) => {
                    ordner_beenden(&mut ordner, false);
                    let nadel = crate::suche::nadel(&a.text);
                    ordner = Some(OrdnerLauf { auftrag: a, nadel, datei_nr: 0, offen: None, unlesbar: 0, gefunden: 0 });
                }
                Auftrag::OrdnerAbbrechen => ordner_beenden(&mut ordner, false),
                Auftrag::Ausfuehren(f) => f(&mut stand),
                andere => sichtbar.push(andere),
            }
        }

        // Neueste Wünsche zuerst: das ist, was gerade auf dem Bildschirm steht
        for auftrag in sichtbar.into_iter().rev() {
            match auftrag {
                Auftrag::Rendern { dok: nr, seite, breite_px, drehung, gueltig, fertig } => {
                    let bild = if nr == stand.nr && gueltig() {
                        stand.dok.as_ref().and_then(|d| rendern(d, seite, breite_px)).map(|b| drehen(b, drehung))
                    } else {
                        None
                    };
                    fertig(bild);
                }
                Auftrag::Links { dok: nr, seite, fertig } => {
                    let links = if nr == stand.nr {
                        stand.dok.as_ref().map(|d| links(d, seite)).unwrap_or_default()
                    } else {
                        vec![]
                    };
                    fertig(links);
                }
                _ => unreachable!("Öffnen/Suchen oben behandelt"),
            }
        }

        // Dann eine Seite weitersuchen
        let seiten = stand.dok.as_ref().map(|d| d.pages().len()).unwrap_or(0);
        if let Some((auftrag, nadel, erledigt)) = suche.as_mut() {
            match stand.dok.as_ref() {
                Some(d) if *erledigt < seiten => {
                    let seite = (auftrag.ab_seite.max(0) + *erledigt) % seiten;
                    let treffer = seite_durchsuchen(d, seite, nadel, auftrag.optionen);
                    if !treffer.is_empty() {
                        (auftrag.bei_treffern)(treffer);
                    }
                    *erledigt += 1;
                    if *erledigt % 20 == 0 || *erledigt == seiten {
                        (auftrag.fortschritt)(*erledigt as f32 / seiten as f32);
                    }
                }
                _ => suche_beenden(&mut suche),
            }
        }

        // und ein Stück Ordnersuche (höchstens etwa 15 ms, damit Scrollen flüssig bleibt)
        if let Some(lauf) = ordner.as_mut() {
            match ordner_schritt(&pdfium, lauf, &mut textspeicher) {
                Weiter::Ja => {}
                Weiter::Fertig => ordner_beenden(&mut ordner, false),
                Weiter::Grenze => ordner_beenden(&mut ordner, true),
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Ordnersuche: viele PDFs nacheinander, Seite für Seite zwischen den Render-Aufträgen

struct OrdnerLauf<'a> {
    auftrag: OrdnerAuftrag,
    nadel: Vec<char>,
    datei_nr: usize,
    offen: Option<OffeneDatei<'a>>,
    unlesbar: usize,
    gefunden: usize,
}

struct OffeneDatei<'a> {
    pfad: Arc<PathBuf>,
    dok: PdfDocument<'a>,
    seiten: i32,
    seite: i32,
    /// aufbereiteter Text je Seite, für den Zwischenspeicher
    texte: Vec<String>,
}

enum Weiter {
    Ja,
    Fertig,
    Grenze,
}

/// Seitentexte schon durchsuchter Dateien: Eine zweite Suche im selben Ordner braucht PDFium nicht
/// mehr. Gültig, solange Änderungszeit und Größe der Datei gleich sind; begrenzt auf 256 MB.
#[derive(Default)]
struct Textspeicher {
    dateien: HashMap<PathBuf, (std::time::SystemTime, u64, Arc<Vec<String>>)>,
    reihenfolge: VecDeque<PathBuf>,
    bytes: usize,
}

const TEXTSPEICHER_GRENZE: usize = 256 << 20;

fn dateikennung(pfad: &Path) -> Option<(std::time::SystemTime, u64)> {
    let m = std::fs::metadata(pfad).ok()?;
    Some((m.modified().ok()?, m.len()))
}

impl Textspeicher {
    fn holen(&self, pfad: &Path) -> Option<Arc<Vec<String>>> {
        let (zeit, groesse, texte) = self.dateien.get(pfad)?;
        (dateikennung(pfad)? == (*zeit, *groesse)).then(|| texte.clone())
    }

    fn ablegen(&mut self, pfad: &Path, texte: Vec<String>) {
        let Some((zeit, groesse)) = dateikennung(pfad) else { return };
        let bytes: usize = texte.iter().map(|t| t.len()).sum();
        if let Some((.., alt)) = self.dateien.remove(pfad) {
            self.bytes -= alt.iter().map(|t| t.len()).sum::<usize>();
            self.reihenfolge.retain(|p| p != pfad);
        }
        while self.bytes + bytes > TEXTSPEICHER_GRENZE {
            let Some(alt) = self.reihenfolge.pop_front() else { break };
            if let Some((.., t)) = self.dateien.remove(&alt) {
                self.bytes -= t.iter().map(|t| t.len()).sum::<usize>();
            }
        }
        if bytes <= TEXTSPEICHER_GRENZE {
            self.bytes += bytes;
            self.dateien.insert(pfad.to_path_buf(), (zeit, groesse, Arc::new(texte)));
            self.reihenfolge.push_back(pfad.to_path_buf());
        }
    }
}

/// Fundstellen eines aufbereiteten Seitentexts als Ordner-Treffer
fn ordner_treffer(
    st: &crate::suche::Seitentext,
    nadel: &[char],
    optionen: Suchoptionen,
    datei: &Arc<PathBuf>,
    seite: i32,
) -> Vec<OrdnerTreffer> {
    crate::suche::finden(st, nadel, optionen)
        .into_iter()
        .enumerate()
        .map(|(nr, (a, b))| {
            let k = crate::suche::kontext(&st.zeichen, a, b);
            OrdnerTreffer { datei: datei.clone(), seite, nr, vor: k.vor, wort: k.wort, nach: k.nach }
        })
        .collect()
}

fn ordner_schritt<'a>(pdfium: &'a Pdfium, lauf: &mut OrdnerLauf<'a>, speicher: &mut Textspeicher) -> Weiter {
    let beginn = std::time::Instant::now();
    let mut treffer = Vec::new();
    let mut ergebnis = Weiter::Ja;
    let gesamt = lauf.auftrag.dateien.len();
    let melden = |lauf: &OrdnerLauf, name: &Path| {
        (lauf.auftrag.fortschritt)(OrdnerFortschritt {
            fertig: lauf.datei_nr,
            gesamt,
            unlesbar: lauf.unlesbar,
            datei: name.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        });
    };
    while beginn.elapsed() < std::time::Duration::from_millis(15) {
        if lauf.gefunden + treffer.len() >= lauf.auftrag.grenze {
            ergebnis = Weiter::Grenze;
            break;
        }
        if lauf.offen.is_none() {
            let Some(pfad) = lauf.auftrag.dateien.get(lauf.datei_nr).cloned() else {
                ergebnis = Weiter::Fertig;
                break;
            };
            let datei = Arc::new(pfad);
            // schon einmal gelesen: nur noch Text vergleichen
            if let Some(texte) = speicher.holen(&datei) {
                for (seite, text) in texte.iter().enumerate() {
                    let st = crate::suche::aus_gespeichert(text);
                    treffer.extend(ordner_treffer(&st, &lauf.nadel, lauf.auftrag.optionen, &datei, seite as i32));
                }
                lauf.datei_nr += 1;
                melden(lauf, &datei);
                continue;
            }
            match pdfium.load_pdf_from_file(datei.as_path(), None) {
                Ok(dok) => {
                    let seiten = dok.pages().len();
                    lauf.offen = Some(OffeneDatei { pfad: datei, dok, seiten, seite: 0, texte: Vec::new() });
                }
                Err(_) => {
                    // kaputt oder mit Passwort geschützt
                    lauf.unlesbar += 1;
                    lauf.datei_nr += 1;
                    melden(lauf, &datei);
                    continue;
                }
            }
        }
        let offen = lauf.offen.as_mut().unwrap();
        if offen.seite < offen.seiten {
            let text = offen
                .dok
                .pages()
                .get(offen.seite)
                .ok()
                .and_then(|p| p.text().ok().map(|t| crate::suche::seitentext(&t)));
            let st = text.unwrap_or(crate::suche::Seitentext { zeichen: Vec::new(), index: Vec::new() });
            treffer.extend(ordner_treffer(&st, &lauf.nadel, lauf.auftrag.optionen, &offen.pfad, offen.seite));
            offen.texte.push(st.zeichen.iter().collect());
            offen.seite += 1;
        }
        if offen.seite >= offen.seiten {
            let offen = lauf.offen.take().unwrap();
            speicher.ablegen(&offen.pfad, offen.texte);
            lauf.datei_nr += 1;
            melden(lauf, &offen.pfad);
        }
    }
    if !treffer.is_empty() {
        treffer.truncate(lauf.auftrag.grenze.saturating_sub(lauf.gefunden));
        lauf.gefunden += treffer.len();
        (lauf.auftrag.bei_treffern)(treffer);
    }
    ergebnis
}

fn seitenmasse(d: &PdfDocument) -> Result<Vec<Seitenmass>, String> {
    let rects = d.pages().page_sizes().map_err(|e| format!("Seitengrößen: {e}"))?;
    Ok(rects.iter().map(|r| Seitenmass { breite: r.width().value, hoehe: r.height().value }).collect())
}

fn rendern(d: &PdfDocument, seite: i32, breite_px: i32) -> Option<Bild> {
    let page = d.pages().get(seite).ok()?;
    let config =
        PdfRenderConfig::new().set_target_width(breite_px.max(16)).render_form_data(true).render_annotations(true);
    let bitmap = page.render_with_config(&config).ok()?;
    Some(Bild { breite: bitmap.width(), hoehe: bitmap.height(), rgba: Arc::new(bitmap.as_rgba_bytes()) })
}

/// Gerendertes Bild im Uhrzeigersinn um 90/180/270 Grad drehen (verlustfrei, Pixel umsortieren)
fn drehen(bild: Bild, grad: i32) -> Bild {
    let grad = grad.rem_euclid(360);
    if grad == 0 {
        return bild;
    }
    let (w, h) = (bild.breite as usize, bild.hoehe as usize);
    let quelle = &bild.rgba;
    let mut ziel = vec![0u8; quelle.len()];
    let (nw, nh) = if grad == 180 { (w, h) } else { (h, w) };
    for y in 0..h {
        for x in 0..w {
            let (zx, zy) = match grad {
                90 => (h - 1 - y, x),
                180 => (w - 1 - x, h - 1 - y),
                _ => (y, w - 1 - x), // 270
            };
            let q = (y * w + x) * 4;
            let z = (zy * nw + zx) * 4;
            ziel[z..z + 4].copy_from_slice(&quelle[q..q + 4]);
        }
    }
    Bild { breite: nw as i32, hoehe: nh as i32, rgba: Arc::new(ziel) }
}

/// Sichtbarer Bereich einer Seite: CropBox (sonst MediaBox) in PDF-Koordinaten und die im PDF
/// gespeicherte Drehung (/Rotate). Text- und Link-Koordinaten liegen im ungedrehten Seitenraum,
/// der sichtbare Ausschnitt beginnt oft nicht bei 0 (gescannte, beschnittene PDFs).
#[derive(Clone, Copy)]
pub(crate) struct Rahmen {
    links: f32,
    unten: f32,
    rechts: f32,
    oben: f32,
    drehung: i32,
}

impl Rahmen {
    pub(crate) fn von(page: &PdfPage) -> Rahmen {
        let b = page.boundaries();
        let box_ = b.crop().or_else(|_| b.media()).map(|x| x.bounds).ok();
        let drehung = match page.rotation() {
            Ok(PdfPageRenderRotation::Degrees90) => 90,
            Ok(PdfPageRenderRotation::Degrees180) => 180,
            Ok(PdfPageRenderRotation::Degrees270) => 270,
            _ => 0,
        };
        match box_ {
            Some(r) if r.width().value > 0.0 && r.height().value > 0.0 => Rahmen {
                links: r.left().value,
                unten: r.bottom().value,
                rechts: r.right().value,
                oben: r.top().value,
                drehung,
            },
            _ => {
                // ohne Rahmen: Seitengröße ab Ursprung (bei /Rotate 90/270 sind Breite und Höhe vertauscht)
                let (w, h) = (page.width().value, page.height().value);
                let (w, h) = if drehung % 180 == 0 { (w, h) } else { (h, w) };
                Rahmen { links: 0.0, unten: 0.0, rechts: w, oben: h, drehung }
            }
        }
    }

    /// PDF-Rechteck (Ursprung unten links, Punkte) relativ zur angezeigten Seite (oben links, 0..1)
    pub(crate) fn relativ(&self, r: PdfRect) -> RelRect {
        let (b, h) = (self.rechts - self.links, self.oben - self.unten);
        let roh = RelRect {
            x: (r.left().value - self.links) / b,
            y: (self.oben - r.top().value) / h,
            b: (r.right().value - r.left().value) / b,
            h: (r.top().value - r.bottom().value) / h,
        };
        crate::zustand::gedreht(roh, self.drehung)
    }

    /// Umkehrung: relativer Punkt der angezeigten Seite (0..1, oben links) in PDF-Punkte
    pub(crate) fn punkt(&self, x: f32, y: f32) -> (f32, f32) {
        let (x, y) = crate::zustand::ungedreht(x as f64, y as f64, self.drehung);
        let (x, y) = (x as f32, y as f32);
        (self.links + x * (self.rechts - self.links), self.oben - y * (self.oben - self.unten))
    }

    /// Höhe im PDF (Punkte) als relative Höhe der angezeigten Seite; None, wenn quer gedreht
    pub(crate) fn relative_hoehe(&self, y: f32) -> Option<f32> {
        let rel = ((self.oben - y) / (self.oben - self.unten)).clamp(0.0, 1.0);
        match self.drehung {
            0 => Some(rel),
            180 => Some(1.0 - rel),
            _ => None,
        }
    }
}

fn links(d: &PdfDocument, seite: i32) -> Vec<Link> {
    let Ok(page) = d.pages().get(seite) else { return vec![] };
    let rahmen = Rahmen::von(&page);
    let mut ergebnis = Vec::new();
    for link in page.links().iter() {
        let Ok(rect) = link.rect() else { continue };
        let ziel = if let Some(dest) = link.destination() {
            ziel_aus_destination(d, &dest)
        } else if let Some(action) = link.action() {
            match &action {
                PdfAction::LocalDestination(lokal) => {
                    lokal.destination().ok().and_then(|dest| ziel_aus_destination(d, &dest))
                }
                PdfAction::Uri(u) => u.uri().ok().map(|uri| LinkZiel::Extern { uri }),
                _ => None,
            }
        } else {
            None
        };
        if let Some(ziel) = ziel {
            ergebnis.push(Link { rect: rahmen.relativ(rect), ziel });
        }
    }
    ergebnis
}

pub(crate) fn ziel_aus_destination(d: &PdfDocument, dest: &PdfDestination) -> Option<LinkZiel> {
    let seite = dest.page_index().ok()?;
    let rahmen = d.pages().get(seite).ok().map(|p| Rahmen::von(&p))?;
    let y_pt = match dest.view_settings().ok()? {
        PdfDestinationViewSettings::SpecificCoordinatesAndZoom(_, y, _) => y.map(|v| v.value),
        PdfDestinationViewSettings::FitPageHorizontallyToWindow(y) => y.map(|v| v.value),
        PdfDestinationViewSettings::FitBoundsHorizontallyToWindow(y) => y.map(|v| v.value),
        PdfDestinationViewSettings::FitPageToRectangle(r) => Some(r.top().value),
        _ => None,
    };
    let y = y_pt.and_then(|y| rahmen.relative_hoehe(y));
    Some(LinkZiel::Intern { seite, y })
}

/// Alle Treffer einer Seite mit Rechtecken und Zusammenhang
pub(crate) fn seite_durchsuchen(d: &PdfDocument, seite: i32, nadel: &[char], optionen: Suchoptionen) -> Vec<Treffer> {
    let Ok(page) = d.pages().get(seite) else { return vec![] };
    let rahmen = Rahmen::von(&page);
    let Ok(seitentext) = page.text() else { return vec![] };
    let st = crate::suche::seitentext(&seitentext);
    let mut treffer = Vec::new();
    for (a, b) in crate::suche::finden(&st, nadel, optionen) {
        let Some((start, anzahl)) = crate::suche::zeichenbereich(&st, a, b) else { continue };
        let rects: Vec<RelRect> =
            seitentext.segments_subset(start, anzahl).iter().map(|s| rahmen.relativ(s.bounds())).collect();
        if rects.is_empty() {
            continue;
        }
        let k = crate::suche::kontext(&st.zeichen, a, b);
        treffer.push(Treffer { seite, rects, vor: k.vor, wort: k.wort, nach: k.nach });
    }
    treffer
}

// ---------------------------------------------------------------------------------------------
// Bild-Zwischenspeicher: gerenderte Seiten, nach Speicherbedarf begrenzt

/// Dokument, Seite, angezeigte Breite in Pixeln, Drehung
pub type Schluessel = (u64, i32, i32, i32);

pub struct Cache {
    eintraege: HashMap<Schluessel, (Arc<Vec<u8>>, i32, i32)>,
    reihenfolge: VecDeque<Schluessel>,
    bytes: usize,
    grenze: usize,
    in_arbeit: std::collections::HashSet<Schluessel>,
    /// nach einer Bearbeitung veraltet: nur noch Platzhalter, bis die neue Fassung da ist
    veraltet: std::collections::HashSet<Schluessel>,
}

pub static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

pub fn cache() -> &'static Mutex<Cache> {
    CACHE.get_or_init(|| {
        Mutex::new(Cache {
            eintraege: HashMap::new(),
            reihenfolge: VecDeque::new(),
            bytes: 0,
            grenze: 768 << 20,
            in_arbeit: Default::default(),
            veraltet: Default::default(),
        })
    })
}

impl Cache {
    pub fn holen(&mut self, key: Schluessel) -> Option<(Arc<Vec<u8>>, i32, i32)> {
        if self.veraltet.contains(&key) {
            return None;
        }
        let e = self.eintraege.get(&key)?.clone();
        if let Some(pos) = self.reihenfolge.iter().position(|k| *k == key) {
            self.reihenfolge.remove(pos);
        }
        self.reihenfolge.push_back(key);
        Some(e)
    }

    /// Irgendeine gecachte Breite derselben Seite, als Platzhalter während neu gerendert wird.
    pub fn naechstbeste(&self, dok: u64, seite: i32, breite: i32, drehung: i32) -> Option<(Arc<Vec<u8>>, i32, i32)> {
        self.eintraege
            .iter()
            .filter(|((d, s, _, r), _)| *d == dok && *s == seite && *r == drehung)
            .min_by_key(|((_, _, b, _), _)| (b - breite).abs())
            .map(|(_, v)| v.clone())
    }

    pub fn ablegen(&mut self, key: Schluessel, bild: Bild) {
        self.in_arbeit.remove(&key);
        self.veraltet.remove(&key);
        if let Some((alt, _, _)) = self.eintraege.remove(&key) {
            self.bytes -= alt.len();
            self.reihenfolge.retain(|k| *k != key);
        }
        self.bytes += bild.rgba.len();
        self.eintraege.insert(key, (bild.rgba, bild.breite, bild.hoehe));
        self.reihenfolge.push_back(key);
        while self.bytes > self.grenze {
            let Some(alt) = self.reihenfolge.pop_front() else { break };
            if let Some((daten, _, _)) = self.eintraege.remove(&alt) {
                self.bytes -= daten.len();
            }
        }
    }

    /// true, wenn der Auftrag neu ist (nicht schon unterwegs)
    pub fn vormerken(&mut self, key: Schluessel) -> bool {
        self.in_arbeit.insert(key)
    }

    /// Seite wurde bearbeitet: alle gerenderten Fassungen neu anfordern
    pub fn seite_verwerfen(&mut self, dok: u64, seite: i32) {
        let alle: Vec<Schluessel> = self.eintraege.keys().filter(|k| k.0 == dok && k.1 == seite).copied().collect();
        self.veraltet.extend(alle);
        self.in_arbeit.retain(|k| !(k.0 == dok && k.1 == seite));
    }

    pub fn fehlgeschlagen(&mut self, key: Schluessel) {
        self.in_arbeit.remove(&key);
    }
}
