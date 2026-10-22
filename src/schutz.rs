// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Schutzmaßnahmen beim Start, je Betriebssystem (siehe doku/sicherheit.md):
//! - Linux: Netzsperre (eigener, leerer Netzwerk-Namensraum) und Ausführsperre (Landlock)
//! - Windows: Ausführsperre (Prozessrichtlinie „keine Kindprozesse“); die Netzsperre ist dort eine
//!   Firewall-Regel, die der Installer bei einer Installation für alle Benutzer anlegt
//!
//! Alles muss laufen, bevor der erste weitere Thread startet.

/// Zustand einer Schutzmaßnahme
#[derive(Clone, Debug)]
pub enum Zustand {
    /// greift; Text = Einzelheit (z.B. Landlock-Version)
    Aktiv(String),
    /// sollte greifen, tut es aber nicht; Text = Grund (wird in der Statuszeile gemeldet)
    Fehlt(String),
    /// liegt außerhalb des Programms (Windows-Firewall); Text = Beschreibung
    #[cfg_attr(not(windows), allow(dead_code))]
    Extern(String),
}

impl Zustand {
    /// Grund für die Warnung in der Statuszeile, sonst ""
    pub fn warnung(&self) -> &str {
        match self {
            Zustand::Fehlt(grund) => grund,
            _ => "",
        }
    }

    /// Beschreibung für „Über“ und Hilfe
    pub fn beschreibung(&self) -> String {
        match self {
            Zustand::Aktiv(einzelheit) if einzelheit.is_empty() => "aktiv".into(),
            Zustand::Aktiv(einzelheit) => format!("aktiv ({einzelheit})"),
            Zustand::Fehlt(grund) => format!("nicht aktiv ({grund})"),
            Zustand::Extern(text) => text.clone(),
        }
    }
}

pub struct Schutz {
    pub netz: Zustand,
    pub ausfuehren: Zustand,
}

/// Netz- und Ausführsperre einrichten; Warnungen gehen zusätzlich auf stderr
pub fn einrichten() -> Schutz {
    let schutz = system::einrichten();
    if let Zustand::Fehlt(grund) = &schutz.netz {
        eprintln!("Warnung: Netzsperre nicht möglich ({grund}); Abolos PDF nutzt trotzdem keine Netzwerkfunktionen.");
    }
    if let Zustand::Fehlt(grund) = &schutz.ausfuehren {
        eprintln!("Warnung: Ausführsperre nicht möglich ({grund}).");
    }
    schutz
}

/// Selbsttest (ABOLOS_NETZTEST=1): Verbindung ins Internet und Programmstart versuchen
pub fn selbsttest() {
    let versuch =
        std::net::TcpStream::connect_timeout(&"1.1.1.1:443".parse().unwrap(), std::time::Duration::from_secs(3));
    ausgeben(&format!(
        "Netztest: Verbindung zu 1.1.1.1:443 {}",
        match versuch {
            Ok(_) => "MÖGLICH (keine Netzsperre wirksam)".to_string(),
            Err(e) => format!("blockiert: {e}"),
        }
    ));
    let (programm, argumente): (&str, &[&str]) =
        if cfg!(windows) { ("cmd.exe", &["/c", "exit"]) } else { ("/usr/bin/true", &[]) };
    let start = std::process::Command::new(programm).args(argumente).status();
    ausgeben(&format!(
        "Ausführtest: {programm} {}",
        match start {
            Ok(_) => "gestartet (Ausführsperre wirkt nicht!)".to_string(),
            Err(e) => format!("blockiert: {e}"),
        }
    ));
}

/// Text auf die Konsole schreiben. Unter Windows hat das Programm keine eigene Konsole
/// (sonst ginge bei jedem Start ein schwarzes Fenster auf) und hängt sich an die des Aufrufers.
pub fn ausgeben(text: &str) {
    #[cfg(windows)]
    {
        use std::io::Write;
        use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
        // SAFETY: einfacher Systemaufruf ohne Zeiger; schlägt fehl, wenn es keine aufrufende Konsole gibt
        if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } != 0
            && let Ok(mut konsole) = std::fs::OpenOptions::new().write(true).open("CONOUT$")
        {
            let _ = writeln!(konsole, "{text}");
        }
    }
    #[cfg(not(windows))]
    {
        println!("{text}");
    }
}

