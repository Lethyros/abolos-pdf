// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Textsuche auf dem Seitentext, ohne die Suchfunktion von PDFium: So bringt jeder Treffer seinen
//! Zusammenhang (Text davor und danach) mit, und Dokument- und Ordnersuche finden dasselbe.
//! Leerraum und Zeilenumbrüche zählen als ein Leerzeichen, Ligaturen werden aufgelöst, am
//! Zeilenende getrennte Wörter ("Betriebs-⏎anlage") werden zusammengefügt.

use pdfium_render::prelude::*;

/// Aufbereiteter Text einer Seite: ein Eintrag je Zeichen
pub struct Seitentext {
    pub zeichen: Vec<char>,
    /// PDFium-Zeichenindex je Eintrag in `zeichen` (für die Rechtecke); leer, wenn der Text
    /// aus dem Zwischenspeicher der Ordnersuche stammt
    pub index: Vec<usize>,
}

/// Wie gesucht wird
#[derive(Clone, Copy, Debug, Default)]
pub struct Suchoptionen {
    pub gross_klein: bool,
    /// nur ganze Wörter: "Schutz" findet "Schutz" und "Schutz-Leiter", nicht "Schutzleiter"
    pub ganze_woerter: bool,
}

/// Text davor, Treffer, Text danach (für Ergebnislisten)
pub struct Kontext {
    pub vor: String,
    pub wort: String,
    pub nach: String,
}

fn ist_leer(c: char) -> bool {
    c.is_whitespace() || c == '\u{0}'
}

fn ist_umbruch(c: char) -> bool {
    matches!(c, '\r' | '\n' | '\u{2028}' | '\u{2029}')
}

/// Trennstriche; PDFium liefert für Trennungen am Zeilenende teils 0x02
fn ist_trennstrich(c: char) -> bool {
    matches!(c, '-' | '\u{AD}' | '\u{2010}' | '\u{2}')
}

fn ligatur(c: char) -> Option<&'static str> {
    Some(match c {
        'ﬀ' => "ff",
        'ﬁ' => "fi",
        'ﬂ' => "fl",
        'ﬃ' => "ffi",
        'ﬄ' => "ffl",
        'ﬅ' | 'ﬆ' => "st",
        _ => return None,
    })
}

/// Wörter nach einem Ergänzungsstrich ("Erdungs- oder Kurzschließvorrichtung"): dort ist der Strich
/// am Zeilenende keine Silbentrennung
const BINDEWOERTER: [&str; 8] = ["und", "oder", "bzw", "sowie", "bis", "als", "wie", "noch"];

/// Trennung am Zeilenende zusammenfügen? Ja bei "Nieder-⏎spannung", nein bei "Erdungs-⏎oder"
/// und "VDE-⏎Norm". `j` zeigt auf das erste sichtbare Zeichen der nächsten Zeile.
fn zusammenfuegen(roh: &[(usize, char, bool)], j: usize) -> bool {
    if !roh.get(j).is_some_and(|x| x.1.is_lowercase()) {
        return false;
    }
    let wort: String = roh[j..].iter().map(|x| x.1).take_while(|c| c.is_alphabetic()).collect();
    !BINDEWOERTER.contains(&wort.as_str())
}

/// Vergleichsform eines Zeichens ohne Groß-/Kleinschreibung
fn falten(c: char) -> char {
    if c.is_ascii() { c.to_ascii_lowercase() } else { c.to_lowercase().next().unwrap_or(c) }
}

