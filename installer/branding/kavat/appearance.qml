// The Appearance page of the installer: the palette, the mode and the accent the new
// system starts with. What is chosen is left in Calamares' global storage, where the
// kavat-look step takes it from; the picture on the right is drawn with the same colours.
import io.calamares.core 1.0
import io.calamares.ui 1.0

import QtQuick 2.15
import QtQuick.Layouts 1.15

import "palettes.js" as Data

Item {
    id: page

    // the installer's own colours: Kavat's dark side
    readonly property color bg: "#151821"
    readonly property color raised: "#1e2230"
    readonly property color line: "#2a2f3f"
    readonly property color ink: "#e8e6df"
    readonly property color soft: "#8b90a0"
    readonly property color mark: "#f2a93b"

    property int palette: 0
    property string mode: "dark"
    property int accent: 0

    readonly property var chosen: Data.palettes[palette]
    readonly property var side: chosen[mode]
    readonly property color tint: chosen.accents[accent].colour

    function keep() {
        Global.insert("kavatPalette", chosen.id)
        Global.insert("kavatMode", mode)
        Global.insert("kavatAccent", chosen.accents[accent].name)
    }
    Component.onCompleted: keep()
    onPaletteChanged: { accent = 0; keep() }
    onModeChanged: keep()
    onAccentChanged: keep()

    Rectangle { anchors.fill: parent; color: page.bg }

    component Heading: Text {
        color: page.soft
        font.pixelSize: 11
        font.letterSpacing: 1.2
        font.capitalization: Font.AllUppercase
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 28
        spacing: 6

        Text { text: "Appearance"; color: page.ink; font.pixelSize: 24; font.weight: Font.Bold }
        Text {
            text: "Your choice is applied to the login screen, the boot splash and the desktop from the very first start."
            color: page.soft; font.pixelSize: 13
            Layout.bottomMargin: 14
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 28

            ColumnLayout {
                Layout.alignment: Qt.AlignTop
                Layout.fillWidth: false
                Layout.minimumWidth: 452
                Layout.maximumWidth: 452
                spacing: 12

                Heading { text: "Palette" }
                GridLayout {
                    columns: 4
                    columnSpacing: 10
                    rowSpacing: 10
                    Repeater {
                        model: Data.palettes
                        Rectangle {
                            required property int index
                            required property var modelData
                            readonly property var shown: modelData[page.mode]
                            width: 105; height: 84; radius: 10
                            color: index === page.palette ? page.raised : "transparent"
                            border.width: index === page.palette ? 2 : 1
                            border.color: index === page.palette ? page.mark : page.line
                            Row {
                                x: 10; y: 10; spacing: 6
                                Rectangle { width: 48; height: 40; radius: 6; color: shown.surface; border.width: 1; border.color: modelData.accents[0].colour }
                                Column {
                                    spacing: 4
                                    Rectangle { width: 28; height: 18; radius: 4; color: shown.surface2 }
                                    Rectangle { width: 28; height: 18; radius: 4; color: modelData.accents[0].colour }
                                }
                            }
                            Text { x: 10; y: 60; text: modelData.name; color: page.ink; font.pixelSize: 12; font.weight: Font.Medium }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.palette = index }
                        }
                    }
                }

                Heading { text: "Mode"; Layout.topMargin: 8 }
                Rectangle {
                    Layout.fillWidth: true
                    height: 36; radius: 8; color: page.raised
                    Row {
                        anchors.fill: parent; anchors.margins: 3
                        Repeater {
                            model: [ { name: "Light", value: "light" }, { name: "Dark", value: "dark" } ]
                            Rectangle {
                                required property var modelData
                                width: parent.width / 2; height: parent.height; radius: 6
                                color: page.mode === modelData.value ? page.bg : "transparent"
                                Text { anchors.centerIn: parent; text: modelData.name; color: page.mode === modelData.value ? page.ink : page.soft; font.pixelSize: 13 }
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.mode = modelData.value }
                            }
                        }
                    }
                }

                Heading { text: "Accent"; Layout.topMargin: 8 }
                Row {
                    spacing: 10
                    Repeater {
                        model: page.chosen.accents
                        Rectangle {
                            required property int index
                            required property var modelData
                            width: 34; height: 34; radius: 17
                            color: modelData.colour
                            border.width: 3
                            border.color: index === page.accent ? page.ink : modelData.colour
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.accent = index }
                        }
                    }
                }
                Text {
                    text: "Change it any time with Super+T."
                    color: page.soft; font.pixelSize: 12
                    Layout.topMargin: 10
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 12

                Heading { text: "Preview" }
                // a desktop in small: the bar, three windows and the one in use with its border
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: 14
                    color: page.side.bg
                    border.width: 1; border.color: page.line
                    clip: true
                    Rectangle {
                        width: parent.width * 0.9; height: parent.width * 0.9; radius: width / 2
                        x: parent.width * 0.45; y: -height * 0.55
                        color: page.tint; opacity: 0.16
                    }
                    Rectangle {
                        id: bar
                        x: 18; y: 18; width: parent.width - 36; height: 30; radius: 9
                        color: page.side.surface
                        Rectangle { x: 10; anchors.verticalCenter: parent.verticalCenter; width: 34; height: 12; radius: 6; color: page.tint }
                        Text { anchors.centerIn: parent; text: "14:01"; color: page.side.text; font.pixelSize: 12; font.weight: Font.Medium }
                        Rectangle { anchors.right: parent.right; anchors.rightMargin: 10; anchors.verticalCenter: parent.verticalCenter; width: 60; height: 8; radius: 4; color: page.side.muted; opacity: 0.6 }
                    }
                    Rectangle {
                        x: 18; y: 62; width: (parent.width - 48) * 0.58; height: parent.height - 80; radius: 10
                        color: page.side.surface
                        border.width: 2; border.color: page.tint
                        Column {
                            x: 16; y: 16; spacing: 9
                            Repeater {
                                model: [ 0.5, 0.8, 0.65, 0.4, 0.75, 0.55 ]
                                Rectangle { required property real modelData; width: 220 * modelData; height: 8; radius: 4; color: index === 0 ? page.tint : page.side.muted; opacity: index === 0 ? 1 : 0.55; required property int index }
                            }
                        }
                    }
                    Rectangle {
                        x: 30 + (parent.width - 48) * 0.58; y: 62; width: (parent.width - 48) * 0.42; height: (parent.height - 92) / 2; radius: 10
                        color: page.side.surface
                        border.width: 1; border.color: page.side.border
                        Text { x: 14; y: 12; text: "Aa"; color: page.side.text; font.pixelSize: 22; font.weight: Font.Bold }
                        Text { x: 14; y: 44; text: page.chosen.name + " · " + page.mode; color: page.side.muted; font.pixelSize: 12 }
                    }
                    Rectangle {
                        x: 30 + (parent.width - 48) * 0.58; y: 74 + (parent.height - 92) / 2; width: (parent.width - 48) * 0.42; height: (parent.height - 92) / 2; radius: 10
                        color: page.side.surface2
                        border.width: 1; border.color: page.side.border
                        Rectangle { x: 14; y: 14; width: 84; height: 26; radius: 7; color: page.tint
                            Text { anchors.centerIn: parent; text: "Button"; color: page.mode === "dark" ? "#14110a" : "#ffffff"; font.pixelSize: 12; font.weight: Font.Medium } }
                    }
                }
            }
        }
    }
}
