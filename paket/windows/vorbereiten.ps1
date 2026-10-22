# SPDX-FileCopyrightText: 2026 Oliver Weber
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Stellt build\windows zusammen: alles, was Abolos PDF unter Windows braucht, in einem Ordner.
#   - abolos-pdf.exe, pdfium.dll
#   - Qt mit QML-Modulen und Plugins (windeployqt)
#   - Visual-C++-Laufzeit direkt daneben: kein zusätzlicher Installer, keine Abfrage
#   - Lizenztexte und Software-Stückliste im Unterordner lizenzen
# Daraus entstehen der Installer (abolos-pdf.iss) und die ZIP-Fassung ohne Installation.
#
# Aufruf: pwsh paket\windows\vorbereiten.ps1 [-Programm target\release\abolos-pdf.exe]
# Voraussetzungen: Qt (windeployqt im PATH), Visual Studio Build Tools, PDFium in third_party\pdfium
param([string]$Programm = "target\release\abolos-pdf.exe")
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..\..")

$ziel = "build\windows"
if (Test-Path $ziel) { Remove-Item -Recurse -Force $ziel }
New-Item -ItemType Directory -Force $ziel | Out-Null

Copy-Item $Programm "$ziel\abolos-pdf.exe"
Copy-Item "third_party\pdfium\bin\pdfium.dll" $ziel

# Qt: Bibliotheken, Plugins und die QML-Module, die qml\*.qml importiert
& windeployqt --release --qmldir qml --no-translations "$ziel\abolos-pdf.exe"
if ($LASTEXITCODE -ne 0) { throw "windeployqt fehlgeschlagen" }

# Visual-C++-Laufzeit (vcruntime140.dll, msvcp140.dll, ...) aus den Build Tools
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
$crt = Get-ChildItem "$vs\VC\Redist\MSVC\*\x64\Microsoft.VC14*.CRT" -Directory |
    Sort-Object FullName -Descending | Select-Object -First 1
if (-not $crt) { throw "Visual-C++-Laufzeit nicht gefunden unter $vs" }
Copy-Item "$($crt.FullName)\*.dll" $ziel
Write-Host "VC-Laufzeit: $($crt.FullName)"

# Lizenzen, Stückliste, kurze Anleitung
$lizenzen = "$ziel\lizenzen"
New-Item -ItemType Directory -Force $lizenzen | Out-Null
Copy-Item "LICENSE" "$lizenzen\LICENSE.txt"
Copy-Item "lizenzen\DRITTANBIETER.txt" $lizenzen
Copy-Item "sbom\abolos-pdf.cdx.json" $lizenzen

$anzahl = (Get-ChildItem -Recurse -File $ziel).Count
Write-Host "build\windows: $anzahl Dateien"
