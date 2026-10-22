// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bearbeiten im PDF-Thread: Anmerkungen anlegen, auflisten, ändern, löschen; Gliederung; Speichern.
//! Alles sind echte PDF-Anmerkungen, andere Programme (Acrobat, Okular, ...) zeigen sie ebenso an.
//! Koordinaten kommen relativ zur gerenderten Seite (0..1, oben links, ohne Ansichtsdrehung).

use std::path::{Path, PathBuf};

use chrono::Utc;
use pdfium_render::prelude::*;

use crate::engine::{LinkZiel, Rahmen, RelRect, Stand, ziel_aus_destination};

pub type Farbe = [u8; 4];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Markierung {
    Hervorheben,
    Unterstreichen,
    Durchstreichen,
    Wellenlinie,
}

pub enum Bearbeitung {
    /// Text zwischen zwei Punkten markieren
    Markieren {
        seite: i32,
        art: Markierung,
        von: (f32, f32),
        bis: (f32, f32),
        farbe: Farbe,
    },
    /// Freihand-Striche; breite in Punkten
    Freihand {
        seite: i32,
        striche: Vec<Vec<(f32, f32)>>,
        farbe: Farbe,
        breite: f32,
    },
    Form {
        seite: i32,
        ellipse: bool,
        a: (f32, f32),
        b: (f32, f32),
        farbe: Farbe,
        breite: f32,
    },
    /// gerade Linie von a nach b, mit Pfeilspitze bei b
    Linie {
        seite: i32,
        a: (f32, f32),
        b: (f32, f32),
        pfeil: bool,
        farbe: Farbe,
        breite: f32,
    },
    Notiz {
        seite: i32,
        punkt: (f32, f32),
        text: String,
        farbe: Farbe,
    },
    Textfeld {
        seite: i32,
        a: (f32, f32),
        b: (f32, f32),
        text: String,
        farbe: Farbe,
        groesse: f32,
    },
    Bild {
        seite: i32,
        a: (f32, f32),
        b: (f32, f32),
        pfad: PathBuf,
    },
}

pub struct Anmerkung {
    pub seite: i32,
    pub index: usize,
    pub typ: String,
    pub rect: RelRect,
    pub farbe: Option<Farbe>,
    pub inhalt: String,
    pub autor: String,
    pub datum: String,
}

pub struct Gliederungspunkt {
    pub titel: String,
    pub tiefe: u32,
    pub seite: i32,
    pub y: Option<f32>,
}

fn pdf_farbe(f: Farbe) -> PdfColor {
    PdfColor::new(f[0], f[1], f[2], f[3])
}

fn fehler(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Benutzername als Autor neuer Anmerkungen (Linux: USER, Windows: USERNAME)
pub(crate) fn autor() -> String {
    std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "Abolos PDF".into())
}

/// Rechteck aus zwei Punkten (PDF-Koordinaten), richtig sortiert
fn rechteck(a: (f32, f32), b: (f32, f32)) -> PdfRect {
    PdfRect::new_from_values(a.1.min(b.1), a.0.min(b.0), a.1.max(b.1), a.0.max(b.0))
}

/// Zeichenbereich zwischen zwei Punkten: Rechtecke (PDF-Koordinaten) und Text
fn zeichenbereich(page: &PdfPage, rahmen: &Rahmen, von: (f32, f32), bis: (f32, f32)) -> Option<(Vec<PdfRect>, String)> {
    let text = page.text().ok()?;
    let chars = text.chars();
    let naechstes = |p: (f32, f32)| {
        let (x, y) = rahmen.punkt(p.0, p.1);
        // erst eng, dann großzügiger suchen (Zwischenräume, Zeilenenden)
        [4.0, 12.0, 30.0].iter().find_map(|t| {
            chars
                .get_char_near_point(PdfPoints::new(x), PdfPoints::new(*t), PdfPoints::new(y), PdfPoints::new(*t))
                .map(|c| c.index())
        })
    };
    let (a, b) = (naechstes(von)?, naechstes(bis)?);
    let (a, b) = (a.min(b), a.max(b));
    let segmente = text.segments_subset(a, b - a + 1);
    let rects: Vec<PdfRect> = segmente.iter().map(|s| s.bounds()).collect();
    let inhalt = segmente.iter().map(|s| s.text().trim().to_string()).collect::<Vec<_>>().join(" ");
    (!rects.is_empty()).then_some((rects, inhalt))
}

