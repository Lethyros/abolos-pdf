// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import AbolosPdf

// Jede Aktion: Tasten anklicken = neu belegen, × = entfernen, + = weitere Taste, ↺ = Standard
Dialog {
    id: dialog
    title: "Tastenbelegung"
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(Overlay.overlay ? Overlay.overlay.width - 60 : 760, 760)
    height: Overlay.overlay ? Overlay.overlay.height - 60 : 700
    standardButtons: Dialog.Close

    property var aktionen: fenster.nach(Tasten.version, JSON.parse(Tasten.aktionen()))
    // laufende Aufnahme: Aktion und die Taste, die ersetzt wird ("" = neue Taste)
    property string aufnahmeAktion: ""
    property string aufnahmeAlt: ""
    property string meldung: ""

    onClosed: aufnahmeAktion = ""

    function aufnehmen(aktion, alt) {
        aufnahmeAktion = aktion
        aufnahmeAlt = alt
        meldung = "Jetzt die gewünschte Taste oder Tastenkombination drücken (Esc bricht ab)."
        fang.forceActiveFocus()
    }

    // Tastendruck in die Schreibweise von QKeySequence übersetzen ("Ctrl+Shift+K")
    function tastenText(e) {
        var k = e.key
        if (k === Qt.Key_Control || k === Qt.Key_Shift || k === Qt.Key_Alt || k === Qt.Key_Meta
                || k === Qt.Key_AltGr || k === 0 || k === Qt.Key_unknown)
            return ""
        var namen = {}
        namen[Qt.Key_PageDown] = "PgDown"; namen[Qt.Key_PageUp] = "PgUp"
        namen[Qt.Key_Home] = "Home"; namen[Qt.Key_End] = "End"
        namen[Qt.Key_Left] = "Left"; namen[Qt.Key_Right] = "Right"; namen[Qt.Key_Up] = "Up"; namen[Qt.Key_Down] = "Down"
        namen[Qt.Key_Space] = "Space"; namen[Qt.Key_Return] = "Return"; namen[Qt.Key_Enter] = "Enter"
        namen[Qt.Key_Escape] = "Esc"; namen[Qt.Key_Tab] = "Tab"; namen[Qt.Key_Backtab] = "Tab"
        namen[Qt.Key_Backspace] = "Backspace"; namen[Qt.Key_Delete] = "Del"; namen[Qt.Key_Insert] = "Ins"
        namen[Qt.Key_Plus] = "+"; namen[Qt.Key_Minus] = "-"; namen[Qt.Key_Equal] = "="
        namen[Qt.Key_Comma] = ","; namen[Qt.Key_Period] = "."; namen[Qt.Key_Slash] = "/"
        var name = namen[k]
        var symbol = false
        if (!name) {
            if (k >= Qt.Key_F1 && k <= Qt.Key_F35) name = "F" + (k - Qt.Key_F1 + 1)
            else if ((k >= Qt.Key_A && k <= Qt.Key_Z) || (k >= Qt.Key_0 && k <= Qt.Key_9)) name = String.fromCharCode(k)
            else if (e.text && e.text.length === 1 && e.text.charCodeAt(0) > 32) { name = e.text.toUpperCase(); symbol = true }
            else return ""
        }
        if (k === Qt.Key_Plus || k === Qt.Key_Minus || k === Qt.Key_Equal || k === Qt.Key_Comma
                || k === Qt.Key_Period || k === Qt.Key_Slash)
            symbol = true
        var m = ""
        if (e.modifiers & Qt.ControlModifier) m += "Ctrl+"
        // Bei Zeichen, die Shift selbst erzeugt (z.B. "/" auf deutscher Tastatur), Shift weglassen
        if ((e.modifiers & Qt.ShiftModifier) && !symbol) m += "Shift+"
        if (e.modifiers & Qt.AltModifier) m += "Alt+"
        if (e.modifiers & Qt.MetaModifier) m += "Meta+"
        return m + name
    }

    Item {
        id: fang
        focus: dialog.aufnahmeAktion !== ""
        Keys.onPressed: (e) => {
            if (dialog.aufnahmeAktion === "") return
            var t = dialog.tastenText(e)
            if (t === "") return
            e.accepted = true
            if (t === "Esc") {
                dialog.meldung = ""
            } else {
                var vorher = Tasten.belegen(dialog.aufnahmeAktion, dialog.aufnahmeAlt, t)
                dialog.meldung = vorher ? "„" + t + "“ gehörte zu „" + vorher + "“ und wurde dort entfernt." : "„" + t + "“ belegt."
            }
            dialog.aufnahmeAktion = ""
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 6

        Label {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "Taste anklicken zum Ändern, × entfernt sie, + fügt eine weitere hinzu, ↺ stellt den Standard her. "
                  + "Gespeichert in " + Tasten.datei()
        }

        ListView {
            id: liste
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: dialog.aktionen
            ScrollBar.vertical: ScrollBar {}
            delegate: ColumnLayout {
                id: zeile
                required property var modelData
                required property int index
                width: liste.width - 16
                spacing: 2

                Label {
                    visible: zeile.index === 0 || dialog.aktionen[zeile.index - 1].gruppe !== zeile.modelData.gruppe
                    text: zeile.modelData.gruppe
                    font.bold: true
                    topPadding: 10
                }

                RowLayout {
                    Layout.fillWidth: true
                    Label {
                        text: zeile.modelData.name
                        Layout.preferredWidth: 250
                        elide: Text.ElideRight
                    }
                    Repeater {
                        model: zeile.modelData.tasten
                        delegate: RowLayout {
                            required property string modelData
                            spacing: 0
                            Button {
                                text: dialog.aufnahmeAktion === zeile.modelData.id && dialog.aufnahmeAlt === modelData
                                      ? "Taste drücken …" : modelData
                                highlighted: dialog.aufnahmeAktion === zeile.modelData.id && dialog.aufnahmeAlt === modelData
                                onClicked: dialog.aufnehmen(zeile.modelData.id, modelData)
                            }
                            ToolButton {
                                text: "×"
                                onClicked: Tasten.entfernen(zeile.modelData.id, parent.modelData)
                                ToolTip.visible: hovered
                                ToolTip.text: "Taste entfernen"
                            }
                        }
                    }
                    Button {
                        text: dialog.aufnahmeAktion === zeile.modelData.id && dialog.aufnahmeAlt === "" ? "Taste drücken …" : "+"
                        highlighted: dialog.aufnahmeAktion === zeile.modelData.id && dialog.aufnahmeAlt === ""
                        onClicked: dialog.aufnehmen(zeile.modelData.id, "")
                        ToolTip.visible: hovered
                        ToolTip.text: "Weitere Taste hinzufügen"
                    }
                    ToolButton {
                        text: "↺"
                        onClicked: Tasten.zuruecksetzen(zeile.modelData.id)
                        ToolTip.visible: hovered
                        ToolTip.text: "Standard: " + zeile.modelData.standard.join(", ")
                    }
                    Item { Layout.fillWidth: true }
                }
            }
        }

        Label {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: dialog.meldung
            color: dialog.aufnahmeAktion !== "" ? palette.highlight : palette.windowText
        }

        Button {
            text: "Alle auf Standard zurücksetzen"
            onClicked: { Tasten.zuruecksetzen(""); dialog.meldung = "Alle Tasten auf Standard zurückgesetzt." }
        }
    }
}
