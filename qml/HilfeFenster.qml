// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import AbolosPdf

// Hilfefenster: Kapitel links, Inhalt rechts. Die Texte kommen aus dem Programm selbst (hilfe.rs),
// die Tastenbelegung wird aus der aktuellen Belegung erzeugt. Externe Adressen werden nie geöffnet.
ApplicationWindow {
    id: hilfe
    title: "Hilfe – Abolos PDF"
    width: 1050
    height: 780
    minimumWidth: 640
    minimumHeight: 420

    property string aktuell: "handbuch"
    readonly property var kapitel: JSON.parse(Hilfe.kapitel())
    readonly property string art: {
        for (var i = 0; i < kapitel.length; i++)
            if (kapitel[i].id === aktuell) return kapitel[i].art
        return "markdown"
    }

    function nach(version, wert) { return wert }

    function zeigen(id) {
        if (id) aktuell = id
        show()
        raise()
        requestActivate()
    }

    // Tastennamen wie auf deutschen Tastaturen
    function tasteDeutsch(t) { return Tasten.anzeigename(t) }

    // Tastenbelegung als Markdown, nach Gruppen in der Reihenfolge ihres ersten Auftretens
    function tastenMarkdown() {
        var liste = JSON.parse(Tasten.aktionen())
        var gruppen = [], zeilen = {}
        for (var i = 0; i < liste.length; i++) {
            var a = liste[i]
            if (!(a.gruppe in zeilen)) { gruppen.push(a.gruppe); zeilen[a.gruppe] = [] }
            var tasten = a.tasten.length ? a.tasten.map(function(t) { return "`" + tasteDeutsch(t) + "`" }).join(", ") : "–"
            zeilen[a.gruppe].push("| " + a.name + " | " + tasten + " |")
        }
        var md = "# Tastenbelegung\n\nSo sind die Tasten gerade belegt. Ändern lässt sich jede Taste unter **Tastenbelegung** "
               + "(" + tasteDeutsch(erstesTaste("tastenbelegung")) + "); eine Aktion kann mehrere Tasten haben.\n"
        for (var g = 0; g < gruppen.length; g++)
            md += "\n## " + gruppen[g] + "\n\n| Aktion | Tasten |\n|---|---|\n" + zeilen[gruppen[g]].join("\n") + "\n"
        return md
    }

    function erstesTaste(id) {
        var liste = JSON.parse(Tasten.aktionen())
        for (var i = 0; i < liste.length; i++)
            if (liste[i].id === id && liste[i].tasten.length > 0) return liste[i].tasten[0]
        return "–"
    }

    readonly property string inhalt: aktuell === "tasten" ? nach(Tasten.version, tastenMarkdown()) : Hilfe.inhalt(aktuell)

    onAktuellChanged: {
        blatt.contentY = 0
        suchPos = 0
        textfeld.deselect()
    }

    // Verweise: "hilfe:kapitel" springt, alles andere wird nur angezeigt (Abolos PDF ist offline)
    function verweis(link) {
        if (link.indexOf("hilfe:") === 0) aktuell = link.slice(6)
        else { adresseDialog.adresse = link; adresseDialog.open() }
    }

    // Suche im aktuellen Kapitel; Enter springt zum nächsten Vorkommen
    property int suchPos: 0
    function weitersuchen(vonVorn) {
        var such = suchfeld.text.toLowerCase()
        if (!such) { textfeld.deselect(); suchInfo.text = ""; return }
        var text = textfeld.getText(0, textfeld.length).toLowerCase()
        var i = text.indexOf(such, vonVorn ? 0 : suchPos)
        if (i < 0) i = text.indexOf(such) // wieder von vorn
        if (i < 0) { textfeld.deselect(); suchInfo.text = "nicht gefunden"; return }
        textfeld.select(i, i + such.length)
        suchPos = i + such.length
        var r = textfeld.positionToRectangle(i)
        blatt.contentY = Math.max(0, Math.min(r.y + textfeld.y - blatt.height / 3, blatt.contentHeight - blatt.height))
        var anzahl = 0, p = -1
        while ((p = text.indexOf(such, p + 1)) >= 0) anzahl++
        suchInfo.text = anzahl + " ×"
    }

    // Bildschirmfoto des Fensterinhalts (Testskript)
    function bildSpeichern(pfad, fertig) {
        wurzel.grabToImage(function(r) { fertig(r.saveToFile(pfad)) })
    }

    SplitView {
        id: wurzel
        anchors.fill: parent
        orientation: Qt.Horizontal
        // Hintergrund gehört zum Inhalt, damit auch Bildschirmfotos (grabToImage) ihn enthalten
        background: Rectangle { color: palette.window }

        ListView {
            id: kapitelListe
            SplitView.preferredWidth: 260
            SplitView.minimumWidth: 180
            clip: true
            model: hilfe.kapitel
            header: Label {
                width: kapitelListe.width
                padding: 12
                text: "Abolos PDF " + JSON.parse(Hilfe.info()).version
                font.bold: true
                font.pointSize: 12
            }
            delegate: ItemDelegate {
                required property var modelData
                width: kapitelListe.width
                text: modelData.titel
                highlighted: modelData.id === hilfe.aktuell
                onClicked: hilfe.aktuell = modelData.id
            }
        }

        ColumnLayout {
            SplitView.fillWidth: true
            spacing: 0

            RowLayout {
                Layout.fillWidth: true
                Layout.margins: 8
                spacing: 8
                TextField {
                    id: suchfeld
                    Layout.fillWidth: true
                    Layout.maximumWidth: 360
                    placeholderText: "In diesem Kapitel suchen …"
                    onTextEdited: hilfe.weitersuchen(true)
                    onAccepted: hilfe.weitersuchen(false)
                }
                Label { id: suchInfo; color: palette.placeholderText }
                Item { Layout.fillWidth: true }
                Button {
                    text: "Text kopieren"
                    onClicked: { textfeld.selectAll(); textfeld.copy(); textfeld.deselect() }
                }
            }

            Flickable {
                id: blatt
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                contentWidth: width
                contentHeight: textfeld.implicitHeight + 48
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar {}

                TextEdit {
                    id: textfeld
                    x: 24
                    y: 12
                    width: blatt.width - 48
                    readOnly: true
                    selectByMouse: true
                    persistentSelection: true
                    wrapMode: hilfe.art === "text" ? TextEdit.WrapAtWordBoundaryOrAnywhere : TextEdit.Wrap
                    textFormat: hilfe.art === "text" ? TextEdit.PlainText : TextEdit.MarkdownText
                    font.family: hilfe.art !== "text" ? Qt.application.font.family
                                 : (Qt.platform.os === "windows" ? "Consolas" : "monospace")
                    color: palette.text
                    selectionColor: palette.highlight
                    selectedTextColor: palette.highlightedText
                    text: hilfe.inhalt
                    onLinkActivated: (link) => hilfe.verweis(link)
                    HoverHandler {
                        cursorShape: textfeld.hoveredLink ? Qt.PointingHandCursor : Qt.IBeamCursor
                    }
                }
            }
        }
    }

    Dialog {
        id: adresseDialog
        title: "Externe Adresse"
        modal: true
        anchors.centerIn: Overlay.overlay
        width: Math.min(hilfe.width - 80, 560)
        standardButtons: Dialog.Close
        property string adresse: ""
        ColumnLayout {
            anchors.fill: parent
            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: "Abolos PDF öffnet keine Verbindungen ins Internet. Die Adresse können Sie kopieren und selbst im Browser öffnen."
            }
            TextField {
                id: adresseFeld
                Layout.fillWidth: true
                readOnly: true
                text: adresseDialog.adresse
            }
            Button {
                text: "Adresse kopieren"
                onClicked: { adresseFeld.selectAll(); adresseFeld.copy(); adresseFeld.deselect() }
            }
        }
    }

    Shortcut {
        sequences: [StandardKey.Find]
        onActivated: { suchfeld.forceActiveFocus(); suchfeld.selectAll() }
    }
    Shortcut {
        sequence: "Esc"
        onActivated: hilfe.close()
    }
}