/// Vorschau für das Markieren: Rechtecke relativ zur Seite und der markierte Text
pub fn text_auswahl(d: &PdfDocument, seite: i32, von: (f32, f32), bis: (f32, f32)) -> (Vec<RelRect>, String) {
    let Ok(page) = d.pages().get(seite) else { return (vec![], String::new()) };
    let rahmen = Rahmen::von(&page);
    match zeichenbereich(&page, &rahmen, von, bis) {
        Some((rects, text)) => (rects.iter().map(|r| rahmen.relativ(*r)).collect(), text),
        None => (vec![], String::new()),
    }
}

/// QuadPoints in der Reihenfolge, die PDFium und Acrobat erwarten: oben links, oben rechts,
/// unten links, unten rechts. (PdfQuadPoints::from_rect liefert unten links zuerst; PDFium
/// erzeugt daraus eine Hervorhebung der Breite null.)
fn quad(r: &PdfRect) -> PdfQuadPoints {
    PdfQuadPoints::new(r.left(), r.top(), r.right(), r.top(), r.left(), r.bottom(), r.right(), r.bottom())
}

/// gemeinsame Angaben jeder neuen Anmerkung
fn stempeln<T: PdfPageAnnotationCommon>(a: &mut T) -> Result<(), String> {
    let jetzt = Utc::now();
    a.set_creator(&autor()).map_err(fehler)?;
    a.set_creation_date(jetzt).map_err(fehler)?;
    a.set_modification_date(jetzt).map_err(fehler)?;
    Ok(())
}

macro_rules! markup {
    ($annots:expr, $erzeugen:ident, $rects:expr, $farbe:expr, $text:expr, $huelle:expr) => {{
        let mut a = $annots.$erzeugen().map_err(fehler)?;
        for r in $rects {
            a.attachment_points_mut().create_attachment_point_at_end(quad(r)).map_err(fehler)?;
        }
        a.set_bounds($huelle).map_err(fehler)?;
        a.set_stroke_color($farbe).map_err(fehler)?;
        a.set_contents($text).map_err(fehler)?;
        stempeln(&mut a)?;
    }};
}

