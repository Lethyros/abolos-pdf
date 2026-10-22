// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Speicherorte für Einstellungen und Daten:
//! - Linux nach XDG: ~/.config/abolos-pdf und ~/.local/share/abolos-pdf
//! - Windows: %APPDATA%\abolos-pdf für beides
//!
//! Beim ersten Zugriff unter dem neuen Namen werden Dateien aus dem alten Ordner "seitenblick" kopiert
//! (nicht verschoben: der alte Ordner bleibt als Sicherung).

use std::path::PathBuf;

/// Ordnername in ~/.config und ~/.local/share
pub const ORDNERNAME: &str = "abolos-pdf";
/// Arbeitstitel bis Version 0.9 (2026-10-22)
const ALTER_ORDNERNAME: &str = "seitenblick";

/// `$XDG_…` (nur absolut), sonst `$HOME/<ersatz>`; unter Windows immer `%APPDATA%`
fn basis(variable: &str, ersatz: &str) -> PathBuf {
    #[cfg(windows)]
    let (variable, ersatz) = {
        let _ = (variable, ersatz);
        ("APPDATA", "AppData\\Roaming")
    };
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(ersatz)))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn mit_uebernahme(basis: PathBuf, datei: &str) -> PathBuf {
    let neu = basis.join(ORDNERNAME).join(datei);
    let alt = basis.join(ALTER_ORDNERNAME).join(datei);
    if !neu.exists() && alt.is_file() {
        if let Some(ordner) = neu.parent() {
            let _ = std::fs::create_dir_all(ordner);
        }
        if let Err(e) = std::fs::copy(&alt, &neu) {
            eprintln!("Übernahme {} -> {}: {e}", alt.display(), neu.display());
        }
    }
    neu
}

/// Einstellungsdatei, z.B. `konfig("tasten.toml")`
pub fn konfig(datei: &str) -> PathBuf {
    mit_uebernahme(basis("XDG_CONFIG_HOME", ".config"), datei)
}

/// Datendatei, z.B. `daten("lesezeichen.json")`
pub fn daten(datei: &str) -> PathBuf {
    mit_uebernahme(basis("XDG_DATA_HOME", ".local/share"), datei)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alte_datei_wird_kopiert_nicht_verschoben() {
        let wurzel = std::env::temp_dir().join(format!("abolos-pfade-{}", std::process::id()));
        let alt = wurzel.join(ALTER_ORDNERNAME);
        std::fs::create_dir_all(&alt).unwrap();
        std::fs::write(alt.join("tasten.toml"), "x = 1").unwrap();

        let neu = mit_uebernahme(wurzel.clone(), "tasten.toml");
        assert_eq!(neu, wurzel.join(ORDNERNAME).join("tasten.toml"));
        assert_eq!(std::fs::read_to_string(&neu).unwrap(), "x = 1");
        assert!(alt.join("tasten.toml").exists(), "alte Datei bleibt als Sicherung");

        // vorhandene neue Datei wird nie überschrieben
        std::fs::write(&neu, "y = 2").unwrap();
        mit_uebernahme(wurzel.clone(), "tasten.toml");
        assert_eq!(std::fs::read_to_string(&neu).unwrap(), "y = 2");
        std::fs::remove_dir_all(&wurzel).unwrap();
    }
}
