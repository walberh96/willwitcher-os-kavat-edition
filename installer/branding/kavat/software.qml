// The Software page of the installer: the programs the new system brings from its first
// start, besides the base system. What is checked is left in Calamares' global storage,
// where the kavat-software step takes it from. It needs a network: what cannot be
// downloaded is left out, and the system is installed as it comes.
import io.calamares.core 1.0
import io.calamares.ui 1.0

import QtQuick 2.15
import QtQuick.Layouts 1.15

import "software.js" as Data

Item {
    id: page

    // the installer's own colours: Kavat's dark side
    readonly property color bg: "#151821"
    readonly property color raised: "#1e2230"
    readonly property color line: "#2a2f3f"
    readonly property color ink: "#e8e6df"
    readonly property color soft: "#8b90a0"
    readonly property color mark: "#f2a93b"

    property int category: 0
    // the packages checked, by the name of their package line: { "firefox": true, … }
    property var checked: ({})
    property string extra: ""
    property int count: 0

    function toggle(pkg) {
        // a new object each time: the boxes follow the property, not what is inside it
        var now = Object.assign({}, checked)
        if (now[pkg]) delete now[pkg]; else now[pkg] = true
        checked = now
        keep()
    }
    function countIn(index) {
        var n = 0
        var programs = Data.categories[index].programs
        for (var i = 0; i < programs.length; i++) if (checked[programs[i].pkg]) n++
        return n
    }
    function keep() {
        var names = []
        for (var pkg in checked) names.push(pkg)
        // what is typed is only ever names of packages
        var typed = extra.toLowerCase().split(/[\s,]+/).filter(function (name) { return /^[a-z0-9][a-z0-9@._+-]*$/.test(name) })
        count = names.length + typed.length
        Global.insert("kavatSoftware", names.concat(typed).join(" "))
    }
    Component.onCompleted: keep()

    Rectangle { anchors.fill: parent; color: page.bg }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 28
        spacing: 6

        Text { text: "Software"; color: page.ink; font.pixelSize: 24; font.weight: Font.Bold }
        Text {
            text: "Pick what the system brings from its first start. It is downloaded while installing, so it needs a network: without one, nothing is added and the system is installed as it comes."
            color: page.soft; font.pixelSize: 13; wrapMode: Text.WordWrap
            Layout.fillWidth: true; Layout.bottomMargin: 14
        }

        RowLayout {
            Layout.fillWidth: true; Layout.fillHeight: true
            spacing: 24

            // the categories, each with how many of its programs are checked
            ColumnLayout {
                Layout.preferredWidth: 190; Layout.minimumWidth: 190; Layout.maximumWidth: 190
                Layout.fillHeight: true
                spacing: 2
                Repeater {
                    model: Data.categories
                    Rectangle {
                        required property var modelData
                        required property int index
                        Layout.fillWidth: true; height: 34; radius: 7
                        color: index === page.category ? page.raised : "transparent"
                        Text {
                            anchors.left: parent.left; anchors.leftMargin: 12; anchors.verticalCenter: parent.verticalCenter
                            text: modelData.name; font.pixelSize: 14
                            color: index === page.category ? page.ink : page.soft
                        }
                        Text {
                            // read page.checked so that the number follows the boxes
                            property var follow: page.checked
                            anchors.right: parent.right; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter
                            text: page.countIn(index) > 0 ? page.countIn(index) : ""
                            color: page.mark; font.pixelSize: 13; font.weight: Font.Bold
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.category = index }
                    }
                }
                Item { Layout.fillHeight: true }
            }

            // the programs of the category in view
            ColumnLayout {
                Layout.fillWidth: true; Layout.fillHeight: true
                spacing: 4
                Repeater {
                    model: Data.categories[page.category].programs
                    Rectangle {
                        required property var modelData
                        readonly property bool on: page.checked[modelData.pkg] === true
                        Layout.fillWidth: true; height: 38; radius: 7
                        color: on ? page.raised : "transparent"
                        Rectangle {
                            id: box
                            x: 10; anchors.verticalCenter: parent.verticalCenter
                            width: 16; height: 16; radius: 4
                            color: parent.on ? page.mark : "transparent"
                            border.width: 1.5; border.color: parent.on ? page.mark : page.soft
                            Text { anchors.centerIn: parent; text: "✓"; visible: parent.parent.on; color: "#14110a"; font.pixelSize: 11; font.weight: Font.Bold }
                        }
                        Text {
                            id: label
                            anchors.left: box.right; anchors.leftMargin: 12; anchors.verticalCenter: parent.verticalCenter
                            text: modelData.name; color: page.ink; font.pixelSize: 14
                        }
                        Text {
                            anchors.left: label.right; anchors.leftMargin: 12; anchors.verticalCenter: parent.verticalCenter
                            text: modelData.what; color: page.soft; font.pixelSize: 12
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.toggle(modelData.pkg) }
                    }
                }
                Item { Layout.fillHeight: true }
            }
        }

        // any other package, by its name
        RowLayout {
            Layout.fillWidth: true
            spacing: 12
            Text { text: "Any other package"; color: page.soft; font.pixelSize: 13 }
            Rectangle {
                Layout.fillWidth: true; height: 32; radius: 7
                color: page.raised; border.width: 1; border.color: field.activeFocus ? page.mark : page.line
                TextInput {
                    id: field
                    anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 10
                    verticalAlignment: TextInput.AlignVCenter
                    color: page.ink; selectionColor: page.mark; font.pixelSize: 13; font.family: "JetBrains Mono"
                    clip: true
                    onTextChanged: { page.extra = text; page.keep() }
                }
                Text {
                    anchors.fill: parent; anchors.leftMargin: 10; verticalAlignment: Text.AlignVCenter
                    visible: field.text === ""; color: page.soft; font.pixelSize: 13
                    text: "names from Arch's repositories, with spaces between them"
                }
            }
        }
        Text {
            text: page.count === 0 ? "Nothing added: the base system only."
                                   : page.count + (page.count === 1 ? " line of packages is added." : " lines of packages are added.") + " What is only in the AUR comes later, through kav's reviewed list."
            color: page.soft; font.pixelSize: 12
        }
    }
}