/// Neue Anmerkung anlegen; Rückgabe: geänderte Seite
pub fn anlegen(stand: &mut Stand, b: Bearbeitung) -> Result<i32, String> {
    let Some(d) = stand.dok.as_mut() else { return Err("kein Dokument offen".into()) };
    let seite = match &b {
        Bearbeitung::Markieren { seite, .. }
        | Bearbeitung::Freihand { seite, .. }
        | Bearbeitung::Form { seite, .. }
        | Bearbeitung::Linie { seite, .. }
        | Bearbeitung::Notiz { seite, .. }
        | Bearbeitung::Textfeld { seite, .. }
        | Bearbeitung::Bild { seite, .. } => *seite,
    };
    // Schrift für Textfelder vorab holen (braucht das Dokument veränderlich)
    let schrift = d.fonts_mut().helvetica();
    let d: &PdfDocument = d;
    let mut page = d.pages().get(seite).map_err(fehler)?;
    let rahmen = Rahmen::von(&page);
    let punkt = |p: (f32, f32)| rahmen.punkt(p.0, p.1);
    // Betreff (/Subj): Zeichnungen und Stempel unterscheiden sich sonst nicht (Liste, andere Reader)
    let betreff = match &b {
        Bearbeitung::Freihand { .. } => Some("Freihand"),
        Bearbeitung::Form { ellipse: true, .. } => Some("Ellipse"),
        Bearbeitung::Form { ellipse: false, .. } => Some("Rechteck"),
        Bearbeitung::Linie { pfeil: true, .. } => Some("Pfeil"),
        Bearbeitung::Linie { pfeil: false, .. } => Some("Linie"),
        Bearbeitung::Textfeld { .. } => Some("Textfeld"),
        Bearbeitung::Bild { .. } => Some("Bild"),
        _ => None,
    };

    match b {
        Bearbeitung::Markieren { art, von, bis, farbe, .. } => {
            let (rects, text) = zeichenbereich(&page, &rahmen, von, bis).ok_or("an dieser Stelle ist kein Text")?;
            let huelle = rects.iter().skip(1).fold(rects[0], |h, r| {
                PdfRect::new_from_values(
                    h.bottom().value.min(r.bottom().value),
                    h.left().value.min(r.left().value),
                    h.top().value.max(r.top().value),
                    h.right().value.max(r.right().value),
                )
            });
            let farbe = pdf_farbe(farbe);
            let annots = page.annotations_mut();
            match art {
                Markierung::Hervorheben => markup!(annots, create_highlight_annotation, &rects, farbe, &text, huelle),
                Markierung::Unterstreichen => {
                    markup!(annots, create_underline_annotation, &rects, farbe, &text, huelle)
                }
                Markierung::Durchstreichen => {
                    markup!(annots, create_strikeout_annotation, &rects, farbe, &text, huelle)
                }
                Markierung::Wellenlinie => markup!(annots, create_squiggly_annotation, &rects, farbe, &text, huelle),
            }
        }
        Bearbeitung::Freihand { striche, farbe, breite, .. } => {
            let striche: Vec<Vec<(f32, f32)>> =
                striche.into_iter().filter(|s| s.len() >= 2).map(|s| s.into_iter().map(punkt).collect()).collect();
            if striche.is_empty() {
                return Err("Strich zu kurz".into());
            }
            let alle = striche.iter().flatten();
            let rand = breite;
            let huelle = PdfRect::new_from_values(
                alle.clone().map(|p| p.1).fold(f32::MAX, f32::min) - rand,
                alle.clone().map(|p| p.0).fold(f32::MAX, f32::min) - rand,
                alle.clone().map(|p| p.1).fold(f32::MIN, f32::max) + rand,
                alle.map(|p| p.0).fold(f32::MIN, f32::max) + rand,
            );
            let mut ink = page.annotations_mut().create_ink_annotation().map_err(fehler)?;
            ink.set_bounds(huelle).map_err(fehler)?;
            // Farbe der Anmerkung (/C) vor den Objekten setzen: danach würde pdfium-render die Objekte
            // umfärben, und das stürzt in PDFium ab
            ink.set_stroke_color(pdf_farbe(farbe)).map_err(fehler)?;
            for strich in &striche {
                let mut pfad = PdfPagePathObject::new(
                    d,
                    PdfPoints::new(strich[0].0),
                    PdfPoints::new(strich[0].1),
                    Some(pdf_farbe(farbe)),
                    Some(PdfPoints::new(breite)),
                    None,
                )
                .map_err(fehler)?;
                for p in &strich[1..] {
                    pfad.line_to(PdfPoints::new(p.0), PdfPoints::new(p.1)).map_err(fehler)?;
                }
                ink.objects_mut().add_path_object(pfad).map_err(fehler)?;
            }
            stempeln(&mut ink)?;
        }
        Bearbeitung::Form { ellipse, a, b, farbe, breite, .. } => {
            let r = rechteck(punkt(a), punkt(b));
            if r.width().value < 2.0 || r.height().value < 2.0 {
                return Err("Form zu klein".into());
            }
            let farbe = pdf_farbe(farbe);
            let breite = Some(PdfPoints::new(breite));
            if ellipse {
                let mut form = page.annotations_mut().create_ink_annotation().map_err(fehler)?;
                form.set_bounds(r).map_err(fehler)?;
                form.set_stroke_color(farbe).map_err(fehler)?; // vor den Objekten, siehe Freihand
                form.objects_mut().create_path_object_ellipse(r, Some(farbe), breite, None).map_err(fehler)?;
                stempeln(&mut form)?;
            } else {
                // Rechteck als Zeichnung: nur so lässt sich die Strichstärke festlegen
                let mut form = page.annotations_mut().create_ink_annotation().map_err(fehler)?;
                form.set_bounds(r).map_err(fehler)?;
                form.set_stroke_color(farbe).map_err(fehler)?; // vor den Objekten, siehe Freihand
                form.objects_mut().create_path_object_rect(r, Some(farbe), breite, None).map_err(fehler)?;
                stempeln(&mut form)?;
            }
        }
        Bearbeitung::Linie { a, b, pfeil, farbe, breite, .. } => {
            let (p0, p1) = (punkt(a), punkt(b));
            let (dx, dy) = (p1.0 - p0.0, p1.1 - p0.1);
            let laenge = dx.hypot(dy);
            if laenge < 2.0 {
                return Err("Linie zu kurz".into());
            }
            let (ux, uy) = (dx / laenge, dy / laenge);
            // Pfeilspitze: wächst mit der Strichstärke, höchstens 60 % der Linie
            let spitze = (3.5 * breite).max(7.0).min(laenge * 0.6);
            let halb = spitze * 0.45;
            let basis = (p1.0 - ux * spitze, p1.1 - uy * spitze);
            let ecken = [(basis.0 - uy * halb, basis.1 + ux * halb), (basis.0 + uy * halb, basis.1 - ux * halb)];
            // die Linie endet in der Spitze, damit ihr Ende bei dicken Strichen nicht übersteht
            let ende = if pfeil { (basis.0 + ux * spitze * 0.2, basis.1 + uy * spitze * 0.2) } else { p1 };
            let mut punkte = vec![p0, p1];
            if pfeil {
                punkte.extend(ecken);
            }
            let rand = breite + 1.0;
            let huelle = PdfRect::new_from_values(
                punkte.iter().map(|p| p.1).fold(f32::MAX, f32::min) - rand,
                punkte.iter().map(|p| p.0).fold(f32::MAX, f32::min) - rand,
                punkte.iter().map(|p| p.1).fold(f32::MIN, f32::max) + rand,
                punkte.iter().map(|p| p.0).fold(f32::MIN, f32::max) + rand,
            );
            let farbe = pdf_farbe(farbe);
            let mut ink = page.annotations_mut().create_ink_annotation().map_err(fehler)?;
            ink.set_bounds(huelle).map_err(fehler)?;
            ink.set_stroke_color(farbe).map_err(fehler)?; // vor den Objekten, siehe Freihand
            let mut linie = PdfPagePathObject::new(
                d,
                PdfPoints::new(p0.0),
                PdfPoints::new(p0.1),
                Some(farbe),
                Some(PdfPoints::new(breite)),
                None,
            )
            .map_err(fehler)?;
            linie.line_to(PdfPoints::new(ende.0), PdfPoints::new(ende.1)).map_err(fehler)?;
            ink.objects_mut().add_path_object(linie).map_err(fehler)?;
            if pfeil {
                let mut kopf =
                    PdfPagePathObject::new(d, PdfPoints::new(p1.0), PdfPoints::new(p1.1), None, None, Some(farbe))
                        .map_err(fehler)?;
                for e in ecken {
                    kopf.line_to(PdfPoints::new(e.0), PdfPoints::new(e.1)).map_err(fehler)?;
                }
                kopf.close_path().map_err(fehler)?;
                ink.objects_mut().add_path_object(kopf).map_err(fehler)?;
            }
            stempeln(&mut ink)?;
        }
        Bearbeitung::Notiz { punkt: p, text, farbe, .. } => {
            let (x, y) = punkt(p);
            let mut notiz = page.annotations_mut().create_text_annotation(&text).map_err(fehler)?;
            notiz.set_bounds(PdfRect::new_from_values(y - 20.0, x, y, x + 20.0)).map_err(fehler)?;
            notiz.set_stroke_color(pdf_farbe(farbe)).map_err(fehler)?;
            stempeln(&mut notiz)?;
        }
        Bearbeitung::Textfeld { a, b, text, farbe, groesse, .. } => {
            let r = rechteck(punkt(a), punkt(b));
            // als Stempel mit Textobjekten: PDFium zeichnet FreeText-Anmerkungen ohne eigene Darstellung nicht
            let mut feld = page.annotations_mut().create_stamp_annotation().map_err(fehler)?;
            feld.set_bounds(r).map_err(fehler)?;
            feld.set_contents(&text).map_err(fehler)?;
            // heller Hintergrund mit dünnem Rahmen: auf bedrucktem Text lesbar
            feld.objects_mut()
                .create_path_object_rect(
                    r,
                    Some(pdf_farbe(farbe)),
                    Some(PdfPoints::new(0.6)),
                    Some(PdfColor::new(255, 255, 240, 230)),
                )
                .map_err(fehler)?;
            // Sichtbarkeit in jedem Reader: Text als Objekte in die Darstellung der Anmerkung
            let zeichen_je_zeile = ((r.width().value / (groesse * 0.5)).floor() as usize).max(1);
            let mut y = r.top().value - groesse;
            for zeile in umbrechen(&text, zeichen_je_zeile) {
                if y < r.bottom().value - groesse {
                    break;
                }
                // Farbe vor dem Einfügen setzen: danach wirkt sie nicht mehr auf die Darstellung
                let mut obj = PdfPageTextObject::new(d, zeile, schrift, PdfPoints::new(groesse)).map_err(fehler)?;
                obj.set_fill_color(pdf_farbe(farbe)).map_err(fehler)?;
                obj.translate(PdfPoints::new(r.left().value + 2.0), PdfPoints::new(y)).map_err(fehler)?;
                feld.objects_mut().add_text_object(obj).map_err(fehler)?;
                y -= groesse * 1.25;
            }
            stempeln(&mut feld)?;
        }
        Bearbeitung::Bild { a, b, pfad, .. } => {
            let bild = image::open(&pfad).map_err(|e| format!("{}: {e}", pfad.display()))?;
            let r = rechteck(punkt(a), punkt(b));
            // Seitenverhältnis halten, im gezogenen Rahmen zentrieren
            let (bw, bh) = (bild.width() as f32, bild.height() as f32);
            let massstab = (r.width().value / bw).min(r.height().value / bh);
            let (w, h) = (bw * massstab, bh * massstab);
            let x = r.left().value + (r.width().value - w) / 2.0;
            let y = r.bottom().value + (r.height().value - h) / 2.0;
            let mut stempel = page.annotations_mut().create_stamp_annotation().map_err(fehler)?;
            stempel.set_bounds(PdfRect::new_from_values(y, x, y + h, x + w)).map_err(fehler)?;
            stempel
                .objects_mut()
                .create_image_object(
                    PdfPoints::new(x),
                    PdfPoints::new(y),
                    &bild,
                    Some(PdfPoints::new(w)),
                    Some(PdfPoints::new(h)),
                )
                .map_err(fehler)?;
            stempel
                .set_contents(&pfad.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())
                .map_err(fehler)?;
            stempeln(&mut stempel)?;
        }
    }
    let index = page.annotations().len().saturating_sub(1);
    if let Some(betreff) = betreff {
        betreff_setzen(&page, index, betreff);
    }
    stand.rueckgaengig.push((seite, index));
    stand.geaendert = true;
    Ok(seite)
}

