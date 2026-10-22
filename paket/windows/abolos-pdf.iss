; SPDX-FileCopyrightText: 2026 Oliver Weber
; SPDX-License-Identifier: GPL-3.0-or-later
;
; Windows-Installer für Abolos PDF (Inno Setup 6).
; Standard: Installation nur für den angemeldeten Benutzer, ohne Adminrechte und ohne UAC-Abfrage.
; Wer Admin ist, kann für alle Benutzer installieren und bekommt dann die Firewall-Regel (Netzsperre).
; Stumme Installation für IT-Verteilung: Abolos-PDF-...-Setup-x64.exe /VERYSILENT /SUPPRESSMSGBOXES
;
; Aufruf (siehe .github/workflows/release.yml):
;   iscc /DVersion=0.9.0 /DQuelle=..\..\build\windows paket\windows\abolos-pdf.iss

#define Name "Abolos PDF"
#ifndef Version
  #define Version "0.0.0"
#endif
#ifndef Quelle
  #define Quelle "..\..\build\windows"
#endif
#ifndef Webseite
  #define Webseite "https://github.com/"
#endif
#define Firewallregel "Abolos PDF - kein Netzwerk"

[Setup]
; feste Kennung: Updates erkennen die bestehende Installation (nie ändern)
AppId={{2FF89107-EF01-4778-8051-10E4132BCF78}
AppName={#Name}
AppVersion={#Version}
AppVerName={#Name} {#Version}
AppPublisher=Oliver Weber
AppPublisherURL={#Webseite}
AppSupportURL={#Webseite}
AppUpdatesURL={#Webseite}
AppCopyright=Copyright © 2026 Oliver Weber
DefaultDirName={autopf}\{#Name}
DefaultGroupName={#Name}
DisableProgramGroupPage=yes
DisableWelcomePage=no
; ohne Adminrechte; auf Wunsch (Dialog oder /ALLUSERS) für alle Benutzer
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Windows 10 1809 oder neuer (Voraussetzung von Qt 6)
MinVersion=10.0.17763
OutputDir=..\..\dist
OutputBaseFilename=Abolos-PDF-{#Version}-Setup-x64
SetupIconFile=..\symbol\abolos-pdf.ico
UninstallDisplayIcon={app}\abolos-pdf.exe
UninstallDisplayName={#Name}
InfoBeforeFile=hinweis.txt
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
ChangesAssociations=yes
; laufendes Programm bei Updates sauber schließen
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#Version}
VersionInfoCompany=Oliver Weber
VersionInfoDescription={#Name} – Installation
VersionInfoProductName={#Name}
VersionInfoCopyright=Copyright © 2026 Oliver Weber, GNU GPL 3 oder später

[Languages]
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
de.Netzsperre=Netzsperre: Windows-Firewall-Regel, die Abolos PDF jeden Netzwerkzugriff verbietet (empfohlen)
en.Netzsperre=Network lock: Windows Firewall rule that blocks all network access for Abolos PDF (recommended)
de.OeffnenMit=Im Menü „Öffnen mit“ für PDF-Dateien anbieten
en.OeffnenMit=Offer in the "Open with" menu for PDF files
de.Schutz=Schutz:
en.Schutz=Protection:
de.Dateien=PDF-Dateien:
en.Dateien=PDF files:

[Tasks]
Name: "oeffnenmit"; Description: "{cm:OeffnenMit}"; GroupDescription: "{cm:Dateien}"
Name: "netzsperre"; Description: "{cm:Netzsperre}"; GroupDescription: "{cm:Schutz}"; Check: IsAdminInstallMode
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#Quelle}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\{#Name}"; Filename: "{app}\abolos-pdf.exe"; Comment: "PDFs lesen, durchsuchen, markieren – offline"
Name: "{autodesktop}\{#Name}"; Filename: "{app}\abolos-pdf.exe"; Tasks: desktopicon

[Registry]
; „Öffnen mit“ und „Standard-Apps“: HKA = für alle Benutzer HKLM, sonst HKCU (ohne Adminrechte)
Root: HKA; Subkey: "Software\Classes\AbolosPDF.Dokument"; ValueType: string; ValueName: ""; ValueData: "PDF-Dokument"; Flags: uninsdeletekey; Tasks: oeffnenmit
Root: HKA; Subkey: "Software\Classes\AbolosPDF.Dokument\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\abolos-pdf.exe,0"; Tasks: oeffnenmit
Root: HKA; Subkey: "Software\Classes\AbolosPDF.Dokument\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\abolos-pdf.exe"" ""%1"""; Tasks: oeffnenmit
Root: HKA; Subkey: "Software\Classes\.pdf\OpenWithProgids"; ValueType: string; ValueName: "AbolosPDF.Dokument"; ValueData: ""; Flags: uninsdeletevalue; Tasks: oeffnenmit
Root: HKA; Subkey: "Software\Abolos PDF\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#Name}"; Flags: uninsdeletekey; Tasks: oeffnenmit
Root: HKA; Subkey: "Software\Abolos PDF\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Offline-PDF-Reader: lesen, suchen, markieren, kommentieren"; Tasks: oeffnenmit
Root: HKA; Subkey: "Software\Abolos PDF\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pdf"; ValueData: "AbolosPDF.Dokument"; Tasks: oeffnenmit
Root: HKA; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "{#Name}"; ValueData: "Software\Abolos PDF\Capabilities"; Flags: uninsdeletevalue; Tasks: oeffnenmit
; Start auch über Win+R „abolos-pdf“
Root: HKA; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\abolos-pdf.exe"; ValueType: string; ValueName: ""; ValueData: "{app}\abolos-pdf.exe"; Flags: uninsdeletekey

[Run]
; Netzsperre: eine alte Regel (frühere Installation) erst entfernen, dann Regeln für beide Richtungen anlegen
Filename: "{sys}\netsh.exe"; Parameters: "advfirewall firewall delete rule name=""{#Firewallregel}"""; Flags: runhidden; Tasks: netzsperre
Filename: "{sys}\netsh.exe"; Parameters: "advfirewall firewall add rule name=""{#Firewallregel}"" dir=out action=block program=""{app}\abolos-pdf.exe"" enable=yes profile=any"; Flags: runhidden; Tasks: netzsperre
Filename: "{sys}\netsh.exe"; Parameters: "advfirewall firewall add rule name=""{#Firewallregel}"" dir=in action=block program=""{app}\abolos-pdf.exe"" enable=yes profile=any"; Flags: runhidden; Tasks: netzsperre
Filename: "{app}\abolos-pdf.exe"; Description: "{cm:LaunchProgram,{#Name}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\netsh.exe"; Parameters: "advfirewall firewall delete rule name=""{#Firewallregel}"""; Flags: runhidden; RunOnceId: "Firewallregel"; Check: IsAdminInstallMode
