// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import AbolosPdf

// Links: Gliederung des PDFs, eigene Lesezeichen, alle Anmerkungen, Suchergebnisse. Klick springt hin.
Pane {
    id: leiste
    required property var fenster
    padding: 0

    function reiter(nr) { reiterLeiste.currentIndex = nr }

    property alias ordnerNurGanzeWoerter: ordnerGanzeWoerter.checked

    // die Ordnersuche geht auch ohne geöffnetes Dokument
    readonly property bool ordnerSucheOffen: reiterLeiste.currentIndex === 3 && suchArt.currentIndex === 1

    // Suche-Reiter zeigen; ordner = true: Suche über einen Ordner, sonst Treffer im Dokument
    function zeigeSuche(ordner) {
        reiterLeiste.currentIndex = 3
        suchArt.currentIndex = ordner ? 1 : 0
        if (ordner) {
            if (!ordnerFeld.text) ordnerFeld.text = ordnerVon(Dokument.pfad)
            ordnerSuchfeld.forceActiveFocus()
            ordnerSuchfeld.selectAll()
        }
    }

    function ordnerWaehlen() { ordnerDialog.open() }

    function ordnerVon(pfad) { return Dokument.ordnerVon(pfad) }

    function ordnerSuchen(ordner, text) {
        if (ordner) ordnerFeld.text = ordner
        if (text !== undefined) ordnerSuchfeld.text = text
        if (!ordnerFeld.text || !ordnerSuchfeld.text.trim()) return
        letzteOrdnersuche = { text: ordnerSuchfeld.text, grossKlein: ordnerGrossKlein.checked, ganzeWoerter: ordnerGanzeWoerter.checked }
        Dokument.ordnerSuchen(ordnerFeld.text, ordnerSuchfeld.text, unterordner.checked, ordnerGrossKlein.checked, ordnerGanzeWoerter.checked)
    }

    // Suchbegriff der Ergebnisse (das Feld kann sich inzwischen geändert haben)
    property var letzteOrdnersuche: ({ text: "", grossKlein: false, ganzeWoerter: false })

    function ordnerTrefferKlick(nr) {
        if (nr < 0 || nr >= ordnerModell.count) return
        var t = ordnerModell.get(nr)
        ordnerListe.currentIndex = nr
        fenster.ordnerTrefferOeffnen({ datei: t.datei, seite: t.seite, nr: t.nr,
                                       suchtext: letzteOrdnersuche.text, grossKlein: letzteOrdnersuche.grossKlein,
                                       ganzeWoerter: letzteOrdnersuche.ganzeWoerter })
    }

    // Text sicher in StyledText einsetzen, Treffer hervorheben
    function zusammenhang(vor, wort, nach) {
        var esc = function(t) { return t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;") }
        return esc(vor) + "<b><font color=\"#e07000\">" + esc(wort) + "</font></b>" + esc(nach)
    }

    // Trefferliste im Dokument: neu aufbauen höchstens alle 300 ms, solange die Suche läuft
    property int trefferZahl: 0
    Timer {
        id: trefferNachladen
        interval: 300
        onTriggered: {
            var y = trefferListe.contentY
            leiste.trefferZahl = Dokument.trefferAnzahl
            trefferListe.contentY = y
        }
    }

    // Ordnersuche: Ergebnisse werden nur angehängt
    function ordnerNachholen() {
        var von = ordnerModell.count, bis = Dokument.ordnerAnzahl
        if (bis <= von) return
        var neu = JSON.parse(Dokument.ordnerTreffer(von, bis))
        for (var i = 0; i < neu.length; i++) ordnerModell.append(neu[i])
    }
    Timer { id: ordnerNachladen; interval: 250; onTriggered: leiste.ordnerNachholen() }

    Connections {
        target: Dokument
        function onTrefferAnzahlChanged() {
            if (Dokument.trefferAnzahl === 0) leiste.trefferZahl = 0
            else if (!trefferNachladen.running) trefferNachladen.start()
        }
        function onSucheLaeuftChanged() { if (!Dokument.sucheLaeuft) trefferNachladen.restart() }
        function onAktuellerTrefferChanged() {
            if (Dokument.aktuellerTreffer >= 0 && Dokument.aktuellerTreffer < trefferListe.count)
                trefferListe.positionViewAtIndex(Dokument.aktuellerTreffer, ListView.Contain)
        }
        function onOrdnerGenerationChanged() { ordnerModell.clear() }
        function onOrdnerAnzahlChanged() { if (!ordnerNachladen.running) ordnerNachladen.start() }
        function onOrdnerLaeuftChanged() { if (!Dokument.ordnerLaeuft) ordnerNachladen.restart() }
    }

    FolderDialog {
        id: ordnerDialog
        title: "Ordner durchsuchen"
        currentFolder: Dokument.dateiUrl(ordnerFeld.text)
        onAccepted: {
            ordnerFeld.text = Dokument.lokalerPfad(selectedFolder.toString())
            ordnerSuchfeld.forceActiveFocus()
            if (ordnerSuchfeld.text.trim()) leiste.ordnerSuchen()
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        TabBar {
            id: reiterLeiste
            Layout.fillWidth: true
            // Breite nach Text, damit nichts abgeschnitten wird
            TabButton { text: "Inhalt"; width: implicitWidth; leftPadding: 10; rightPadding: 10 }
            TabButton { text: "Lesezeichen"; width: implicitWidth; leftPadding: 10; rightPadding: 10 }
            TabButton { text: "Anmerkungen"; width: implicitWidth; leftPadding: 10; rightPadding: 10 }
            TabButton { text: "Suche"; width: implicitWidth; leftPadding: 10; rightPadding: 10 }
        }

        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: reiterLeiste.currentIndex

            // ------------------------------------------------------------ Inhalt (Gliederung des PDFs)
            ListView {
                id: gliederung
                clip: true
                model: leiste.fenster.nach(Dokument.generation, JSON.parse(Dokument.gliederungJson()))
                ScrollBar.vertical: ScrollBar {}
                delegate: ItemDelegate {
                    required property var modelData
                    width: gliederung.width
                    leftPadding: 8 + modelData.tiefe * 14
                    text: modelData.titel.trim() + (modelData.seite >= 0 ? "   ·  " + (modelData.seite + 1) : "")
                    font.bold: modelData.tiefe === 0
                    enabled: modelData.seite >= 0
                    onClicked: leiste.fenster.springeMitVerlauf(modelData.seite, modelData.y, null)
                }
                Label {
                    anchors.centerIn: parent
                    visible: gliederung.count === 0
                    text: Dokument.seitenzahl > 0 ? "Dieses PDF hat keine Gliederung." : ""
                    color: palette.placeholderText
                }
            }

            // ------------------------------------------------------------ Lesezeichen
            ColumnLayout {
                spacing: 4
                Button {
                    Layout.fillWidth: true
                    Layout.margins: 4
                    text: leiste.fenster.tipp("Lesezeichen hier setzen", "lesezeichen_setzen")
                    enabled: Dokument.seitenzahl > 0
                    onClicked: leiste.fenster.aktion("lesezeichen_setzen")
                }
                ListView {
                    id: lesezeichen
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: leiste.fenster.nach(Dokument.lesezeichenVersion, JSON.parse(Dokument.lesezeichenJson()))
                    ScrollBar.vertical: ScrollBar {}
                    delegate: ItemDelegate {
                        id: lzEintrag
                        required property var modelData
                        required property int index
                        width: lesezeichen.width
                        contentItem: ColumnLayout {
                            spacing: 0
                            Label { text: lzEintrag.modelData.name; elide: Text.ElideRight; Layout.fillWidth: true; font.bold: true }
                            Label {
                                text: "Seite " + (lzEintrag.modelData.seite + 1) + "  ·  " + lzEintrag.modelData.zeit
                                font.pointSize: 8
                                color: palette.placeholderText
                            }
                        }
                        onClicked: leiste.fenster.springeMitVerlauf(lzEintrag.modelData.seite, lzEintrag.modelData.y, null)
                        TapHandler {
                            acceptedButtons: Qt.RightButton
                            onTapped: lzMenue.popup()
                        }
                        Menu {
                            id: lzMenue
                            MenuItem {
                                text: "Umbenennen …"
                                onTriggered: leiste.fenster.texteingabe("Lesezeichen umbenennen", lzEintrag.modelData.name, false,
                                    function(t) { Dokument.lesezeichenUmbenennen(lzEintrag.index, t) })
                            }
                            MenuItem { text: "Entfernen"; onTriggered: Dokument.lesezeichenEntfernen(lzEintrag.index) }
                        }
                    }
                    Label {
                        anchors.centerIn: parent
                        width: parent.width - 20
                        horizontalAlignment: Text.AlignHCenter
                        wrapMode: Text.WordWrap
                        visible: lesezeichen.count === 0 && Dokument.seitenzahl > 0
                        text: "Noch keine Lesezeichen.\nRechtsklick auf einen Eintrag: umbenennen oder entfernen."
                        color: palette.placeholderText
                    }
                }
            }

            // ------------------------------------------------------------ Anmerkungen
            ColumnLayout {
                spacing: 4
                TextField {
                    id: filter
                    Layout.fillWidth: true
                    Layout.margins: 4
                    placeholderText: "Anmerkungen filtern …"
                }
                ListView {
                    id: anmerkungen
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    readonly property var alle: leiste.fenster.nach(Dokument.anmerkungenVersion, JSON.parse(Dokument.anmerkungenJson()))
                    model: {
                        var f = filter.text.toLowerCase()
                        if (!f) return alle
                        return alle.filter(function(a) {
                            return (a.inhalt + " " + a.typ + " " + a.autor).toLowerCase().indexOf(f) >= 0
                        })
                    }
                    ScrollBar.vertical: ScrollBar {}
                    delegate: ItemDelegate {
                        id: amEintrag
                        required property var modelData
                        width: anmerkungen.width
                        contentItem: RowLayout {
                            spacing: 8
                            Rectangle {
                                Layout.preferredWidth: 12
                                Layout.preferredHeight: 12
                                radius: 2
                                color: amEintrag.modelData.farbe || leiste.fenster.typFarbe(amEintrag.modelData.typ)
                                border.color: "#60000000"
                            }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 0
                                Label {
                                    Layout.fillWidth: true
                                    text: amEintrag.modelData.typ + "  ·  Seite " + (amEintrag.modelData.seite + 1)
                                    font.bold: true
                                }
                                Label {
                                    Layout.fillWidth: true
                                    visible: amEintrag.modelData.inhalt !== ""
                                    text: amEintrag.modelData.inhalt
                                    elide: Text.ElideRight
                                    maximumLineCount: 2
                                    wrapMode: Text.WordWrap
                                }
                                Label {
                                    text: [amEintrag.modelData.autor, amEintrag.modelData.datum].filter(function(x) { return x }).join("  ·  ")
                                    font.pointSize: 8
                                    color: palette.placeholderText
                                }
                            }
                        }
                        onClicked: leiste.fenster.springeMitVerlauf(amEintrag.modelData.seite, null, amEintrag.modelData.rect)
                        onDoubleClicked: leiste.fenster.anmerkungBearbeiten(amEintrag.modelData)
                        TapHandler {
                            acceptedButtons: Qt.RightButton
                            onTapped: amMenue.popup()
                        }
                        Menu {
                            id: amMenue
                            MenuItem { text: "Kommentar bearbeiten …"; onTriggered: leiste.fenster.anmerkungBearbeiten(amEintrag.modelData) }
                            MenuItem {
                                text: "Löschen"
                                onTriggered: leiste.fenster.meldung(Dokument.loeschen(amEintrag.modelData.seite, amEintrag.modelData.index))
                            }
                        }
                    }
                    Label {
                        anchors.centerIn: parent
                        width: parent.width - 20
                        horizontalAlignment: Text.AlignHCenter
                        wrapMode: Text.WordWrap
                        visible: anmerkungen.count === 0 && Dokument.seitenzahl > 0
                        text: anmerkungen.alle.length === 0 ? "Keine Anmerkungen in diesem PDF." : "Nichts passt zum Filter."
                        color: palette.placeholderText
                    }
                }
                Label {
                    Layout.margins: 4
                    text: anmerkungen.alle.length + " Anmerkungen" + (Dokument.geaendert ? "  ·  ungespeichert" : "")
                    font.pointSize: 8
                    color: palette.placeholderText
                }
            }

            // ------------------------------------------------------------ Suche
            ColumnLayout {
                spacing: 4
                TabBar {
                    id: suchArt
                    Layout.fillWidth: true
                    TabButton { text: "In diesem Dokument" }
                    TabButton { text: "In einem Ordner" }
                }

                StackLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    currentIndex: suchArt.currentIndex

                    // -------------------------------------------- im Dokument
                    ColumnLayout {
                        spacing: 4
                        TextField {
                            Layout.fillWidth: true
                            Layout.margins: 4
                            placeholderText: "Im Dokument suchen …"
                            text: leiste.fenster.suchText
                            onTextEdited: leiste.fenster.sucheEingabe(text)
                            Keys.onReturnPressed: (e) => leiste.fenster.trefferSpringen(e.modifiers & Qt.ShiftModifier ? -1 : +1, null)
                            Keys.onEnterPressed: (e) => leiste.fenster.trefferSpringen(e.modifiers & Qt.ShiftModifier ? -1 : +1, null)
                        }
                        Flow {
                            Layout.fillWidth: true
                            Layout.leftMargin: 4
                            spacing: 8
                            CheckBox {
                                text: "Groß/klein beachten"
                                checked: leiste.fenster.sucheGrossKlein
                                onToggled: leiste.fenster.sucheOptionSetzen(checked, leiste.fenster.sucheGanzeWoerter)
                            }
                            CheckBox {
                                text: "Nur ganze Wörter"
                                checked: leiste.fenster.sucheGanzeWoerter
                                onToggled: leiste.fenster.sucheOptionSetzen(leiste.fenster.sucheGrossKlein, checked)
                            }
                        }
                        Label {
                            Layout.leftMargin: 6
                            font.pointSize: 8
                            color: palette.placeholderText
                            text: Dokument.sucheLaeuft
                                  ? "Suche läuft: " + Math.round(Dokument.sucheFortschritt * 100) + " %, " + Dokument.trefferAnzahl + " Treffer"
                                  : (Dokument.trefferAnzahl > 0 ? Dokument.trefferAnzahl + " Treffer" + (Dokument.aktuellerTreffer >= 0 ? ", ausgewählt: " + (Dokument.aktuellerTreffer + 1) : "")
                                                                : (leiste.fenster.suchText.trim() ? "Keine Treffer" : "Suchbegriff eingeben: Treffer erscheinen hier mit ihrem Zusammenhang."))
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                        }
                        ListView {
                            id: trefferListe
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            clip: true
                            model: leiste.trefferZahl
                            ScrollBar.vertical: ScrollBar {}
                            delegate: ItemDelegate {
                                id: tEintrag
                                required property int index
                                readonly property var t: JSON.parse(Dokument.trefferEintrag(index) || "null")
                                width: trefferListe.width
                                highlighted: index === Dokument.aktuellerTreffer
                                contentItem: ColumnLayout {
                                    spacing: 1
                                    Label {
                                        text: tEintrag.t ? "Seite " + (tEintrag.t.seite + 1) : ""
                                        font.pointSize: 8
                                        font.bold: true
                                        color: palette.placeholderText
                                    }
                                    Label {
                                        Layout.fillWidth: true
                                        textFormat: Text.StyledText
                                        text: tEintrag.t ? leiste.zusammenhang(tEintrag.t.vor, tEintrag.t.wort, tEintrag.t.nach) : ""
                                        wrapMode: Text.WordWrap
                                        maximumLineCount: 3
                                        elide: Text.ElideRight
                                    }
                                }
                                onClicked: leiste.fenster.trefferWaehlen(index)
                            }
                        }
                    }

                    // -------------------------------------------- in einem Ordner
                    ColumnLayout {
                        spacing: 4
                        RowLayout {
                            Layout.fillWidth: true
                            Layout.margins: 4
                            spacing: 4
                            TextField {
                                id: ordnerFeld
                                Layout.fillWidth: true
                                placeholderText: "Ordner …"
                                onAccepted: leiste.ordnerSuchen()
                                ToolTip.visible: hovered && text
                                ToolTip.text: text
                            }
                            Button {
                                text: "Wählen …"
                                onClicked: leiste.ordnerWaehlen()
                            }
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            Layout.leftMargin: 4
                            Layout.rightMargin: 4
                            spacing: 4
                            TextField {
                                id: ordnerSuchfeld
                                Layout.fillWidth: true
                                placeholderText: "Suchbegriff, dann Enter"
                                onAccepted: leiste.ordnerSuchen()
                            }
                            Button {
                                text: Dokument.ordnerLaeuft ? "Stopp" : "Suchen"
                                enabled: Dokument.ordnerLaeuft || (ordnerFeld.text !== "" && ordnerSuchfeld.text.trim() !== "")
                                onClicked: Dokument.ordnerLaeuft ? Dokument.ordnerAbbrechen() : leiste.ordnerSuchen()
                            }
                        }
                        Flow {
                            Layout.fillWidth: true
                            Layout.leftMargin: 4
                            spacing: 8
                            CheckBox { id: unterordner; text: "Unterordner"; checked: true }
                            CheckBox { id: ordnerGrossKlein; text: "Groß/klein beachten" }
                            CheckBox { id: ordnerGanzeWoerter; text: "Nur ganze Wörter" }
                        }
                        // eigener Balken: der ProgressBar des Breeze-Stils meldet Fehler im Protokoll
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.leftMargin: 6
                            Layout.rightMargin: 6
                            Layout.preferredHeight: 4
                            visible: Dokument.ordnerLaeuft
                            radius: 2
                            color: palette.mid
                            Rectangle {
                                width: parent.width * Dokument.ordnerFortschritt
                                height: parent.height
                                radius: 2
                                color: palette.highlight
                            }
                        }
                        Label {
                            Layout.fillWidth: true
                            Layout.leftMargin: 6
                            Layout.rightMargin: 6
                            text: Dokument.ordnerStatus || "Durchsucht alle PDFs im Ordner. Ein Klick auf einen Treffer öffnet die Datei an dieser Stelle."
                            font.pointSize: 8
                            color: palette.placeholderText
                            wrapMode: Text.WordWrap
                        }
                        ListView {
                            id: ordnerListe
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            clip: true
                            currentIndex: -1
                            model: ListModel { id: ordnerModell }
                            ScrollBar.vertical: ScrollBar {}
                            section.property: "datei"
                            section.delegate: Rectangle {
                                id: kopf
                                required property string section
                                readonly property string ordner: {
                                    var o = leiste.ordnerVon(section)
                                    var w = ordnerFeld.text
                                    return o.indexOf(w) === 0 ? o.slice(w.length).replace(/^[\/\\]/, "") : o
                                }
                                width: ordnerListe.width
                                height: kopfText.implicitHeight + 10
                                color: palette.alternateBase
                                ColumnLayout {
                                    id: kopfText
                                    anchors.verticalCenter: parent.verticalCenter
                                    x: 6
                                    width: parent.width - 12
                                    spacing: 0
                                    Label {
                                        Layout.fillWidth: true
                                        text: Dokument.dateiname(kopf.section)
                                        font.bold: true
                                        elide: Text.ElideMiddle
                                        color: Dokument.pfad === kopf.section ? palette.highlight : palette.text
                                    }
                                    Label {
                                        Layout.fillWidth: true
                                        visible: kopf.ordner !== ""
                                        text: kopf.ordner
                                        font.pointSize: 8
                                        elide: Text.ElideMiddle
                                        color: palette.placeholderText
                                    }
                                }
                            }
                            delegate: ItemDelegate {
                                id: oEintrag
                                required property int index
                                required property int seite
                                required property string vor
                                required property string wort
                                required property string nach
                                width: ordnerListe.width
                                highlighted: ListView.isCurrentItem
                                contentItem: RowLayout {
                                    spacing: 8
                                    Label {
                                        Layout.alignment: Qt.AlignTop
                                        Layout.preferredWidth: 46
                                        text: "S. " + (oEintrag.seite + 1)
                                        font.pointSize: 8
                                        font.bold: true
                                        color: palette.placeholderText
                                    }
                                    Label {
                                        Layout.fillWidth: true
                                        textFormat: Text.StyledText
                                        text: leiste.zusammenhang(oEintrag.vor, oEintrag.wort, oEintrag.nach)
                                        wrapMode: Text.WordWrap
                                        maximumLineCount: 3
                                        elide: Text.ElideRight
                                    }
                                }
                                onClicked: leiste.ordnerTrefferKlick(index)
                            }
                        }
                    }
                }
            }
        }
    }
}