/// /Subj einer Anmerkung setzen (pdfium-render kann das nicht)
fn betreff_setzen(page: &PdfPage, index: usize, text: &str) {
    crate::engine::mit_roh(|b| {
        let seite = b.get_handle_from_page(page);
        // SAFETY: gültiger Seiten-Handle; der Anmerkungs-Handle wird gleich wieder geschlossen
        unsafe {
            let a = b.FPDFPage_GetAnnot(seite, index as std::os::raw::c_int);
            if !a.is_null() {
                b.FPDFAnnot_SetStringValue_str(a, "Subj", text);
                b.FPDFPage_CloseAnnot(a);
            }
        }
    });
}

/// /Subj einer Anmerkung, falls vorhanden
fn betreff(page: &PdfPage, index: usize) -> Option<String> {
    crate::engine::mit_roh(|b| {
        let seite = b.get_handle_from_page(page);
        // SAFETY: wie oben; der Puffer hat die von PDFium genannte Länge in Bytes
        unsafe {
            let a = b.FPDFPage_GetAnnot(seite, index as std::os::raw::c_int);
            if a.is_null() {
                return None;
            }
            let laenge = b.FPDFAnnot_GetStringValue(a, "Subj", std::ptr::null_mut(), 0) as usize;
            let mut text = None;
            if laenge > 2 {
                let mut puffer = vec![0u8; laenge];
                b.FPDFAnnot_GetStringValue(a, "Subj", puffer.as_mut_ptr() as *mut _, laenge as _);
                text = b.get_string_from_pdfium_utf16le_bytes(puffer).filter(|t| !t.trim().is_empty());
            }
            b.FPDFPage_CloseAnnot(a);
            text
        }
    })
    .flatten()
}