/// Zeichen (PDFium-Index, Zeichen, Trennstrich am Zeilenende laut PDFium) aufbereiten
fn aufbereiten(roh: &[(usize, char, bool)], trennungen: bool) -> Seitentext {
    let mut st = Seitentext { zeichen: Vec::with_capacity(roh.len()), index: Vec::with_capacity(roh.len()) };
    let mut i = 0;
    while i < roh.len() {
        let (idx, c, weich) = roh[i];
        if trennungen && weich && ist_trennstrich(c) {
            // PDFium hängt getrennte Zeilen oft ohne Umbruch an ("Nieder-spannung"): klein weiter = ein Wort
            let mut j = i + 1;
            while j < roh.len() && ist_leer(roh[j].1) {
                j += 1;
            }
            if st.zeichen.last().is_some_and(|z| z.is_alphabetic()) && roh.get(j).is_some_and(|x| x.1.is_lowercase()) {
                if !zusammenfuegen(roh, j) {
                    // Ergänzungsstrich: das Leerzeichen danach fehlt im PDF
                    st.zeichen.extend(['-', ' ']);
                    st.index.extend([idx, idx]);
                }
                i = j;
                continue;
            }
        }
        if ist_leer(c) {
            let mut j = i;
            let mut umbruch = false;
            while j < roh.len() && ist_leer(roh[j].1) {
                umbruch |= ist_umbruch(roh[j].1);
                j += 1;
            }
            // "Betriebs-" am Zeilenende, danach klein weiter: ein Wort
            let n = st.zeichen.len();
            let getrennt = trennungen
                && umbruch
                && n >= 2
                && ist_trennstrich(st.zeichen[n - 1])
                && st.zeichen[n - 2].is_alphabetic()
                && zusammenfuegen(roh, j);
            if getrennt {
                st.zeichen.pop();
                st.index.pop();
            } else if n > 0 && st.zeichen[n - 1] != ' ' {
                st.zeichen.push(' ');
                st.index.push(idx);
            }
            i = j;
            continue;
        }
        i += 1;
        if c == '\u{AD}' && !roh.get(i).is_some_and(|x| ist_leer(x.1)) {
            continue; // weiches Trennzeichen mitten in der Zeile: unsichtbar
        }
        if (c.is_control() && c != '\u{2}') || c == '\u{FFFE}' || c == '\u{FFFF}' {
            continue;
        }
        let c = if c == '\u{2}' { '-' } else { c };
        match ligatur(c) {
            Some(s) => {
                for z in s.chars() {
                    st.zeichen.push(z);
                    st.index.push(idx);
                }
            }
            None => {
                st.zeichen.push(c);
                st.index.push(idx);
            }
        }
    }
    if st.zeichen.last() == Some(&' ') {
        st.zeichen.pop();
        st.index.pop();
    }
    st
}

/// Text einer PDF-Seite, Zeichen für Zeichen (der Index passt zu `segments_subset`)
pub fn seitentext(text: &PdfPageText) -> Seitentext {
    let roh: Vec<(usize, char, bool)> = text
        .chars()
        .iter()
        .map(|c| {
            let z = c.unicode_char().unwrap_or(' ');
            // nur bei Strichen nachfragen: ein Aufruf mehr je Zeichen wäre unnötig
            let weich = ist_trennstrich(z) && c.is_hyphen().unwrap_or(false);
            (c.index(), z, weich)
        })
        .collect();
    aufbereiten(&roh, true)
}

/// Seitentext aus dem Zwischenspeicher (schon aufbereitet, ohne PDFium-Indizes)
pub fn aus_gespeichert(text: &str) -> Seitentext {
    Seitentext { zeichen: text.chars().collect(), index: Vec::new() }
}

/// Suchbegriff wie den Seitentext aufbereiten (ohne Trennungen zusammenzufügen)
pub fn nadel(text: &str) -> Vec<char> {
    let roh: Vec<(usize, char, bool)> = text.trim().chars().map(|c| (0, c, false)).collect();
    aufbereiten(&roh, false).zeichen
}