/// Dateidialoge über das Desktop-Portal (nur Linux): So erscheint der gewohnte Dialog des Desktops
/// (KDE, GNOME) mit Symbolen und Vorschau statt Qts schlichtem Ersatzdialog, den es ohne Qt Widgets
/// sonst gibt. Das Portal läuft als eigener Prozess und wird über D-Bus (Unix-Socket) erreicht, die
/// Netzsperre stört es nicht. Abschalten: ABOLOS_QT_DIALOGE=1 oder eigenes QT_QPA_PLATFORMTHEME.
pub fn portal_dialoge() {
    #[cfg(target_os = "linux")]
    {
        let gesetzt = |n: &str| std::env::var_os(n).is_some_and(|v| !v.is_empty());
        let bus = gesetzt("DBUS_SESSION_BUS_ADDRESS")
            || std::env::var_os("XDG_RUNTIME_DIR").is_some_and(|d| std::path::Path::new(&d).join("bus").exists());
        if bus && !gesetzt("QT_QPA_PLATFORMTHEME") && !gesetzt("ABOLOS_QT_DIALOGE") {
            // SAFETY: es läuft noch kein zweiter Thread, der die Umgebung liest
            unsafe { std::env::set_var("QT_QPA_PLATFORMTHEME", "xdgdesktopportal") };
        }
    }
}

#[cfg(target_os = "linux")]
mod system {
    use super::{Schutz, Zustand};

    pub fn einrichten() -> Schutz {
        let netz = match netz_sperren() {
            Ok(()) => Zustand::Aktiv(String::new()),
            Err(e) => Zustand::Fehlt(e),
        };
        let ausfuehren = match ausfuehren_sperren() {
            Ok(abi) => Zustand::Aktiv(format!("Landlock ABI {abi}")),
            Err(e) => Zustand::Fehlt(e),
        };
        Schutz { netz, ausfuehren }
    }

    /// Netzsperre: der Prozess zieht in einen eigenen, leeren Netzwerk-Namensraum (nur ein abgeschaltetes
    /// Loopback). Danach kann nichts im Prozess eine Verbindung aufbauen, weder Qt noch PDFium noch Inhalte
    /// eines PDFs. Unix-Sockets mit Dateipfad (Wayland, D-Bus) funktionieren weiter.
    /// unshare(CLONE_NEWUSER) verlangt einen Prozess mit nur einem Thread.
    fn netz_sperren() -> Result<(), String> {
        // SAFETY: einfache Systemaufrufe ohne Zeiger; der Prozess hat hier noch nur einen Thread
        let (uid, gid) = unsafe { (libc::getuid(), libc::getgid()) };
        // SAFETY: wie oben
        if unsafe { libc::unshare(libc::CLONE_NEWUSER | libc::CLONE_NEWNET) } != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        // eigene Benutzer-/Gruppenkennung im neuen Namensraum behalten
        let schreiben =
            |datei: &str, inhalt: String| std::fs::write(datei, inhalt).map_err(|e| format!("{datei}: {e}"));
        schreiben("/proc/self/setgroups", "deny".into())?;
        schreiben("/proc/self/uid_map", format!("{uid} {uid} 1"))?;
        schreiben("/proc/self/gid_map", format!("{gid} {gid} 1"))?;
        Ok(())
    }