fn umbrechen(text: &str, breite: usize) -> Vec<String> {
    let mut zeilen = Vec::new();
    for absatz in text.lines() {
        let mut zeile = String::new();
        for wort in absatz.split_whitespace() {
            if !zeile.is_empty() && zeile.chars().count() + 1 + wort.chars().count() > breite {
                zeilen.push(std::mem::take(&mut zeile));
            }
            if !zeile.is_empty() {
                zeile.push(' ');
            }
            zeile.push_str(wort);
        }
        zeilen.push(zeile);
    }
    zeilen
}

pub fn loeschen(stand: &mut Stand, seite: i32, index: usize) -> Result<(), String> {
    let Some(d) = stand.dok.as_ref() else { return Err("kein Dokument offen".into()) };
    let mut page = d.pages().get(seite).map_err(fehler)?;
    let annots = page.annotations_mut();
    let a = annots.get(index).map_err(fehler)?;
    annots.delete_annotation(a).map_err(fehler)?;
    // Rückgängig-Liste anpassen: dahinterliegende Indizes rücken nach
    stand.rueckgaengig.retain(|e| *e != (seite, index));
    for e in stand.rueckgaengig.iter_mut() {
        if e.0 == seite && e.1 > index {
            e.1 -= 1;
        }
    }
    stand.geaendert = true;
    Ok(())
}

