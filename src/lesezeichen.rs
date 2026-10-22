// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Eigene Lesezeichen je Datei in ~/.local/share/abolos-pdf/lesezeichen.json.
//! (PDFium kann keine Gliederungseinträge ins PDF schreiben, und so bleibt das PDF unverändert.)

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Lesezeichen {
    pub seite: i32,
    /// relative Höhe auf der Seite (Anzeige ohne Drehung)
    pub y: f64,
    pub name: String,
    pub zeit: String,
}

fn datei() -> PathBuf {
    crate::pfade::daten("lesezeichen.json")
}

fn laden() -> BTreeMap<String, Vec<Lesezeichen>> {
    std::fs::read_to_string(datei()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn sichern(alle: &BTreeMap<String, Vec<Lesezeichen>>) {
    let pfad = datei();
    if let Some(dir) = pfad.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string_pretty(alle)
        && let Err(e) = std::fs::write(&pfad, text)
    {
        eprintln!("{}: {e}", pfad.display());
    }
}

pub fn als_json(dokument: &str) -> String {
    serde_json::to_string(laden().get(dokument).map(Vec::as_slice).unwrap_or(&[])).unwrap_or_else(|_| "[]".into())
}

pub fn setzen(dokument: &str, seite: i32, y: f64, name: &str) {
    if dokument.is_empty() {
        return;
    }
    let mut alle = laden();
    let liste = alle.entry(dokument.to_string()).or_default();
    let zeit = chrono::Local::now().format("%d.%m.%Y %H:%M").to_string();
    let name = if name.trim().is_empty() { format!("Seite {}", seite + 1) } else { name.trim().to_string() };
    liste.push(Lesezeichen { seite, y, name, zeit });
    liste.sort_by(|a, b| (a.seite, a.y).partial_cmp(&(b.seite, b.y)).unwrap_or(std::cmp::Ordering::Equal));
    sichern(&alle);
}

pub fn entfernen(dokument: &str, nr: i32) {
    let mut alle = laden();
    if let Some(liste) = alle.get_mut(dokument)
        && nr >= 0
        && (nr as usize) < liste.len()
    {
        liste.remove(nr as usize);
    }
    sichern(&alle);
}

pub fn umbenennen(dokument: &str, nr: i32, name: &str) {
    let mut alle = laden();
    if let Some(eintrag) = alle.get_mut(dokument).and_then(|l| l.get_mut(nr.max(0) as usize)) {
        eintrag.name = name.trim().to_string();
    }
    sichern(&alle);
}