    /// Ausführsperre (Landlock, Linux ab 5.13): Ab hier kann der Prozess keine Programme mehr starten.
    /// Wer über eine manipulierte PDF-Datei Code in PDFium zur Ausführung bringt, kann so weder eine Shell
    /// noch andere Programme starten. Das Laden von Bibliotheken (Qt-Plugins) ist davon nicht betroffen.
    /// Gilt für den aufrufenden Thread und alle später gestarteten. Rückgabe: Landlock-Version des Kernels.
    fn ausfuehren_sperren() -> Result<i64, String> {
        /// struct landlock_ruleset_attr, erste Fassung (ABI 1); neuere Kernel verstehen sie ebenso
        #[repr(C)]
        struct Regelsatz {
            handled_access_fs: u64,
        }
        const LANDLOCK_CREATE_RULESET_VERSION: libc::c_uint = 1;
        const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1;
        let fehler = |was: &str| format!("{was}: {}", std::io::Error::last_os_error());

        // SAFETY: Versionsabfrage, kein Zeiger wird gelesen
        let abi = unsafe {
            libc::syscall(
                libc::SYS_landlock_create_ruleset,
                std::ptr::null::<Regelsatz>(),
                0usize,
                LANDLOCK_CREATE_RULESET_VERSION,
            )
        };
        if abi < 1 {
            return Err(fehler("Landlock nicht verfügbar"));
        }
        let regeln = Regelsatz { handled_access_fs: LANDLOCK_ACCESS_FS_EXECUTE };
        // SAFETY: Zeiger auf eine lebende Struktur, Größe passt dazu
        let fd = unsafe {
            libc::syscall(
                libc::SYS_landlock_create_ruleset,
                &regeln as *const Regelsatz,
                std::mem::size_of::<Regelsatz>(),
                0u32,
            )
        };
        if fd < 0 {
            return Err(fehler("Regelsatz"));
        }
        // Ohne Regeln ist "Ausführen" nirgends erlaubt. no_new_privs ist Voraussetzung für unprivilegierte Prozesse.
        // SAFETY: prctl und landlock_restrict_self ohne Zeiger; fd stammt aus landlock_create_ruleset
        let ergebnis = unsafe {
            if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                Err(fehler("no_new_privs"))
            } else if libc::syscall(libc::SYS_landlock_restrict_self, fd as libc::c_int, 0u32) != 0 {
                Err(fehler("Einschränkung"))
            } else {
                Ok(abi)
            }
        };
        // SAFETY: fd gehört nur uns und wird danach nicht mehr benutzt
        unsafe { libc::close(fd as libc::c_int) };
        ergebnis
    }
}

#[cfg(windows)]
mod system {
    use super::{Schutz, Zustand};
    use windows_sys::Win32::System::SystemServices::{
        PROCESS_MITIGATION_CHILD_PROCESS_POLICY, PROCESS_MITIGATION_CHILD_PROCESS_POLICY_0,
    };
    use windows_sys::Win32::System::Threading::{ProcessChildProcessPolicy, SetProcessMitigationPolicy};

    pub fn einrichten() -> Schutz {
        let ausfuehren = match keine_kindprozesse() {
            Ok(()) => Zustand::Aktiv("Windows-Prozessrichtlinie".into()),
            Err(e) => Zustand::Fehlt(e),
        };
        let netz = Zustand::Extern(
            "über die Windows-Firewall: Regel „Abolos PDF - kein Netzwerk“, \
             die der Installer bei einer Installation für alle Benutzer anlegt"
                .into(),
        );
        Schutz { netz, ausfuehren }
    }

    /// Ausführsperre (Windows 10 ab 1709): Der Prozess kann keine weiteren Prozesse mehr starten.
    /// Gilt für den ganzen Prozess und lässt sich nicht wieder aufheben.
    fn keine_kindprozesse() -> Result<(), String> {
        // Bit 0: NoChildProcessCreation
        let richtlinie = PROCESS_MITIGATION_CHILD_PROCESS_POLICY {
            Anonymous: PROCESS_MITIGATION_CHILD_PROCESS_POLICY_0 { Flags: 1 },
        };
        // SAFETY: Zeiger auf eine lebende Struktur, Größe passt dazu
        let ok = unsafe {
            SetProcessMitigationPolicy(
                ProcessChildProcessPolicy,
                &richtlinie as *const PROCESS_MITIGATION_CHILD_PROCESS_POLICY as *const core::ffi::c_void,
                std::mem::size_of::<PROCESS_MITIGATION_CHILD_PROCESS_POLICY>(),
            )
        };
        if ok != 0 { Ok(()) } else { Err(std::io::Error::last_os_error().to_string()) }
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod system {
    use super::{Schutz, Zustand};

    pub fn einrichten() -> Schutz {
        let fehlt = || Zustand::Fehlt("auf diesem Betriebssystem nicht umgesetzt".into());
        Schutz { netz: fehlt(), ausfuehren: fehlt() }
    }
}