/// Letzte selbst angelegte Anmerkung entfernen; Rückgabe: geänderte Seite
pub fn rueckgaengig(stand: &mut Stand) -> Result<Option<i32>, String> {
    let Some((seite, index)) = stand.rueckgaengig.last().copied() else { return Ok(None) };
    loeschen(stand, seite, index)?;
    Ok(Some(seite))
}

pub fn inhalt_setzen(stand: &mut Stand, seite: i32, index: usize, text: &str) -> Result<(), String> {
    let Some(d) = stand.dok.as_ref() else { return Err("kein Dokument offen".into()) };
    let page = d.pages().get(seite).map_err(fehler)?;
    let mut a = page.annotations().get(index).map_err(fehler)?;
    a.set_contents(text).map_err(fehler)?;
    a.set_modification_date(Utc::now()).map_err(fehler)?;
    stand.geaendert = true;
    Ok(())
}

fn typ_name(t: PdfPageAnnotationType) -> Option<&'static str> {
    Some(match t {
        PdfPageAnnotationType::Text => "Notiz",
        PdfPageAnnotationType::FreeText => "Textfeld",
        PdfPageAnnotationType::Line => "Linie",
        PdfPageAnnotationType::Square => "Rechteck",
        PdfPageAnnotationType::Circle => "Ellipse",
        PdfPageAnnotationType::Polygon => "Vieleck",
        PdfPageAnnotationType::Polyline => "Linienzug",
        PdfPageAnnotationType::Highlight => "Hervorhebung",
        PdfPageAnnotationType::Underline => "Unterstreichung",
        PdfPageAnnotationType::Squiggly => "Wellenlinie",
        PdfPageAnnotationType::Strikeout => "Durchstreichung",
        PdfPageAnnotationType::Stamp => "Bild/Stempel",
        PdfPageAnnotationType::Caret => "Einfügemarke",
        PdfPageAnnotationType::Ink => "Zeichnung",
        PdfPageAnnotationType::FileAttachment => "Dateianhang",
        PdfPageAnnotationType::Redacted => "Schwärzung",
        // Links, Formularfelder, Popups usw. sind keine Anmerkungen im Sinne der Liste
        _ => return None,
    })
}