/// Fundstellen als Bereiche [a, b) in `st.zeichen`, ohne Überlappung
pub fn finden(st: &Seitentext, nadel: &[char], optionen: Suchoptionen) -> Vec<(usize, usize)> {
    let m = nadel.len();
    if m == 0 || m > st.zeichen.len() {
        return Vec::new();
    }
    // Wortgrenzen nur dort prüfen, wo der Suchbegriff selbst mit einem Buchstaben/einer Ziffer beginnt bzw. endet
    let wortzeichen = |c: char| c.is_alphanumeric();
    let grenze_vorn = optionen.ganze_woerter && wortzeichen(nadel[0]);
    let grenze_hinten = optionen.ganze_woerter && wortzeichen(nadel[m - 1]);
    let z = &st.zeichen;
    let (heu, nadel): (Vec<char>, Vec<char>) = if optionen.gross_klein {
        (st.zeichen.clone(), nadel.to_vec())
    } else {
        (st.zeichen.iter().map(|&c| falten(c)).collect(), nadel.iter().map(|&c| falten(c)).collect())
    };
    let mut aus = Vec::new();
    let mut i = 0;
    while i + m <= heu.len() {
        let passt = heu[i] == nadel[0]
            && heu[i..i + m] == nadel[..]
            && !(grenze_vorn && i > 0 && wortzeichen(z[i - 1]))
            && !(grenze_hinten && i + m < z.len() && wortzeichen(z[i + m]));
        if passt {
            aus.push((i, i + m));
            i += m;
        } else {
            i += 1;
        }
    }
    aus
}

/// PDFium-Zeichenbereich (Start, Anzahl) einer Fundstelle; nur mit `index`
pub fn zeichenbereich(st: &Seitentext, a: usize, b: usize) -> Option<(usize, usize)> {
    let start = *st.index.get(a)?;
    let ende = *st.index.get(b - 1)?;
    Some((start, ende.saturating_sub(start) + 1))
}

/// Zusammenhang: etwa 40 Zeichen davor und 70 danach, an Wortgrenzen gekürzt
pub fn kontext(z: &[char], a: usize, b: usize) -> Kontext {
    const VOR: usize = 40;
    const NACH: usize = 70;
    let mut start = a.saturating_sub(VOR);
    if start > 0 {
        start = (start..a).find(|&i| z[i] == ' ').map(|i| i + 1).unwrap_or(start);
    }
    let mut ende = (b + NACH).min(z.len());
    if ende < z.len() {
        ende = (b..ende).rev().find(|&i| z[i] == ' ').unwrap_or(ende);
    }
    let text = |r: &[char]| punkte_kuerzen(&r.iter().collect::<String>());
    let (mut vor, mut nach) = (text(&z[start..a]), text(&z[b..ende]));
    if start > 0 {
        vor = format!("…{}", vor.trim_start_matches([' ', '…']));
    }
    if ende < z.len() {
        nach = format!("{}…", nach.trim_end_matches([' ', '…']));
    }
    Kontext { vor, wort: text(&z[a..b]), nach }
}

