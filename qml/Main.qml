// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import QtQuick.Window
import AbolosPdf

ApplicationWindow {
    id: fenster
    width: 1150
    height: 880
    visible: true
    title: Dokument.titel ? (Dokument.geaendert ? "● " : "") + Dokument.titel + " – Abolos PDF" : "Abolos PDF"

    // Zoom 1.0 = 100 % = Originalgröße (1 pt = 1/72 Zoll bei 96 dpi)
    property real zoom: 1.0
    readonly property real pxProPt: zoom * 96 / 72
    readonly property var zoomStufen: [0.1, 0.25, 0.33, 0.5, 0.67, 0.75, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2, 2.5, 3, 4, 5, 6, 8]
    readonly property int aktuelleSeite: fenster.nach(Dokument.layoutVersion, Dokument.seitenzahl > 0
        ? Dokument.seiteBei((ansicht.contentY + ansicht.height * 0.3) / pxProPt) : 0)
    // Suche: Position beim Beginn des Tippens; alle Zwischen-Suchen gehen von hier aus
    property var sucheAnker: null
    // Seite unter dem Rechtsklick (für "nur diese Seite drehen")
    property int kontextSeite: -1

    // ---- Bearbeiten
    property string werkzeug: "hand"
    readonly property var werkzeugListe: [
        { id: "hand", text: "✋", name: "Hand / Auswahl" },
        { id: "hervorheben", text: "▆", name: "Hervorheben" },
        { id: "unterstreichen", text: "U̲", name: "Unterstreichen" },
        { id: "durchstreichen", text: "S̶", name: "Durchstreichen" },
        { id: "freihand", text: "✎", name: "Stift (Freihand)" },
        { id: "rechteck", text: "▭", name: "Rechteck" },
        { id: "ellipse", text: "◯", name: "Ellipse" },
        { id: "linie", text: "╱", name: "Linie (mit Shift: waagrecht, senkrecht, 45°)" },
        { id: "pfeil", text: "➚", name: "Pfeil (mit Shift: waagrecht, senkrecht, 45°)" },
        { id: "notiz", text: "🗨", name: "Notiz" },
        { id: "textfeld", text: "T", name: "Textfeld" },
        { id: "bild", text: "🖼", name: "Bild einfügen" },
        { id: "radierer", text: "⌫", name: "Anmerkung löschen" }
    ]
    // Farbe je Werkzeug (wird mit der Farbwahl geändert)
    property var farben: ({ hervorheben: "#ffd400", unterstreichen: "#0060e0", durchstreichen: "#e00000",
                            freihand: "#e00000", rechteck: "#0060e0", ellipse: "#00a000", notiz: "#ffd400",
                            linie: "#e00000", pfeil: "#e00000",
                            textfeld: "#c00000" })
    property real stiftBreite: 2      // Punkte
    property real schriftGroesse: 11  // Punkte
    property bool leisteSichtbar: true
    property string meldungText: ""

    function aktuelleFarbe() { return farben[werkzeug] || "#e00000" }

    function farbeSetzen(f) {
        var neu = Object.assign({}, farben)
        neu[werkzeug] = f
        farben = neu
    }

    function typFarbe(typ) {
        switch (typ) {
        case "Hervorhebung": return "#ffd400"
        case "Unterstreichung": return "#0060e0"
        case "Durchstreichung": return "#e00000"
        case "Notiz": return "#ffd400"
        case "Bild": return "#808080"
        default: return "#a0a0a0"
        }
    }

    function meldung(text) {
        if (!text) return
        meldungText = text
        meldungTimer.restart()
    }

    function werkzeugSetzen(w) {
        werkzeug = w
        if (w !== "hand") markierung.entfernen()
    }

    // Texteingabe-Dialog; fertig(text) wird nur bei OK aufgerufen
    function texteingabe(titel, vorgabe, mehrzeilig, fertig) {
        texteingabeDialog.title = titel
        texteingabeDialog.mehrzeilig = mehrzeilig
        texteingabeDialog.fertig = fertig
        texteingabeDialog.vorgabe = vorgabe
        texteingabeDialog.open()
    }

    function anmerkungBearbeiten(a) {
        texteingabe("Kommentar: " + a.typ + " auf Seite " + (a.seite + 1), a.inhalt, true, function(t) {
            meldung(Dokument.inhaltSetzen(a.seite, a.index, t))
        })
    }

    // Sprung aus der Seitenleiste: mit Verlauf und roter Markierung (Rechteck oder Höhe)
    function springeMitVerlauf(seite, y, rect) {
        var hier = position()
        verlauf = verlauf.slice(0, verlaufPos + 1)
        if (verlaufPos < 0) verlauf.push(hier); else verlauf[verlaufPos] = hier
        if (rect) {
            springeZu(seite, rect.y, false)
            markierung.setzenRechteck(seite, rect)
        } else {
            var yAnz = (y === null || y === undefined) ? null : Dokument.anzeigeY(seite, y)
            springeZu(seite, (yAnz === null || yAnz < 0) ? null : yAnz, false)
            if (y !== null && y !== undefined) markierung.setzen(seite, y); else markierung.entfernen()
        }
        verlauf.push(position())
        verlaufPos = verlauf.length - 1
    }

    // Ordner einer Datei als URL (für die Startordner der Dateidialoge)
    function ordnerUrl(pfad) {
        return pfad ? Dokument.dateiUrl(Dokument.ordnerVon(pfad)) : ""
    }

    // Treffer der Ordnersuche öffnen: Datei laden (falls nötig), Fundstelle rot umrahmen,
    // alle Vorkommen im Dokument gelb markieren
    function ordnerTrefferOeffnen(t) {
        var weiter = function() {
            if (Dokument.pfad !== t.datei) {
                oeffneDatei(t.datei)
                if (Dokument.pfad !== t.datei) return
            }
            var rect = Dokument.trefferRechteck(t.seite, t.suchtext, t.grossKlein, t.ganzeWoerter, t.nr)
            springeMitVerlauf(t.seite, null, rect ? JSON.parse(rect) : null)
            if (!rect) markierung.setzen(t.seite, null)
            sucheVerzoegert.stop()
            sucheAnker = null
            sucheFeld.text = t.suchtext
            sucheGrossKlein = t.grossKlein
            sucheGanzeWoerter = t.ganzeWoerter
            Dokument.suchen(t.suchtext, t.grossKlein, t.ganzeWoerter, t.seite)
            ersterTrefferOffen = false
        }
        if (Dokument.pfad === t.datei) weiter(); else sicherFortfahren(weiter)
    }

    function speichern() {
        var f = Dokument.speichern("")
        meldung(f ? f : "Gespeichert: " + Dokument.titel)
        return f === ""
    }

    // nach dem Speichern-Dialog: was danach passieren soll (schließen, andere Datei öffnen)
    property var nachSpeichern: null

    function sicherFortfahren(weiter) {
        if (!Dokument.geaendert) { weiter(); return }
        nachSpeichern = weiter
        schliessenDialog.open()
    }

    // Verlauf für Zurück/Vorwärts nach Link-Sprüngen: absolute Positionen in Punkten
    property var verlauf: []
    property int verlaufPos: -1
    // nach Suchstart automatisch zum ersten Treffer
    property bool ersterTrefferOffen: false
    // im Hilfefenster gelten dessen eigene Tasten (Esc, Strg+F)
    readonly property bool tastenGesperrt: tastenDialog.visible || externDialog.visible || hilfeFenster.active

    // Abhängigkeit für Bindungen: `nach(Dokument.layoutVersion, wert)` rechnet neu, wenn sich die
    // Version ändert. (Der Komma-Trick "(version, wert)" wird vom QML-Compiler wegoptimiert.)
    function nach(version, wert) { return wert }

    // gemerkter Zoom-Modus: "" = feste Stufe, "breite" = Seitenbreite, "seite" = ganze Seite
    property string zoomModus: "breite"

    function zoomAnpassen() {
        if (zoomModus === "breite") seitenbreite()
        else if (zoomModus === "seite") ganzeSeite()
    }

    function erstesTaste(id) {
        var liste = fenster.nach(Tasten.version, JSON.parse(Tasten.aktionen()))
        for (var i = 0; i < liste.length; i++)
            if (liste[i].id === id && liste[i].tasten.length > 0)
                return liste[i].tasten[0]
        return ""
    }

    function tipp(text, id) {
        var t = erstesTaste(id)
        return t ? text + "  (" + Tasten.anzeigename(t) + ")" : text
    }

    function aktion(id) {
        switch (id) {
        case "datei_oeffnen": sicherFortfahren(function() { dateiDialog.open() }); break
        case "speichern": if (Dokument.seitenzahl > 0) speichern(); break
        case "speichern_unter": if (Dokument.seitenzahl > 0) speichernDialog.open(); break
        case "rueckgaengig": meldung(Dokument.rueckgaengig()); break
        case "seitenleiste": leisteSichtbar = !leisteSichtbar; break
        case "lesezeichen_setzen": {
            if (Dokument.seitenzahl === 0) break
            var p = hierPosition()
            texteingabe("Lesezeichen setzen", "Seite " + (p.seite + 1), false, function(t) {
                Dokument.lesezeichenSetzen(p.seite, Math.max(0, p.y), t)
                leisteSichtbar = true
            })
            break
        }
        case "beenden": fenster.close(); break
        case "bildschirm_runter": scrollen(0, ansicht.height * 0.9); break
        case "bildschirm_hoch": scrollen(0, -ansicht.height * 0.9); break
        case "naechste_seite": springeZu(Dokument.naechsteZeile(aktuelleSeite), null, false); break
        case "vorige_seite": springeZu(Dokument.vorigeZeile(aktuelleSeite), null, false); break
        case "erste_seite": springeZu(0, null, false); break
        case "letzte_seite": springeZu(Dokument.seitenzahl - 1, null, false); break
        case "runter": scrollen(0, 60); break
        case "hoch": scrollen(0, -60); break
        case "links": scrollen(-60, 0); break
        case "rechts": scrollen(60, 0); break
        case "gehe_zu_seite": seitenFeld.forceActiveFocus(); seitenFeld.selectAll(); break
        case "zurueck": zurueck(); break
        case "vorwaerts": vorwaerts(); break
        case "zoom_rein": zoomModus = ""; zoomStufe(+1); break
        case "zoom_raus": zoomModus = ""; zoomStufe(-1); break
        case "zoom_100": zoomModus = ""; setzeZoom(1.0); break
        case "ganze_seite": zoomModus = "seite"; ganzeSeite(); break
        case "seitenbreite": zoomModus = "breite"; seitenbreite(); break
        case "vollbild": fenster.visibility = fenster.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen; break
        case "suchen": sucheFeld.forceActiveFocus(); sucheFeld.selectAll(); break
        case "naechster_treffer": trefferSpringen(+1, null); break
        case "voriger_treffer": trefferSpringen(-1, null); break
        case "suche_schliessen": sucheSchliessen(); markierung.entfernen(); werkzeugSetzen("hand"); break
        case "ordner_suchen": leisteSichtbar = true; seitenleiste.zeigeSuche(true); break
        case "suche_gross_klein": sucheOptionSetzen(!sucheGrossKlein, sucheGanzeWoerter); meldung(sucheGrossKlein ? "Groß-/Kleinschreibung wird beachtet" : "Groß-/Kleinschreibung egal"); break
        case "suche_ganze_woerter": sucheOptionSetzen(sucheGrossKlein, !sucheGanzeWoerter); meldung(sucheGanzeWoerter ? "Nur ganze Wörter" : "Auch Wortteile"); break
        case "einzelseite": setzeModus(0); break
        case "doppelseite": setzeModus(Dokument.modus === 0 ? 1 : 0); break
        case "deckblatt": setzeModus(Dokument.modus === 2 ? 1 : 2); break
        case "drehen_rechts": drehen(90, -1); break
        case "drehen_links": drehen(-90, -1); break
        case "seite_drehen_rechts": drehen(90, kontextSeite >= 0 ? kontextSeite : aktuelleSeite); break
        case "seite_drehen_links": drehen(-90, kontextSeite >= 0 ? kontextSeite : aktuelleSeite); break
        case "tastenbelegung": tastenDialog.open(); break
        case "hilfe": hilfeFenster.zeigen(""); break
        case "ueber": hilfeFenster.zeigen("ueber"); break
        default:
            if (id.indexOf("werkzeug_") === 0) { werkzeugSetzen(id.slice(9)); break }
            console.log("Unbekannte Aktion:", id)
        }
    }

    // ---------------------------------------------------------------- Navigation

    function begrenzen(wert, min, max) { return Math.max(min, Math.min(max, wert)) }

    function scrollen(dx, dy) {
        ansicht.contentX = begrenzen(ansicht.contentX + dx, 0, Math.max(0, ansicht.contentWidth - ansicht.width))
        ansicht.contentY = begrenzen(ansicht.contentY + dy, 0, Math.max(0, ansicht.contentHeight - ansicht.height))
    }

    function seitenX(seite) { return ansicht.contentWidth / 2 + Dokument.seitenX(seite) * pxProPt }

    // Anordnung/Drehung ändern und dabei auf derselben Seite bleiben
    function setzeModus(m) {
        var seite = aktuelleSeite
        Dokument.setzeModus(m)
        zoomAnpassen()
        springeZu(seite, null, false)
    }

    function drehen(grad, seite) {
        var bleiben = aktuelleSeite
        Dokument.drehen(grad, seite)
        kontextSeite = -1
        zoomAnpassen()
        springeZu(bleiben, null, false)
    }

    // yRel: relative Höhe auf der Seite (0..1) oder null = Seitenanfang
    function springeZu(seite, yRel, blinken) {
        if (seite < 0 || seite >= Dokument.seitenzahl)
            return
        var oben = Dokument.seitenOben(seite) * pxProPt
        var h = Dokument.seitenHoehe(seite) * pxProPt
        var y = (yRel === null || yRel === undefined) ? oben - Dokument.seitenabstand() * pxProPt / 2
                                                      : oben + yRel * h - ansicht.height * 0.2
        ansicht.contentY = begrenzen(y, 0, Math.max(0, ansicht.contentHeight - ansicht.height))
        if (blinken)
            blink.zeigen(seite, yRel)
    }

    function position() { return ansicht.contentY / pxProPt }

    function zuPosition(pt) {
        ansicht.contentY = begrenzen(pt * pxProPt, 0, Math.max(0, ansicht.contentHeight - ansicht.height))
    }

    function linkFolgen(link) {
        if (link.typ === "intern") {
            var hier = position()
            verlauf = verlauf.slice(0, verlaufPos + 1)
            if (verlaufPos < 0) verlauf.push(hier); else verlauf[verlaufPos] = hier
            var y = (link.y === null || link.y === undefined) ? null : Dokument.anzeigeY(link.seite, link.y)
            springeZu(link.seite, (y === null || y < 0) ? null : y, false)
            markierung.setzen(link.seite, link.y)
            verlauf.push(position())
            verlaufPos = verlauf.length - 1
        } else {
            externDialog.zeigen(link.uri)
        }
    }

    function zurueck() {
        if (verlaufPos <= 0) return
        verlauf[verlaufPos] = position()
        verlaufPos--
        zuPosition(verlauf[verlaufPos])
    }

    function vorwaerts() {
        if (verlaufPos < 0 || verlaufPos >= verlauf.length - 1) return
        verlauf[verlaufPos] = position()
        verlaufPos++
        zuPosition(verlauf[verlaufPos])
    }

    // ---------------------------------------------------------------- Zoom

    function setzeZoom(z) {
        z = begrenzen(z, 0.05, 10)
        var mitte = (ansicht.contentY + ansicht.height / 2) / pxProPt
        var mitteX = ansicht.contentWidth > 0 ? (ansicht.contentX + ansicht.width / 2) / ansicht.contentWidth : 0.5
        zoom = z
        ansicht.contentY = begrenzen(mitte * pxProPt - ansicht.height / 2, 0, Math.max(0, ansicht.contentHeight - ansicht.height))
        ansicht.contentX = begrenzen(mitteX * ansicht.contentWidth - ansicht.width / 2, 0, Math.max(0, ansicht.contentWidth - ansicht.width))
    }

    function zoomStufe(richtung) {
        var i
        if (richtung > 0) {
            for (i = 0; i < zoomStufen.length; i++)
                if (zoomStufen[i] > zoom + 0.001) { setzeZoom(zoomStufen[i]); return }
        } else {
            for (i = zoomStufen.length - 1; i >= 0; i--)
                if (zoomStufen[i] < zoom - 0.001) { setzeZoom(zoomStufen[i]); return }
        }
    }

    function seitenbreite() {
        if (Dokument.seitenzahl === 0) return
        var seite = aktuelleSeite
        setzeZoom((ansicht.width - 40) / (Dokument.zeilenBreite(seite) * 96 / 72))
    }

    function ganzeSeite() {
        if (Dokument.seitenzahl === 0) return
        var seite = aktuelleSeite
        setzeZoom(Math.min((ansicht.width - 40) / (Dokument.zeilenBreite(seite) * 96 / 72),
                           (ansicht.height - 2 * Dokument.seitenabstand()) / (Dokument.zeilenHoehe(seite) * 96 / 72)))
        springeZu(seite, null, false)
    }

    // ---------------------------------------------------------------- Suche

    function hierPosition() {
        var seite = aktuelleSeite
        return { seite: seite, y: (ansicht.contentY / pxProPt - Dokument.seitenOben(seite)) / Dokument.seitenHoehe(seite) }
    }

    // Text der Suche; die Trefferliste in der Seitenleiste bindet sich daran
    property alias suchText: sucheFeld.text
    // Suchoptionen (Werkzeugleiste, Seitenleiste und Tasten schalten dieselben)
    property bool sucheGrossKlein: false
    property bool sucheGanzeWoerter: false

    function sucheOptionSetzen(grossKlein, ganzeWoerter) {
        sucheGrossKlein = grossKlein
        sucheGanzeWoerter = ganzeWoerter
        if (sucheFeld.text) sucheStarten()
    }

    // Eingabe in einem der Suchfelder (Werkzeugleiste oder Seitenleiste)
    function sucheEingabe(text) {
        sucheFeld.text = text
        if (sucheAnker === null)
            sucheAnker = hierPosition()
        sucheVerzoegert.restart()
    }

    function sucheStarten() {
        sucheVerzoegert.stop()
        var ab = sucheAnker !== null ? sucheAnker : hierPosition()
        Dokument.suchen(sucheFeld.text, sucheGrossKlein, sucheGanzeWoerter, ab.seite)
        ersterTrefferOffen = sucheFeld.text.trim().length > 0
        // Trefferliste mit Zusammenhang zeigen
        if (ersterTrefferOffen && Dokument.seitenzahl > 0) {
            leisteSichtbar = true
            seitenleiste.zeigeSuche(false)
        }
    }

    // Klick in der Trefferliste
    function trefferWaehlen(nr) {
        sucheAnker = null
        var antwort = Dokument.trefferWaehlen(nr)
        if (antwort) trefferZeigen(JSON.parse(antwort))
    }

    function trefferZeigen(t) {
        springeZu(t.seite, t.y, false)
        // waagrecht nachführen, falls der Treffer außerhalb liegt
        var x = seitenX(t.seite) + t.x * Dokument.seitenBreite(t.seite) * pxProPt
        if (x < ansicht.contentX || x > ansicht.contentX + ansicht.width - 40)
            ansicht.contentX = begrenzen(x - ansicht.width / 3, 0, Math.max(0, ansicht.contentWidth - ansicht.width))
    }

    // ab: Position, von der aus der erste Treffer gesucht wird (null = aktuelle)
    function trefferSpringen(richtung, ab) {
        if (sucheVerzoegert.running) { sucheStarten(); return }
        if (ab === undefined || ab === null) {
            sucheAnker = null // ab jetzt blättert der Nutzer selbst durch die Treffer
            ab = hierPosition()
        }
        var antwort = Dokument.trefferSpringen(richtung, ab.seite, ab.y)
        if (antwort) trefferZeigen(JSON.parse(antwort))
    }

    function sucheSchliessen() {
        sucheVerzoegert.stop()
        sucheAnker = null
        sucheFeld.text = ""
        Dokument.sucheAbbrechen()
        ansicht.forceActiveFocus()
    }

    property bool schliessenErlaubt: false
    onClosing: (close) => {
        if (Dokument.geaendert && !schliessenErlaubt) {
            close.accepted = false
            sicherFortfahren(function() { schliessenErlaubt = true; fenster.close() })
        }
    }

    Timer {
        id: meldungTimer
        interval: 4000
        onTriggered: fenster.meldungText = ""
    }

    Dialog {
        id: schliessenDialog
        title: "Ungespeicherte Änderungen"
        modal: true
        anchors.centerIn: Overlay.overlay
        width: Math.min(fenster.width - 80, 480)
        Label {
            width: parent.width
            wrapMode: Text.WordWrap
            text: "„" + Dokument.titel + "“ enthält ungespeicherte Anmerkungen. Speichern?"
        }
        footer: DialogButtonBox {
            Button { text: "Speichern"; DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole }
            Button { text: "Verwerfen"; DialogButtonBox.buttonRole: DialogButtonBox.DestructiveRole }
            Button { text: "Abbrechen"; DialogButtonBox.buttonRole: DialogButtonBox.RejectRole }
            onAccepted: { schliessenDialog.close(); if (fenster.speichern() && fenster.nachSpeichern) fenster.nachSpeichern() }
            onDiscarded: { schliessenDialog.close(); if (fenster.nachSpeichern) fenster.nachSpeichern() }
            onRejected: schliessenDialog.close()
        }
    }

    Dialog {
        id: texteingabeDialog
        modal: true
        anchors.centerIn: Overlay.overlay
        width: Math.min(fenster.width - 80, 520)
        standardButtons: Dialog.Ok | Dialog.Cancel
        property bool mehrzeilig: false
        property var fertig: null
        property string vorgabe: ""
        onOpened: {
            einzeilig.text = vorgabe
            mehrzeiligFeld.text = vorgabe
            if (mehrzeilig) mehrzeiligFeld.forceActiveFocus(); else { einzeilig.forceActiveFocus(); einzeilig.selectAll() }
        }
        onAccepted: if (fertig) fertig(mehrzeilig ? mehrzeiligFeld.text : einzeilig.text)
        ColumnLayout {
            anchors.fill: parent
            TextField {
                id: einzeilig
                visible: !texteingabeDialog.mehrzeilig
                Layout.fillWidth: true
                onAccepted: texteingabeDialog.accept()
            }
            ScrollView {
                visible: texteingabeDialog.mehrzeilig
                Layout.fillWidth: true
                Layout.preferredHeight: 140
                TextArea { id: mehrzeiligFeld; wrapMode: TextArea.Wrap; placeholderText: "Text eingeben …" }
            }
        }
    }

    // Bild einfügen: Rahmen auf der Seite gezogen, dann Datei wählen
    property var bildRahmen: null
    FileDialog {
        id: bildDialog
        title: "Bild einfügen"
        nameFilters: ["Bilder (*.png *.jpg *.jpeg *.gif *.bmp *.webp *.tif *.tiff)", "Alle Dateien (*)"]
        onAccepted: {
            var r = fenster.bildRahmen
            if (!r) return
            r.pfad = selectedFile.toString()
            fenster.meldung(Dokument.anlegen(JSON.stringify(r)))
        }
    }

    FileDialog {
        id: speichernDialog
        title: "Speichern unter"
        fileMode: FileDialog.SaveFile
        defaultSuffix: "pdf"
        currentFolder: fenster.ordnerUrl(Dokument.pfad)
        nameFilters: ["PDF-Dokumente (*.pdf)"]
        onAccepted: {
            var f = Dokument.speichern(selectedFile.toString())
            fenster.meldung(f ? f : "Gespeichert: " + Dokument.titel)
        }
    }

    Timer {
        id: groesseGeaendert
        interval: 150
        onTriggered: { var s = fenster.aktuelleSeite; fenster.zoomAnpassen(); if (fenster.zoomModus === "seite") fenster.springeZu(s, null, false) }
    }

    Timer {
        id: sucheVerzoegert
        interval: 350
        onTriggered: fenster.sucheStarten()
    }

    Connections {
        target: Dokument
        function onTrefferAnzahlChanged() {
            if (fenster.ersterTrefferOffen && Dokument.trefferAnzahl > 0) {
                fenster.ersterTrefferOffen = false
                fenster.trefferSpringen(+1, fenster.sucheAnker !== null ? fenster.sucheAnker : fenster.hierPosition())
            }
        }
    }

    // ---------------------------------------------------------------- Öffnen

    function oeffneDatei(pfad) {
        if (Dokument.oeffnen(pfad)) {
            verlauf = []
            verlaufPos = -1
            sucheVerzoegert.stop()
            ansicht.contentY = 0
            ansicht.contentX = 0
            zoomAnpassen()
            ansicht.contentY = 0
        }
    }

    FileDialog {
        id: dateiDialog
        title: "PDF öffnen"
        currentFolder: fenster.ordnerUrl(Dokument.pfad)
        nameFilters: ["PDF-Dokumente (*.pdf *.PDF)", "Alle Dateien (*)"]
        onAccepted: fenster.oeffneDatei(selectedFile.toString())
    }

    // ---------------------------------------------------------------- Tasten

    Repeater {
        model: fenster.nach(Tasten.version, JSON.parse(Tasten.aktionen()))
        delegate: Item {
            required property var modelData
            Shortcut {
                sequences: modelData.tasten
                context: Qt.ApplicationShortcut
                enabled: !fenster.tastenGesperrt
                autoRepeat: true
                onActivated: fenster.aktion(modelData.id)
            }
        }
    }

    TastenDialog {
        id: tastenDialog
    }

    HilfeFenster {
        id: hilfeFenster
    }

    // ---------------------------------------------------------------- Externe Links

    Dialog {
        id: externDialog
        title: "Externer Link"
        modal: true
        anchors.centerIn: Overlay.overlay
        width: Math.min(fenster.width - 80, 620)
        standardButtons: Dialog.Close
        property string uri: ""
        function zeigen(adresse) { uri = adresse; open() }
        ColumnLayout {
            anchors.fill: parent
            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: "Dieser Link führt aus dem Dokument hinaus. Abolos PDF öffnet bewusst keine Verbindungen ins Internet."
            }
            TextField {
                id: uriFeld
                Layout.fillWidth: true
                readOnly: true
                text: externDialog.uri
                selectByMouse: true
            }
            Button {
                text: "Adresse kopieren"
                onClicked: { uriFeld.selectAll(); uriFeld.copy(); uriFeld.deselect() }
            }
        }
    }

    // ---------------------------------------------------------------- Oberfläche

    Rectangle {
        anchors.fill: parent
        color: palette.window
    }

    ColumnLayout {
        id: wurzel
        anchors.fill: parent
        spacing: 0

        ToolBar {
            Layout.fillWidth: true
            RowLayout {
                anchors.fill: parent
                spacing: 4

                ToolButton {
                    text: "Öffnen"
                    onClicked: fenster.aktion("datei_oeffnen")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("PDF öffnen", "datei_oeffnen")
                }
                ToolSeparator {}
                ToolButton {
                    text: "◀"
                    enabled: Dokument.seitenzahl > 0
                    onClicked: fenster.aktion("vorige_seite")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Vorige Seite", "vorige_seite")
                }
                TextField {
                    id: seitenFeld
                    Layout.preferredWidth: 64
                    horizontalAlignment: Text.AlignHCenter
                    enabled: Dokument.seitenzahl > 0
                    text: activeFocus ? text : (Dokument.seitenzahl > 0 ? String(fenster.aktuelleSeite + 1) : "")
                    validator: IntValidator { bottom: 1; top: Math.max(1, Dokument.seitenzahl) }
                    onAccepted: { fenster.springeZu(parseInt(text) - 1, null, false); ansicht.forceActiveFocus() }
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Seite eingeben", "gehe_zu_seite")
                }
                Label { text: "/ " + Dokument.seitenzahl }
                ToolButton {
                    text: "▶"
                    enabled: Dokument.seitenzahl > 0
                    onClicked: fenster.aktion("naechste_seite")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Nächste Seite", "naechste_seite")
                }
                ToolSeparator {}
                ToolButton {
                    text: "−"
                    onClicked: fenster.aktion("zoom_raus")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Verkleinern", "zoom_raus")
                }
                Label {
                    text: Math.round(fenster.zoom * 100) + " %"
                    Layout.preferredWidth: 46
                    horizontalAlignment: Text.AlignHCenter
                }
                ToolButton {
                    text: "+"
                    onClicked: fenster.aktion("zoom_rein")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Vergrößern", "zoom_rein")
                }
                ToolButton {
                    text: "Breite"
                    onClicked: fenster.aktion("seitenbreite")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Seitenbreite", "seitenbreite")
                }
                ToolButton {
                    text: "Seite"
                    onClicked: fenster.aktion("ganze_seite")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Ganze Seite", "ganze_seite")
                }
                ToolSeparator {}
                ToolButton {
                    text: "⟲"
                    enabled: Dokument.seitenzahl > 0
                    onClicked: fenster.aktion("drehen_links")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Gegen den Uhrzeigersinn drehen", "drehen_links")
                }
                ToolButton {
                    text: "⟳"
                    enabled: Dokument.seitenzahl > 0
                    onClicked: fenster.aktion("drehen_rechts")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Im Uhrzeigersinn drehen", "drehen_rechts")
                }
                ComboBox {
                    id: modusWahl
                    model: ["Einzelseiten", "Doppelseiten", "Buchansicht"]
                    currentIndex: Dokument.modus
                    onActivated: (i) => fenster.setzeModus(i)
                    implicitContentWidthPolicy: ComboBox.WidestText
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Doppelseiten ein/aus", "doppelseite") + "\nBuchansicht: Doppelseiten, Deckblatt einzeln"
                }
                ToolSeparator {}
                ToolButton {
                    text: "↶"
                    enabled: fenster.verlaufPos > 0
                    onClicked: fenster.aktion("zurueck")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Zurück", "zurueck")
                }
                ToolButton {
                    text: "↷"
                    enabled: fenster.verlaufPos >= 0 && fenster.verlaufPos < fenster.verlauf.length - 1
                    onClicked: fenster.aktion("vorwaerts")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Vorwärts", "vorwaerts")
                }

                Item { Layout.fillWidth: true }

                // schrumpft bei schmalen Fenstern, damit rechts nichts abgeschnitten wird
                TextField {
                    id: sucheFeld
                    Layout.fillWidth: true
                    Layout.minimumWidth: 90
                    Layout.maximumWidth: 260
                    Layout.preferredWidth: 240
                    placeholderText: fenster.tipp("Suchen", "suchen")
                    enabled: Dokument.seitenzahl > 0
                    onTextEdited: fenster.sucheEingabe(text)
                    Keys.onReturnPressed: (e) => fenster.trefferSpringen(e.modifiers & Qt.ShiftModifier ? -1 : +1, null)
                    Keys.onEnterPressed: (e) => fenster.trefferSpringen(e.modifiers & Qt.ShiftModifier ? -1 : +1, null)
                }
                ToolButton {
                    text: "Aa"
                    checkable: true
                    checked: fenster.sucheGrossKlein
                    onToggled: fenster.sucheOptionSetzen(checked, fenster.sucheGanzeWoerter)
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Groß-/Kleinschreibung beachten", "suche_gross_klein")
                }
                ToolButton {
                    text: "Wort"
                    checkable: true
                    checked: fenster.sucheGanzeWoerter
                    onToggled: fenster.sucheOptionSetzen(fenster.sucheGrossKlein, checked)
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Nur ganze Wörter („Schutz“ findet nicht „Schutzleiter“)", "suche_ganze_woerter")
                }
                ToolButton {
                    text: "▲"
                    enabled: Dokument.trefferAnzahl > 0
                    onClicked: fenster.aktion("voriger_treffer")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Voriger Treffer", "voriger_treffer")
                }
                ToolButton {
                    text: "▼"
                    enabled: Dokument.trefferAnzahl > 0
                    onClicked: fenster.aktion("naechster_treffer")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Nächster Treffer", "naechster_treffer")
                }
                Label {
                    Layout.minimumWidth: 36
                    text: Dokument.trefferAnzahl > 0
                          ? (Dokument.aktuellerTreffer >= 0 ? (Dokument.aktuellerTreffer + 1) + " / " : "") + Dokument.trefferAnzahl
                          : (sucheFeld.text && !Dokument.sucheLaeuft && !sucheVerzoegert.running ? "keine" : "")
                }
            }
        }

        ToolBar {
            Layout.fillWidth: true
            visible: Dokument.seitenzahl > 0
            RowLayout {
                anchors.fill: parent
                spacing: 2
                Repeater {
                    model: fenster.werkzeugListe
                    delegate: ToolButton {
                        required property var modelData
                        text: modelData.text
                        checkable: true
                        checked: fenster.werkzeug === modelData.id
                        onClicked: fenster.werkzeugSetzen(modelData.id)
                        ToolTip.visible: hovered
                        ToolTip.text: fenster.tipp(modelData.name, "werkzeug_" + modelData.id)
                        // Werkzeugfarbe als Strich unter dem Knopf
                        Rectangle {
                            visible: fenster.farben[parent.modelData.id] !== undefined
                            anchors.bottom: parent.bottom
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: parent.width * 0.6
                            height: 3
                            color: fenster.farben[parent.modelData.id] || "transparent"
                        }
                    }
                }
                ToolSeparator {}
                Label { text: "Farbe" }
                Repeater {
                    model: ["#ffd400", "#ff8c00", "#e00000", "#c000c0", "#0060e0", "#00a000", "#000000"]
                    delegate: Rectangle {
                        required property string modelData
                        width: 20; height: 20; radius: 3
                        color: modelData
                        border.width: fenster.aktuelleFarbe() === modelData ? 3 : 1
                        border.color: fenster.aktuelleFarbe() === modelData ? palette.highlight : "#60000000"
                        opacity: fenster.farben[fenster.werkzeug] !== undefined ? 1 : 0.35
                        MouseArea { anchors.fill: parent; onClicked: fenster.farbeSetzen(parent.modelData) }
                    }
                }
                ToolButton {
                    text: "…"
                    onClicked: farbDialog.open()
                    ToolTip.visible: hovered
                    ToolTip.text: "Eigene Farbe wählen"
                }
                ToolSeparator {}
                Label { text: fenster.werkzeug === "textfeld" ? "Schrift" : "Stärke" }
                SpinBox {
                    from: 1; to: fenster.werkzeug === "textfeld" ? 72 : 20
                    value: fenster.werkzeug === "textfeld" ? fenster.schriftGroesse : fenster.stiftBreite
                    editable: true
                    onValueModified: if (fenster.werkzeug === "textfeld") fenster.schriftGroesse = value; else fenster.stiftBreite = value
                }
                ToolSeparator {}
                ToolButton {
                    text: "↺ Rückgängig"
                    onClicked: fenster.aktion("rueckgaengig")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Letzte Anmerkung entfernen", "rueckgaengig")
                }
                ToolButton {
                    text: "💾 Speichern"
                    enabled: Dokument.geaendert
                    onClicked: fenster.aktion("speichern")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Ins PDF speichern", "speichern")
                }
                Item { Layout.fillWidth: true }
                ToolButton {
                    text: "?"
                    onClicked: fenster.aktion("hilfe")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Hilfe", "hilfe")
                }
                ToolButton {
                    text: "⌨"
                    onClicked: fenster.aktion("tastenbelegung")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Tastenbelegung", "tastenbelegung")
                }
                ToolButton {
                    text: "☰"
                    checkable: true
                    checked: fenster.leisteSichtbar
                    onClicked: fenster.aktion("seitenleiste")
                    ToolTip.visible: hovered
                    ToolTip.text: fenster.tipp("Seitenleiste", "seitenleiste")
                }
            }
        }

        ColorDialog {
            id: farbDialog
            selectedColor: fenster.aktuelleFarbe()
            onAccepted: fenster.farbeSetzen(String(selectedColor))
        }

        // Seitenleiste und Seiten; die Breite der Leiste lässt sich mit der Maus ziehen
        SplitView {
        Layout.fillWidth: true
        Layout.fillHeight: true
        orientation: Qt.Horizontal

        Seitenleiste {
            id: seitenleiste
            fenster: fenster
            visible: fenster.leisteSichtbar && (Dokument.seitenzahl > 0 || seitenleiste.ordnerSucheOffen)
            SplitView.preferredWidth: 360
            SplitView.minimumWidth: 220
        }

        Flickable {
            id: ansicht
            SplitView.fillWidth: true
            // bleibt bedienbar (Mausrad); beim Zeichnen verhindert preventStealing das Wegziehen
            onWidthChanged: if (fenster.zoomModus !== "") groesseGeaendert.restart()
            onHeightChanged: if (fenster.zoomModus === "seite") groesseGeaendert.restart()
            clip: true
            focus: true
            boundsBehavior: Flickable.StopAtBounds
            contentWidth: Math.max(width, fenster.nach(Dokument.layoutVersion, Dokument.maxBreite()) * fenster.pxProPt + 40)
            contentHeight: fenster.nach(Dokument.layoutVersion, Dokument.gesamtHoehe()) * fenster.pxProPt
            ScrollBar.vertical: ScrollBar { minimumSize: 0.04 }
            ScrollBar.horizontal: ScrollBar {}

            // sichtbarer Seitenbereich (nur diese Seiten existieren als Elemente)
            readonly property int erste: fenster.nach(Dokument.layoutVersion, Dokument.seitenzahl > 0 ? Dokument.seiteBei(contentY / fenster.pxProPt) : 0)
            readonly property int letzte: fenster.nach(Dokument.layoutVersion, Dokument.seitenzahl > 0
                ? Dokument.letzteSeiteBei((contentY + height) / fenster.pxProPt) : -1)

            Rectangle {
                width: ansicht.contentWidth
                height: Math.max(ansicht.contentHeight, ansicht.height)
                color: "#6b6b6b"
            }

            Repeater {
                model: Math.max(0, ansicht.letzte - ansicht.erste + 1)
                delegate: Item {
                    id: blatt
                    required property int index
                    readonly property int nr: ansicht.erste + index
                    x: fenster.nach(Dokument.layoutVersion, fenster.seitenX(nr))
                    y: fenster.nach(Dokument.layoutVersion, Dokument.seitenOben(nr)) * fenster.pxProPt
                    width: fenster.nach(Dokument.layoutVersion, Dokument.seitenBreite(nr)) * fenster.pxProPt
                    height: fenster.nach(Dokument.layoutVersion, Dokument.seitenHoehe(nr)) * fenster.pxProPt

                    onNrChanged: Dokument.linksAnfordern(nr)
                    Component.onCompleted: Dokument.linksAnfordern(nr)

                    Rectangle {
                        anchors.fill: parent
                        anchors.margins: -1
                        color: "transparent"
                        border.color: "#40000000"
                    }

                    Seite {
                        id: seiteBild
                        anchors.fill: parent
                        seite: blatt.nr
                        dichte: Screen.devicePixelRatio
                        onSeiteChanged: neuZeichnen()
                    }

                    Connections {
                        target: Dokument
                        function onTrefferAnzahlChanged() { seiteBild.neuZeichnen() }
                        function onAktuellerTrefferChanged() { seiteBild.neuZeichnen() }
                        function onGenerationChanged() { seiteBild.neuZeichnen() }
                        function onLayoutVersionChanged() { seiteBild.neuZeichnen() }
                        function onAnmerkungenVersionChanged() { seiteBild.neuZeichnen() }
                    }

                    MouseArea {
                        id: maus
                        anchors.fill: parent
                        hoverEnabled: true
                        enabled: fenster.werkzeug === "hand"
                        acceptedButtons: Qt.LeftButton | Qt.RightButton
                        property var link: null
                        property var anm: null
                        cursorShape: link ? Qt.PointingHandCursor : Qt.ArrowCursor
                        function linkUnter(mx, my) {
                            var t = Dokument.linkBei(blatt.nr, mx / width, my / height)
                            return t ? JSON.parse(t) : null
                        }
                        function anmerkungUnter(mx, my) {
                            var t = Dokument.anmerkungBei(blatt.nr, mx / width, my / height)
                            return t ? JSON.parse(t) : null
                        }
                        onPositionChanged: (m) => {
                            link = linkUnter(m.x, m.y)
                            anm = link ? null : anmerkungUnter(m.x, m.y)
                        }
                        onExited: { link = null; anm = null }
                        onClicked: (m) => {
                            if (m.button === Qt.RightButton) { fenster.kontextSeite = blatt.nr; kontextMenue.popup(); return }
                            var l = linkUnter(m.x, m.y)
                            if (l) fenster.linkFolgen(l)
                        }
                        onDoubleClicked: (m) => {
                            var a = anmerkungUnter(m.x, m.y)
                            if (a) fenster.anmerkungBearbeiten(a)
                        }
                        ToolTip.visible: (link !== null && link.typ === "extern") || (anm !== null && anm.inhalt !== "")
                        ToolTip.text: link && link.uri ? link.uri
                                    : (anm ? anm.typ + (anm.autor ? " von " + anm.autor : "") + ":\n" + anm.inhalt + "\n(Doppelklick zum Bearbeiten)" : "")
                    }

                    // ---------------------------------------------------- Werkzeuge
                    Item {
                        id: ebene
                        anchors.fill: parent
                        visible: fenster.werkzeug !== "hand"
                        property var strich: []        // Freihand-Punkte (relativ)
                        property var auswahl: []       // Vorschau-Rechtecke der Textauswahl (relativ)
                        property point start
                        property point ende
                        property bool zieht: false
                        property var loeschKandidat: null
                        readonly property bool rahmenWerkzeug: ["rechteck", "ellipse", "textfeld", "bild"].indexOf(fenster.werkzeug) >= 0
                        readonly property bool linienWerkzeug: fenster.werkzeug === "linie" || fenster.werkzeug === "pfeil"

                        // Endpunkt beim Ziehen; mit Shift: Linien in 45°-Schritten, Rechteck/Ellipse quadratisch
                        function endpunkt(m) {
                            var dx = m.x - start.x, dy = m.y - start.y
                            if (!(m.modifiers & Qt.ShiftModifier)) return Qt.point(m.x, m.y)
                            if (linienWerkzeug) {
                                var laenge = Math.sqrt(dx * dx + dy * dy)
                                var winkel = Math.round(Math.atan2(dy, dx) / (Math.PI / 4)) * (Math.PI / 4)
                                return Qt.point(start.x + laenge * Math.cos(winkel), start.y + laenge * Math.sin(winkel))
                            }
                            if (fenster.werkzeug === "rechteck" || fenster.werkzeug === "ellipse") {
                                var d = Math.max(Math.abs(dx), Math.abs(dy))
                                return Qt.point(start.x + (dx < 0 ? -d : d), start.y + (dy < 0 ? -d : d))
                            }
                            return Qt.point(m.x, m.y)
                        }
                        readonly property bool textWerkzeug: ["hervorheben", "unterstreichen", "durchstreichen"].indexOf(fenster.werkzeug) >= 0

                        function rel(p) { return { x: p.x / width, y: p.y / height } }

                        function auftrag(extra) {
                            var a = { art: fenster.werkzeug, seite: blatt.nr, farbe: fenster.aktuelleFarbe(),
                                      breite: fenster.stiftBreite, groesse: fenster.schriftGroesse }
                            for (var k in extra) a[k] = extra[k]
                            return a
                        }

                        function rahmenAuftrag() {
                            var a = rel(start), b = rel(ende)
                            // Klick statt Ziehen: Standardgröße
                            if (Math.abs(ende.x - start.x) < 8 && Math.abs(ende.y - start.y) < 8)
                                b = { x: Math.min(1, a.x + 0.3), y: Math.min(1, a.y + (fenster.werkzeug === "bild" ? 0.2 : 0.06)) }
                            return auftrag({ a: a, b: b })
                        }

                        Canvas {
                            id: leinwand
                            anchors.fill: parent
                            onPaint: {
                                var ctx = getContext("2d")
                                ctx.reset()
                                ctx.strokeStyle = fenster.aktuelleFarbe()
                                ctx.lineWidth = Math.max(1, fenster.stiftBreite * fenster.pxProPt)
                                ctx.lineCap = "round"
                                ctx.lineJoin = "round"
                                if (fenster.werkzeug === "freihand" && ebene.strich.length > 1) {
                                    ctx.beginPath()
                                    ctx.moveTo(ebene.strich[0].x * width, ebene.strich[0].y * height)
                                    for (var i = 1; i < ebene.strich.length; i++)
                                        ctx.lineTo(ebene.strich[i].x * width, ebene.strich[i].y * height)
                                    ctx.stroke()
                                } else if (ebene.zieht && ebene.linienWerkzeug) {
                                    var x0 = ebene.start.x, y0 = ebene.start.y, x1 = ebene.ende.x, y1 = ebene.ende.y
                                    var l = Math.sqrt((x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0))
                                    if (l < 1) return
                                    var ux = (x1 - x0) / l, uy = (y1 - y0) / l
                                    // Pfeilspitze wie im PDF: 3,5 × Strichstärke, mindestens 7 pt, höchstens 60 % der Linie
                                    var sp = Math.min(Math.max(3.5 * fenster.stiftBreite, 7) * fenster.pxProPt, l * 0.6)
                                    var hb = sp * 0.45
                                    var pfeil = fenster.werkzeug === "pfeil"
                                    ctx.lineCap = "butt"
                                    ctx.beginPath()
                                    ctx.moveTo(x0, y0)
                                    if (pfeil) ctx.lineTo(x1 - ux * sp * 0.8, y1 - uy * sp * 0.8); else ctx.lineTo(x1, y1)
                                    ctx.stroke()
                                    if (pfeil) {
                                        var bx = x1 - ux * sp, by = y1 - uy * sp
                                        ctx.fillStyle = fenster.aktuelleFarbe()
                                        ctx.beginPath()
                                        ctx.moveTo(x1, y1)
                                        ctx.lineTo(bx - uy * hb, by + ux * hb)
                                        ctx.lineTo(bx + uy * hb, by - ux * hb)
                                        ctx.closePath()
                                        ctx.fill()
                                    }
                                } else if (ebene.zieht && ebene.rahmenWerkzeug) {
                                    var x = Math.min(ebene.start.x, ebene.ende.x), y = Math.min(ebene.start.y, ebene.ende.y)
                                    var w = Math.abs(ebene.ende.x - ebene.start.x), h = Math.abs(ebene.ende.y - ebene.start.y)
                                    if (fenster.werkzeug === "ellipse") {
                                        ctx.beginPath()
                                        ctx.ellipse(x, y, w, h)
                                        ctx.stroke()
                                    } else {
                                        if (fenster.werkzeug !== "rechteck") { ctx.setLineDash([6, 4]); ctx.lineWidth = 1.5 }
                                        ctx.strokeRect(x, y, w, h)
                                    }
                                }
                            }
                        }

                        Repeater {
                            model: ebene.auswahl
                            delegate: Rectangle {
                                required property var modelData
                                x: modelData.x * ebene.width
                                y: modelData.y * ebene.height
                                width: modelData.b * ebene.width
                                height: modelData.h * ebene.height
                                color: "#403399ff"
                                border.color: "#803399ff"
                            }
                        }

                        Rectangle {
                            visible: ebene.loeschKandidat !== null
                            x: ebene.loeschKandidat ? ebene.loeschKandidat.rect.x * ebene.width - 3 : 0
                            y: ebene.loeschKandidat ? ebene.loeschKandidat.rect.y * ebene.height - 3 : 0
                            width: ebene.loeschKandidat ? ebene.loeschKandidat.rect.b * ebene.width + 6 : 0
                            height: ebene.loeschKandidat ? ebene.loeschKandidat.rect.h * ebene.height + 6 : 0
                            color: "#20ff0000"
                            border.color: "#e00000"
                            border.width: 2
                        }

                        MouseArea {
                            anchors.fill: parent
                            hoverEnabled: true
                            preventStealing: true // Ziehen zeichnet statt zu scrollen
                            acceptedButtons: Qt.LeftButton | Qt.RightButton
                            cursorShape: fenster.werkzeug === "radierer" ? (ebene.loeschKandidat ? Qt.PointingHandCursor : Qt.ForbiddenCursor)
                                       : ebene.textWerkzeug ? Qt.IBeamCursor : Qt.CrossCursor
                            onPressed: (m) => {
                                if (m.button === Qt.RightButton) return
                                ebene.start = Qt.point(m.x, m.y)
                                ebene.ende = ebene.start
                                ebene.zieht = true
                                if (fenster.werkzeug === "freihand") ebene.strich = [ebene.rel(ebene.start)]
                                leinwand.requestPaint()
                            }
                            onPositionChanged: (m) => {
                                if (fenster.werkzeug === "radierer") {
                                    var t = Dokument.anmerkungBei(blatt.nr, m.x / width, m.y / height)
                                    ebene.loeschKandidat = t ? JSON.parse(t) : null
                                    return
                                }
                                if (!ebene.zieht) return
                                var p = ebene.endpunkt(m)
                                if (fenster.werkzeug === "freihand") {
                                    var letzter = ebene.strich[ebene.strich.length - 1]
                                    if (Math.abs(letzter.x * width - p.x) + Math.abs(letzter.y * height - p.y) >= 2) {
                                        var neu = ebene.strich.slice()
                                        neu.push(ebene.rel(p))
                                        ebene.strich = neu
                                        leinwand.requestPaint()
                                    }
                                } else if (ebene.textWerkzeug) {
                                    if (Math.abs(p.x - ebene.ende.x) + Math.abs(p.y - ebene.ende.y) < 3) return
                                    ebene.ende = p
                                    var a = ebene.rel(ebene.start), b = ebene.rel(p)
                                    var j = Dokument.textAuswahl(blatt.nr, a.x, a.y, b.x, b.y)
                                    ebene.auswahl = j ? JSON.parse(j).rects : []
                                } else {
                                    ebene.ende = p
                                    leinwand.requestPaint()
                                }
                            }
                            onReleased: (m) => {
                                if (m.button === Qt.RightButton) return
                                ebene.ende = ebene.endpunkt(m)
                                ebene.zieht = false
                                var w = fenster.werkzeug
                                if (w === "freihand") {
                                    if (ebene.strich.length > 1)
                                        fenster.meldung(Dokument.anlegen(JSON.stringify(ebene.auftrag({ striche: [ebene.strich] }))))
                                    ebene.strich = []
                                } else if (ebene.textWerkzeug) {
                                    var start = ebene.rel(ebene.start), ende = ebene.rel(ebene.ende)
                                    if (ebene.auswahl.length > 0 || Math.abs(ebene.ende.x - ebene.start.x) > 2)
                                        fenster.meldung(Dokument.anlegen(JSON.stringify(ebene.auftrag({ von: start, bis: ende }))))
                                    ebene.auswahl = []
                                } else if (ebene.linienWerkzeug) {
                                    var dl = Math.abs(ebene.ende.x - ebene.start.x) + Math.abs(ebene.ende.y - ebene.start.y)
                                    if (dl > 4)
                                        fenster.meldung(Dokument.anlegen(JSON.stringify(ebene.auftrag({ a: ebene.rel(ebene.start), b: ebene.rel(ebene.ende) }))))
                                } else if (w === "rechteck" || w === "ellipse") {
                                    if (Math.abs(ebene.ende.x - ebene.start.x) > 4 && Math.abs(ebene.ende.y - ebene.start.y) > 4)
                                        fenster.meldung(Dokument.anlegen(JSON.stringify(ebene.rahmenAuftrag())))
                                } else if (w === "textfeld") {
                                    var feld = ebene.rahmenAuftrag()
                                    fenster.texteingabe("Textfeld", "", true, function(t) {
                                        if (t.trim() === "") return
                                        feld.text = t
                                        fenster.meldung(Dokument.anlegen(JSON.stringify(feld)))
                                    })
                                } else if (w === "bild") {
                                    fenster.bildRahmen = ebene.rahmenAuftrag()
                                    bildDialog.open()
                                } else if (w === "notiz") {
                                    var notiz = ebene.auftrag({ a: ebene.rel(ebene.ende) })
                                    fenster.texteingabe("Notiz", "", true, function(t) {
                                        notiz.text = t
                                        fenster.meldung(Dokument.anlegen(JSON.stringify(notiz)))
                                    })
                                }
                                leinwand.requestPaint()
                            }
                            onClicked: (m) => {
                                if (m.button === Qt.RightButton) { fenster.kontextSeite = blatt.nr; kontextMenue.popup(); return }
                                if (fenster.werkzeug === "radierer" && ebene.loeschKandidat) {
                                    fenster.meldung(Dokument.loeschen(ebene.loeschKandidat.seite, ebene.loeschKandidat.index))
                                    ebene.loeschKandidat = null
                                }
                            }
                            onExited: ebene.loeschKandidat = null
                        }
                    }

                    Rectangle {
                        visible: maus.link !== null
                        x: maus.link ? maus.link.rect.x * blatt.width : 0
                        y: maus.link ? maus.link.rect.y * blatt.height : 0
                        width: maus.link ? maus.link.rect.b * blatt.width : 0
                        height: maus.link ? maus.link.rect.h * blatt.height : 0
                        color: "#22d00000"
                        border.color: "#d00000"
                        radius: 2
                    }
                }
            }

            // Sprungziel eines Links: blinkt rot und bleibt markiert, bis zum nächsten Sprung oder Esc
            Rectangle {
                id: markierung
                property int seite: -1
                property var zielY: null // relative Höhe auf der ungedrehten Seite oder null
                readonly property real anzeigeY: fenster.nach(Dokument.layoutVersion, seite >= 0 && zielY !== null
                    ? Dokument.anzeigeY(seite, zielY) : -1)
                property var zielRect: null // relatives Rechteck (Anzeige) einer Anmerkung
                function setzen(s, y) { zielRect = null; seite = s; zielY = (y === undefined) ? null : y; blinkAnimation.restart() }
                function setzenRechteck(s, r) { zielY = null; zielRect = r; seite = s; blinkAnimation.restart() }
                function entfernen() { blinkAnimation.stop(); seite = -1; zielRect = null }
                visible: seite >= 0
                readonly property real seitenHoehePx: fenster.nach(Dokument.layoutVersion, seite >= 0 ? Dokument.seitenHoehe(seite) : 0) * fenster.pxProPt
                readonly property real seitenObenPx: fenster.nach(Dokument.layoutVersion, seite >= 0 ? Dokument.seitenOben(seite) : 0) * fenster.pxProPt
                readonly property real seitenBreitePx: fenster.nach(Dokument.layoutVersion, seite >= 0 ? Dokument.seitenBreite(seite) : 0) * fenster.pxProPt
                readonly property real seitenXPx: fenster.nach(Dokument.layoutVersion, seite >= 0 ? fenster.seitenX(seite) : 0)
                x: zielRect ? seitenXPx + zielRect.x * seitenBreitePx - 5 : seitenXPx - 4
                width: zielRect ? zielRect.b * seitenBreitePx + 10 : seitenBreitePx + 8
                // ohne Zielhöhe (oder bei 90/270 Grad) wird die ganze Seite umrahmt
                y: zielRect ? seitenObenPx + zielRect.y * seitenHoehePx - 5
                            : (anzeigeY >= 0 ? seitenObenPx + anzeigeY * seitenHoehePx - 6 : seitenObenPx - 4)
                height: zielRect ? zielRect.h * seitenHoehePx + 10
                                 : (anzeigeY >= 0 ? Math.max(28, 30 * fenster.pxProPt) : seitenHoehePx + 8)
                color: (zielRect || anzeigeY >= 0) ? "#22ff0000" : "transparent"
                border.color: "#e00000"
                border.width: 3
                radius: 3
                SequentialAnimation {
                    id: blinkAnimation
                    loops: 4
                    NumberAnimation { target: markierung; property: "opacity"; from: 1; to: 0.15; duration: 220 }
                    NumberAnimation { target: markierung; property: "opacity"; from: 0.15; to: 1; duration: 220 }
                }
            }

            WheelHandler {
                acceptedModifiers: Qt.ControlModifier
                onWheel: (e) => { fenster.zoomModus = ""; fenster.setzeZoom(fenster.zoom * (e.angleDelta.y > 0 ? 1.1 : 1 / 1.1)) }
            }

            // Startseite: Hinweise sind anklickbar
            Label {
                visible: Dokument.seitenzahl === 0
                // mittig im sichtbaren Bereich (ohne Dokument hat die Fläche selbst die Höhe 0)
                x: Math.max(16, (ansicht.width - width) / 2)
                y: Math.max(16, (ansicht.height - height) / 2)
                color: "#e0e0e0"
                linkColor: "#ffffff"
                horizontalAlignment: Text.AlignHCenter
                textFormat: Text.StyledText
                readonly property var verweise: [["datei_oeffnen", "PDF öffnen"], ["ordner_suchen", "In einem Ordner suchen"],
                                                 ["hilfe", "Hilfe"], ["tastenbelegung", "Tastenbelegung"]]
                text: (Dokument.fehler ? Dokument.fehler.replace(/&/g, "&amp;").replace(/</g, "&lt;") + "<br><br>"
                                       : "<b>Abolos PDF</b><br><br>Keine Datei geöffnet.<br><br>")
                      + verweise.map(function(v) {
                            var t = fenster.erstesTaste(v[0])
                            return "<a href=\"" + v[0] + "\">" + v[1] + "</a>" + (t ? " (" + Tasten.anzeigename(t) + ")" : "")
                        }).join("  ·  ")
                onLinkActivated: (link) => fenster.aktion(link)
                HoverHandler { cursorShape: parent.hoveredLink ? Qt.PointingHandCursor : Qt.ArrowCursor }
            }
        }

        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: statusText.implicitHeight + 8
            color: palette.window
            Label {
                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: 8
                // nur sichtbar, wenn eine Schutzmaßnahme ausnahmsweise nicht greift (Normalfall: nichts anzeigen)
                readonly property string netz: Dokument.netzsperre()
                readonly property string ausfuehren: Dokument.ausfuehrsperre()
                visible: netz !== "" || ausfuehren !== ""
                text: [netz !== "" ? "Netzsperre aus" : "", ausfuehren !== "" ? "Ausführsperre aus" : ""].filter(function(x) { return x }).join(" · ")
                color: "#b00000"
                ToolTip.visible: sperreMaus.containsMouse
                ToolTip.text: (netz !== "" ? "Die Netzsperre konnte nicht eingerichtet werden (" + netz + "). "
                                             + "Abolos PDF selbst hat trotzdem keine Netzwerkfunktionen.\n" : "")
                              + (ausfuehren !== "" ? "Die Ausführsperre konnte nicht eingerichtet werden (" + ausfuehren + "). "
                                                     + "Sie braucht Linux ab 5.13 mit Landlock." : "")
                MouseArea { id: sperreMaus; anchors.fill: parent; hoverEnabled: true; onClicked: fenster.aktion("ueber") }
            }
            Label {
                id: statusText
                anchors.verticalCenter: parent.verticalCenter
                x: 8
                text: {
                    if (fenster.meldungText) return fenster.meldungText
                    if (Dokument.fehler && Dokument.seitenzahl > 0) return Dokument.fehler
                    if (Dokument.seitenzahl === 0) return ""
                    var s = "Seite " + (fenster.aktuelleSeite + 1) + " von " + Dokument.seitenzahl
                    if (Dokument.geaendert) s += "   ·   ungespeicherte Änderungen"
                    if (Dokument.drehung !== 0) s += "   ·   gedreht " + Dokument.drehung + "°"
                    if (Dokument.sucheLaeuft)
                        s += "   ·   Suche läuft: " + Math.round(Dokument.sucheFortschritt * 100) + " %, " + Dokument.trefferAnzahl + " Treffer"
                    else if (Dokument.trefferAnzahl > 0)
                        s += "   ·   " + Dokument.trefferAnzahl + " Treffer"
                    return s
                }
            }
        }
    }

    Menu {
        id: kontextMenue
        onClosed: Qt.callLater(function() { fenster.kontextSeite = -1 })
        MenuItem { text: fenster.tipp("Zurück", "zurueck"); enabled: fenster.verlaufPos > 0; onTriggered: fenster.aktion("zurueck") }
        MenuItem { text: fenster.tipp("Vorige Seite", "vorige_seite"); onTriggered: fenster.aktion("vorige_seite") }
        MenuItem { text: fenster.tipp("Nächste Seite", "naechste_seite"); onTriggered: fenster.aktion("naechste_seite") }
        MenuSeparator {}
        MenuItem { text: fenster.tipp("Vergrößern", "zoom_rein"); onTriggered: fenster.aktion("zoom_rein") }
        MenuItem { text: fenster.tipp("Verkleinern", "zoom_raus"); onTriggered: fenster.aktion("zoom_raus") }
        MenuItem { text: fenster.tipp("Seitenbreite", "seitenbreite"); onTriggered: fenster.aktion("seitenbreite") }
        MenuItem { text: fenster.tipp("Ganze Seite", "ganze_seite"); onTriggered: fenster.aktion("ganze_seite") }
        MenuSeparator {}
        MenuItem { text: fenster.tipp("Diese Seite im Uhrzeigersinn drehen", "seite_drehen_rechts"); onTriggered: fenster.aktion("seite_drehen_rechts") }
        MenuItem { text: fenster.tipp("Diese Seite gegen den Uhrzeigersinn drehen", "seite_drehen_links"); onTriggered: fenster.aktion("seite_drehen_links") }
        MenuItem { text: fenster.tipp("Alle Seiten im Uhrzeigersinn drehen", "drehen_rechts"); onTriggered: fenster.aktion("drehen_rechts") }
        MenuItem { text: fenster.tipp("Alle Seiten gegen den Uhrzeigersinn drehen", "drehen_links"); onTriggered: fenster.aktion("drehen_links") }
        MenuSeparator {}
        MenuItem { text: fenster.tipp("Einzelseiten", "einzelseite"); checkable: true; checked: Dokument.modus === 0; onTriggered: fenster.setzeModus(0) }
        MenuItem { text: fenster.tipp("Doppelseiten", "doppelseite"); checkable: true; checked: Dokument.modus === 1; onTriggered: fenster.setzeModus(1) }
        MenuItem { text: fenster.tipp("Buchansicht (Deckblatt einzeln)", "deckblatt"); checkable: true; checked: Dokument.modus === 2; onTriggered: fenster.setzeModus(2) }
        MenuSeparator {}
        MenuItem { text: fenster.tipp("Suchen", "suchen"); onTriggered: fenster.aktion("suchen") }
        MenuItem { text: fenster.tipp("Tastenbelegung", "tastenbelegung"); onTriggered: fenster.aktion("tastenbelegung") }
        MenuSeparator {}
        MenuItem { text: fenster.tipp("Hilfe", "hilfe"); onTriggered: fenster.aktion("hilfe") }
        MenuItem { text: "Über Abolos PDF"; onTriggered: fenster.aktion("ueber") }
    }

    // ---------------------------------------------------------------- Testskript
    // ABOLOS_SKRIPT="warte:800;suche:Text;aktion:naechster_treffer;screenshot:/tmp/a.png;beenden"

    property var skript: []

    Timer {
        id: skriptTimer
        onTriggered: fenster.skriptSchritt()
    }

    function skriptSchritt() {
        if (skript.length === 0) return
        var befehl = skript.shift()
        var i = befehl.indexOf(":")
        var art = i < 0 ? befehl : befehl.slice(0, i)
        var arg = i < 0 ? "" : befehl.slice(i + 1)
        var pause = 60
        switch (art) {
        case "warte": pause = parseInt(arg); break
        case "aktion": aktion(arg); break
        case "suche": sucheFeld.text = arg; sucheStarten(); break
        case "zoom": setzeZoom(parseFloat(arg)); break
        case "modus": setzeModus(parseInt(arg)); break
        case "tippen": {
            // Tippen simulieren: jedes Zeichen als eigene Eingabe mit Pause
            if (sucheAnker === null) sucheAnker = hierPosition()
            sucheFeld.text = arg
            sucheStarten()
            break
        }
        case "seite": springeZu(parseInt(arg) - 1, null, false); break
        case "links": console.log("Links Seite " + arg + ": " + Dokument.linksJson(parseInt(arg) - 1)); break
        case "link": {
            var a = arg.split(",")
            var j = Dokument.linkBei(parseInt(a[0]) - 1, parseFloat(a[1]), parseFloat(a[2]))
            if (j) linkFolgen(JSON.parse(j)); else console.log("Kein Link bei " + arg)
            break
        }
        case "markierung": console.log("Markierung: seite " + markierung.seite + ", zielY " + markierung.zielY
                                        + ", anzeigeY " + markierung.anzeigeY + ", Dokument.anzeigeY "
                                        + (markierung.seite >= 0 ? Dokument.anzeigeY(markierung.seite, markierung.zielY) : "-")
                                        + ", Drehung Seite " + (markierung.seite >= 0 ? Dokument.seitenDrehung(markierung.seite) : "-")
                                        + ", y " + markierung.y + ", h " + markierung.height); break
        case "anlegen": { var f = Dokument.anlegen(arg); console.log("anlegen " + (f ? "FEHLER " + f : "ok")); break }
        case "auswahl": { var w = arg.split(","); console.log("Auswahl: " + Dokument.textAuswahl(parseInt(w[0]) - 1, parseFloat(w[1]), parseFloat(w[2]), parseFloat(w[3]), parseFloat(w[4])).slice(0, 300)); break }
        case "anmerkungen": console.log("Anmerkungen: " + Dokument.anmerkungenJson()); break
        case "gliederung": console.log("Gliederung: " + Dokument.gliederungJson().slice(0, 400)); break
        case "rueckgaengig": console.log("Rückgängig: " + (Dokument.rueckgaengig() || "ok")); break
        case "speichern": console.log("Speichern: " + (Dokument.speichern(arg) || "ok") + ", geändert " + Dokument.geaendert); break
        case "oeffnen": oeffneDatei(arg); break
        case "leiste": leisteSichtbar = true; seitenleiste.reiter(parseInt(arg)); break
        case "ordnersuche": {
            // ordnersuche:pfad|text  oder  ordnersuche:pfad|text|w  (nur ganze Wörter)
            var o = arg.split("|")
            leisteSichtbar = true
            seitenleiste.zeigeSuche(true)
            seitenleiste.ordnerNurGanzeWoerter = o[2] === "w"
            seitenleiste.ordnerSuchen(o[0], o[1])
            break
        }
        case "ordnerstatus": console.log("Ordnersuche: " + Dokument.ordnerStatus + ", läuft " + Dokument.ordnerLaeuft + ", Treffer " + Dokument.ordnerAnzahl
                                         + ", erste: " + Dokument.ordnerTreffer(0, 3)); break
        case "ordneroeffnen": seitenleiste.ordnerTrefferKlick(parseInt(arg)); break
        case "ordnerdialog": seitenleiste.ordnerWaehlen(); break
        case "hilfe": hilfeFenster.zeigen(arg); break
        case "hilfebild":
            hilfeFenster.bildSpeichern(arg, function(ok) {
                console.log("Hilfebild " + (ok ? "gespeichert: " : "FEHLER: ") + arg)
                skriptTimer.interval = 60
                skriptTimer.start()
            })
            return
        case "trefferliste": console.log("Trefferliste: " + Dokument.trefferAnzahl + " Treffer, erster: " + Dokument.trefferEintrag(0)
                                         + ", letzter: " + Dokument.trefferEintrag(Dokument.trefferAnzahl - 1)); break
        case "treffer": trefferWaehlen(parseInt(arg)); break
        case "status": console.log("Status: Seite " + (aktuelleSeite + 1) + "/" + Dokument.seitenzahl + ", Zoom " + zoom.toFixed(2)
                                    + ", Treffer " + Dokument.trefferAnzahl + ", aktuell " + Dokument.aktuellerTreffer
                                    + ", Suche läuft " + Dokument.sucheLaeuft + ", Verlauf " + verlaufPos + "/" + verlauf.length); break
        case "screenshot":
            wurzel.grabToImage(function(r) {
                console.log("Screenshot " + (r.saveToFile(arg) ? "gespeichert: " : "FEHLER: ") + arg)
                skriptTimer.interval = 60
                skriptTimer.start()
            })
            return
        case "beenden": schliessenErlaubt = true; Qt.quit(); return
        default: console.log("Unbekannter Skriptbefehl: " + befehl)
        }
        skriptTimer.interval = pause
        skriptTimer.start()
    }

    Component.onCompleted: {
        var start = Dokument.startPfad()
        if (start) oeffneDatei(start)
        var s = Dokument.umgebung("ABOLOS_SKRIPT")
        if (s) {
            skript = s.split(";")
            skriptTimer.interval = 300
            skriptTimer.start()
        }
    }
}
