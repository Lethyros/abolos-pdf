// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Abolos PDF: PDF-Reader für Linux und Windows, ohne Netzwerkfunktionen.

// Windows: als Fensterprogramm starten, ohne zusätzliches Konsolenfenster
#![cfg_attr(windows, windows_subsystem = "windows")]

mod bearbeiten;
mod dokument;
mod engine;
mod hilfe;
mod lesezeichen;
mod pfade;
#[cfg(test)]
mod pruefung;
mod schutz;
mod seite;
mod suche;
mod tasten;
mod zustand;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    // Kommandozeile: --version, --help, sonst die zu öffnende Datei
    let argumente: Vec<String> = std::env::args().skip(1).collect();
    if argumente.iter().any(|a| a == "--version" || a == "-V") {
        schutz::ausgeben(&format!("{} {}", hilfe::PROGRAMMNAME, env!("CARGO_PKG_VERSION")));
        return;
    }
    if argumente.iter().any(|a| a == "--help" || a == "-h") {
        schutz::ausgeben(&hilfe::kommandozeile());
        return;
    }

    // vor dem ersten weiteren Thread: Netz- und Ausführsperre
    let _ = dokument::SCHUTZ.set(schutz::einrichten());
    if std::env::var_os("ABOLOS_NETZTEST").is_some() {
        schutz::selbsttest();
    }

    if let Some(pfad) = argumente.iter().find(|a| !a.starts_with('-')) {
        // absolute statt canonicalize: unter Windows ohne den Vorsatz \\?\ vor dem Laufwerk
        let pfad = std::path::absolute(pfad).map(|p| p.to_string_lossy().into_owned()).unwrap_or(pfad.clone());
        let _ = dokument::START_PFAD.set(pfad);
    }

    schutz::portal_dialoge();
    let mut app = QGuiApplication::new();
    if let Some(mut app) = app.as_mut() {
        app.as_mut().set_application_name(&QString::from(pfade::ORDNERNAME));
        app.as_mut().set_application_display_name(&QString::from(hilfe::PROGRAMMNAME));
        app.as_mut().set_application_version(&QString::from(env!("CARGO_PKG_VERSION")));
    }
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/AbolosPdf/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