/// Punktreihen aus Inhaltsverzeichnissen (". . . . . 43") zu " … " kürzen
fn punkte_kuerzen(s: &str) -> String {
    let z: Vec<char> = s.chars().collect();
    let punkt = |c: char| matches!(c, '.' | '…' | '·');
    let mut aus = String::with_capacity(s.len());
    let mut i = 0;
    while i < z.len() {
        if punkt(z[i]) {
            let mut j = i;
            while j < z.len() && (punkt(z[j]) || z[j] == ' ') {
                j += 1;
            }
            if z[i..j].iter().filter(|c| punkt(**c)).count() >= 4 {
                let ende = aus.trim_end().len();
                aus.truncate(ende);
                aus.push_str(if j < z.len() { " … " } else { " …" });
                i = j;
                continue;
            }
        }
        aus.push(z[i]);
        i += 1;
    }
    aus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(text: &str) -> Seitentext {
        // "¬" steht im Test für einen Trennstrich, den PDFium als Zeilenende-Trennung meldet
        let roh: Vec<(usize, char, bool)> =
            text.chars().enumerate().map(|(i, c)| if c == '¬' { (i, '-', true) } else { (i, c, false) }).collect();
        aufbereiten(&roh, true)
    }

    fn treffer(text: &str, such: &str) -> Vec<String> {
        let s = st(text);
        finden(&s, &nadel(such), Suchoptionen::default())
            .into_iter()
            .map(|(a, b)| s.zeichen[a..b].iter().collect())
            .collect()
    }

    fn woerter(text: &str, such: &str) -> Vec<String> {
        let s = st(text);
        let optionen = Suchoptionen { gross_klein: false, ganze_woerter: true };
        finden(&s, &nadel(such), optionen).into_iter().map(|(a, b)| s.zeichen[a..b].iter().collect()).collect()
    }

    #[test]
    fn ganze_woerter() {
        assert_eq!(
            woerter("Schutz, Schutzleiter, Erdungs-Schutz (Schutz)", "schutz"),
            vec!["Schutz", "Schutz", "Schutz"]
        );
        assert_eq!(woerter("Fehlerschutz Schutzleiter", "schutz"), Vec::<String>::new());
        assert_eq!(woerter("VDE 0105-100 und VDE 01050", "vde 0105"), vec!["VDE 0105"]);
        // getrennt am Zeilenende zählt als ein Wort
        assert_eq!(woerter("Nieder¬spannung", "niederspannung"), vec!["Niederspannung"]);
        assert_eq!(woerter("Nieder¬spannung", "spannung"), Vec::<String>::new());
        // ein Fehlversuch darf den nächsten Treffer nicht überspringen
        assert_eq!(woerter("Abab ab", "ab"), vec!["ab"]);
        // Satzzeichen am Rand des Suchbegriffs: dort keine Wortgrenze nötig
        assert_eq!(woerter("Erdungs- und", "erdungs-"), vec!["Erdungs-"]);
    }

    #[test]
    fn leerraum_und_gross_klein() {
        assert_eq!(treffer("Betrieb\r\n  von Anlagen", "betrieb VON"), vec!["Betrieb von"]);
        assert_eq!(treffer("Ölwanne öl ÖL", "öl"), vec!["Öl", "öl", "ÖL"]);
    }

    #[test]
    fn trennung_am_zeilenende() {
        assert_eq!(treffer("Betriebs-\r\nanlage", "betriebsanlage"), vec!["Betriebsanlage"]);
        // "Schutz- und": groß weiter oder kein Umbruch bleibt getrennt
        assert_eq!(treffer("Schutz- und Hilfsmittel", "schutz- und"), vec!["Schutz- und"]);
        assert_eq!(treffer("VDE-\r\nNorm", "vde- norm"), vec!["VDE- Norm"]);
        // von PDFium gemeldete Trennung ohne Umbruch dazwischen
        assert_eq!(treffer("Nieder¬spannungsanlagen", "niederspannung"), vec!["Niederspannung"]);
        assert_eq!(treffer("VDE¬Norm", "vde-norm"), vec!["VDE-Norm"]);
        // Ergänzungsstrich am Zeilenende bleibt stehen
        assert_eq!(
            treffer("Erdungs¬oder Erdungs-\r\nund Kurzschließ", "erdungs- oder erdungs- und"),
            vec!["Erdungs- oder Erdungs- und"]
        );
        assert_eq!(treffer("Erdungs¬oder", "erdungsoder"), Vec::<String>::new());
    }

    #[test]
    fn ligaturen_und_index() {
        let s = st("Pﬂicht");
        let f = finden(&s, &nadel("pflicht"), Suchoptionen::default());
        assert_eq!(f, vec![(0, 7)]);
        assert_eq!(zeichenbereich(&s, 0, 7), Some((0, 6)));
    }

    #[test]
    fn kontext_kuerzt_an_wortgrenzen() {
        let text = "Das ist ein sehr langer Satz, in dem irgendwo das Wort Schutzleiter vorkommt und danach geht es noch lange weiter mit vielen Wörtern, Kommas und noch mehr Text am Ende.";
        let s = st(text);
        let (a, b) = finden(&s, &nadel("schutzleiter"), Suchoptionen::default())[0];
        let k = kontext(&s.zeichen, a, b);
        assert_eq!(k.wort, "Schutzleiter");
        assert!(k.vor.starts_with('…') && k.vor.ends_with("Wort "));
        assert!(k.nach.starts_with(" vorkommt") && k.nach.ends_with('…'));
    }

    #[test]
    fn punktreihen_werden_gekuerzt() {
        assert_eq!(punkte_kuerzen("Schutzmittel . . . . . . . 43 3.6 Text"), "Schutzmittel … 43 3.6 Text");
        assert_eq!(punkte_kuerzen("Ende ........"), "Ende …");
        assert_eq!(punkte_kuerzen("z.B. 1.2.3 und ..."), "z.B. 1.2.3 und ...");
    }
}
