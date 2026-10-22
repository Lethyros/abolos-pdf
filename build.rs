// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bauskript: QML-Modul und Qt-Brücken übersetzen, Versionen der Bausteine für „Über“ und Hilfe festhalten.

use std::process::Command;

use cxx_qt_build::{CxxQtBuilder, QmlModule};

/// erste Zeile der Ausgabe eines Programms, falls es läuft
fn ausgabe(programm: &str, argumente: &[&str]) -> Option<String> {
    let aus = Command::new(programm).args(argumente).output().ok()?;
    let text = String::from_utf8(aus.stdout).ok()?;
    let zeile = text.lines().next()?.trim().to_string();
    (aus.status.success() && !zeile.is_empty()).then_some(zeile)
}

/// Windows-Manifest: ohne Adminrechte starten (keine UAC-Abfrage), Windows 10/11, hohe Auflösung,
/// lange Pfade und UTF-8 für ältere Systemaufrufe (Umlaute in Dateinamen)
const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
      <activeCodePage xmlns="http://schemas.microsoft.com/SMI/2019/WindowsSettings">UTF-8</activeCodePage>
    </windowsSettings>
  </application>
</assembly>
"#;

/// Windows: Programmsymbol, Versionsangaben (Eigenschaften → Details) und Manifest in die .exe.
/// Qt nimmt das Symbol mit dem Namen IDI_ICON1 auch für Fenster und Taskleiste.
fn windows_ressourcen() {
    println!("cargo:rerun-if-changed=paket/symbol/abolos-pdf.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon_with_id("paket/symbol/abolos-pdf.ico", "IDI_ICON1")
            .set("ProductName", "Abolos PDF")
            .set("FileDescription", "Abolos PDF – Offline-PDF-Reader")
            .set("CompanyName", "Oliver Weber")
            .set("LegalCopyright", "Copyright © 2026 Oliver Weber, GNU GPL 3 oder später")
            .set("OriginalFilename", "abolos-pdf.exe")
            .set_manifest(MANIFEST);
        res.compile().expect("Windows-Ressourcen (Symbol, Version, Manifest)");
    }
    #[cfg(not(windows))]
    let _ = MANIFEST;
}

fn main() {
    windows_ressourcen();
    let qt = ausgabe("qmake6", &["-query", "QT_VERSION"])
        .or_else(|| ausgabe("qmake", &["-query", "QT_VERSION"]))
        .unwrap_or_else(|| "unbekannt".into());
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let rust = ausgabe(&rustc, &["--version"]).unwrap_or_else(|| "unbekannt".into());
    // third_party/pdfium/VERSION: MAJOR=151 MINOR=0 BUILD=7881 PATCH=0
    let pdfium = std::fs::read_to_string("third_party/pdfium/VERSION")
        .map(|t| {
            t.lines().filter_map(|z| z.split_once('=').map(|(_, w)| w.trim().to_string())).collect::<Vec<_>>().join(".")
        })
        .unwrap_or_else(|_| "unbekannt".into());
    println!("cargo:rustc-env=ABOLOS_QT_VERSION={qt}");
    println!("cargo:rustc-env=ABOLOS_RUST_VERSION={rust}");
    println!("cargo:rustc-env=ABOLOS_PDFIUM_VERSION={pdfium}");
    println!("cargo:rerun-if-changed=third_party/pdfium/VERSION");
    // eingebettete Dokumentation (hilfe.rs)
    for d in ["doku", "lizenzen", "LICENSE", "CHANGELOG.md"] {
        println!("cargo:rerun-if-changed={d}");
    }

    CxxQtBuilder::new_qml_module(QmlModule::new("AbolosPdf").qml_files([
        "qml/Main.qml",
        "qml/TastenDialog.qml",
        "qml/Seitenleiste.qml",
        "qml/HilfeFenster.qml",
    ]))
    .qt_module("Quick")
    .files(["src/dokument.rs", "src/hilfe.rs", "src/seite.rs", "src/tasten.rs"])
    .build();
}