/// PDF-Datum ("D:20261022143000+00'00'", Zeitzone optional) in Ortszeit: "22.10.2026 16:30"
fn datum_lesbar(d: &str) -> String {
    use chrono::{FixedOffset, Local, NaiveDateTime, TimeZone};
    let z = d.trim().trim_start_matches("D:");
    let ziffern: String = z.chars().take_while(|c| c.is_ascii_digit()).collect();
    if ziffern.len() < 12 {
        return d.to_string();
    }
    let voll = format!("{:0<14}", &ziffern[..ziffern.len().min(14)]); // Sekunden dürfen fehlen
    let Ok(naiv) = NaiveDateTime::parse_from_str(&voll, "%Y%m%d%H%M%S") else { return d.to_string() };
    // Zeitzone: Z, +hh'mm' oder -hh'mm'; ohne Angabe gilt Ortszeit
    let rest = &z[ziffern.len()..];
    let versatz = match rest.chars().next() {
        Some('Z') => Some(0),
        Some(v @ ('+' | '-')) => {
            let zahlen: Vec<i32> = rest[1..]
                .split(|c: char| !c.is_ascii_digit())
                .filter(|t| !t.is_empty())
                .map(|t| t.parse().unwrap_or(0))
                .collect();
            let sekunden = zahlen.first().copied().unwrap_or(0) * 3600 + zahlen.get(1).copied().unwrap_or(0) * 60;
            Some(if v == '-' { -sekunden } else { sekunden })
        }
        _ => None,
    };
    let ortszeit = match versatz.and_then(FixedOffset::east_opt) {
        Some(o) => o.from_local_datetime(&naiv).single().map(|t| t.with_timezone(&Local)),
        None => Local.from_local_datetime(&naiv).single(),
    };
    ortszeit.map(|t| t.format("%d.%m.%Y %H:%M").to_string()).unwrap_or_else(|| d.to_string())
}

/// Farbe einer Anmerkung, ohne pdfium-renders stroke_color()/fill_color(): die behandeln bei
/// Anmerkungen mit eigener Darstellung den Anmerkungs-Handle als Seitenobjekt, und PDFium stürzt ab.
/// Sicher lesbar sind nur die Objekte von Zeichnungen und Stempeln (Strich- bzw. Textfarbe).
fn sichere_farbe(a: &PdfPageAnnotation) -> Option<Farbe> {
    if !matches!(a.annotation_type(), PdfPageAnnotationType::Ink | PdfPageAnnotationType::Stamp) {
        return None;
    }
    a.objects().iter().find_map(|o| {
        let c = match o.object_type() {
            PdfPageObjectType::Path => o.stroke_color().ok(),
            PdfPageObjectType::Text => o.fill_color().ok(),
            _ => None,
        }?;
        Some([c.red(), c.green(), c.blue(), c.alpha()])
    })
}

pub fn anmerkungen(d: &PdfDocument) -> Vec<Anmerkung> {
    let mut liste = Vec::new();
    for (seite, page) in d.pages().iter().enumerate() {
        let rahmen = Rahmen::von(&page);
        for (index, a) in page.annotations().iter().enumerate() {
            let Some(mut typ) = typ_name(a.annotation_type()) else { continue };
            if a.annotation_type() == PdfPageAnnotationType::Stamp {
                // eigene Textfelder sind Stempel mit Textobjekten
                let objekte = a.objects();
                let hat_bild = objekte.iter().any(|o| o.object_type() == PdfPageObjectType::Image);
                let hat_text = objekte.iter().any(|o| o.object_type() == PdfPageObjectType::Text);
                typ = if hat_bild {
                    "Bild"
                } else if hat_text {
                    "Textfeld"
                } else {
                    "Stempel"
                };
            }
            // Zeichnungen und Stempel: der Betreff sagt, was es ist (Linie, Pfeil, Rechteck, ...)
            let typ = match a.annotation_type() {
                PdfPageAnnotationType::Ink | PdfPageAnnotationType::Stamp => {
                    betreff(&page, index).unwrap_or_else(|| typ.to_string())
                }
                _ => typ.to_string(),
            };
            let Ok(bounds) = a.bounds() else { continue };
            let farbe = sichere_farbe(&a);
            liste.push(Anmerkung {
                seite: seite as i32,
                index,
                typ,
                rect: rahmen.relativ(bounds),
                farbe,
                inhalt: a.contents().unwrap_or_default(),
                autor: a.creator().unwrap_or_default(),
                datum: a
                    .modification_date()
                    .or_else(|| a.creation_date())
                    .map(|s| datum_lesbar(&s))
                    .unwrap_or_default(),
            });
        }
    }
    liste
}

pub fn gliederung(d: &PdfDocument) -> Vec<Gliederungspunkt> {
    fn sammeln(d: &PdfDocument, erstes: Option<PdfBookmark>, tiefe: u32, aus: &mut Vec<Gliederungspunkt>) {
        let mut aktuell = erstes;
        while let Some(b) = aktuell {
            let ziel = b.destination().and_then(|dest| ziel_aus_destination(d, &dest)).or_else(|| {
                b.action().and_then(|a| match a {
                    PdfAction::LocalDestination(l) => {
                        l.destination().ok().and_then(|dest| ziel_aus_destination(d, &dest))
                    }
                    _ => None,
                })
            });
            let (seite, y) = match ziel {
                Some(LinkZiel::Intern { seite, y }) => (seite, y),
                _ => (-1, None),
            };
            aus.push(Gliederungspunkt { titel: b.title().unwrap_or_default(), tiefe, seite, y });
            if aus.len() > 20000 {
                return; // Schutz gegen kaputte, zyklische Gliederungen
            }
            sammeln(d, b.first_child(), tiefe + 1, aus);
            aktuell = b.next_sibling();
        }
    }
    let mut aus = Vec::new();
    sammeln(d, d.bookmarks().root(), 0, &mut aus);
    aus
}

/// In eine Datei speichern: erst eine Kopie daneben, dann umbenennen (bricht nie halb ab)
pub fn speichern(stand: &mut Stand, ziel: &Path) -> Result<(), String> {
    let Some(d) = stand.dok.as_ref() else { return Err("kein Dokument offen".into()) };
    let mut tmp = ziel.as_os_str().to_owned();
    tmp.push(".abolos-tmp");
    let tmp = PathBuf::from(tmp);
    d.save_to_file(&tmp).map_err(|e| format!("Speichern fehlgeschlagen: {e}"))?;
    std::fs::rename(&tmp, ziel).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", ziel.display())
    })?;
    stand.geaendert = false;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::datum_lesbar;

    #[test]
    fn datum_in_ortszeit() {
        // ohne Zeitzone: so wie angegeben
        assert_eq!(datum_lesbar("D:20261022093000"), "22.10.2026 09:30");
        // mit Zeitzone: dieselbe Zeit, egal wie geschrieben
        assert_eq!(datum_lesbar("D:20261022143000Z"), datum_lesbar("D:20261022163000+02'00'"));
        assert_eq!(datum_lesbar("D:20261022143000Z"), datum_lesbar("D:20261022093000-05'00'"));
        assert_eq!(datum_lesbar("kein Datum"), "kein Datum");
    }
}
