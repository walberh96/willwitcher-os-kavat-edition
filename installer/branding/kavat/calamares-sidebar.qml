// The installer's steps, down its left side: the ones done with a check, the one in view
// with its number in the accent, the ones to come with theirs; and, at the bottom, which
// step this is. The buttons along the bottom are Calamares' own, so that Alt with a letter
// presses them from anywhere; stylesheet.qss gives them their look.
import io.calamares.ui 1.0
import io.calamares.core 1.0

import QtQuick 2.15
import QtQuick.Layouts 1.15

import "answers.js" as Earlier

Rectangle {
    id: sideBar
    color: "#1b1f2b"
    anchors.fill: parent

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 4

        RowLayout {
            spacing: 12
            Layout.bottomMargin: 22
            Image {
                source: "file:/" + Branding.imagePath(Branding.ProductLogo)
                sourceSize.width: 30; sourceSize.height: 30
                Layout.preferredWidth: 30; Layout.preferredHeight: 30
            }
            Column {
                Text { text: "Willwitcher-OS"; color: "#e8e6df"; font.pixelSize: 16; font.weight: Font.Bold }
                Text { text: "Kavat Edition"; color: "#8b90a0"; font.pixelSize: 11 }
            }
        }

        Repeater {
            model: ViewManager
            Rectangle {
                required property int index
                required property string display
                readonly property bool current: index === ViewManager.currentStepIndex
                readonly property bool done: index < ViewManager.currentStepIndex
                Layout.fillWidth: true
                height: 32; radius: 8
                color: current ? "#2a2f3f" : "transparent"

                Rectangle {
                    id: dot
                    x: 10; anchors.verticalCenter: parent.verticalCenter
                    width: 18; height: 18; radius: 9
                    color: parent.done ? "#7ec77a" : parent.current ? "#f2a93b" : "transparent"
                    border.width: parent.done || parent.current ? 0 : 1
                    border.color: "#4a5066"
                    Text {
                        anchors.centerIn: parent
                        text: parent.parent.done ? "✓" : (parent.parent.index + 1)
                        color: parent.parent.done || parent.parent.current ? "#14110a" : "#8b90a0"
                        font.pixelSize: 10; font.weight: Font.Bold
                    }
                }
                Text {
                    x: 40; anchors.verticalCenter: parent.verticalCenter
                    text: parent.display
                    color: parent.current ? "#e8e6df" : "#b4b7c2"
                    font.pixelSize: 14
                    font.weight: parent.current ? Font.Medium : Font.Normal
                }
            }
        }

        Item { Layout.fillHeight: true }

        // when the installer was started with the answers of an earlier installation
        Text {
            visible: Earlier.answers.from !== undefined
            text: "Filled in from " + Earlier.answers.from + ". The passwords and the disk are asked for again."
            color: "#f2a93b"; font.pixelSize: 12; wrapMode: Text.WordWrap
            Layout.fillWidth: true; Layout.bottomMargin: 8
        }

        Text {
            text: "Step " + (ViewManager.currentStepIndex + 1) + " of " + ViewManager.rowCount()
            color: "#8b90a0"; font.pixelSize: 12
        }

        Text {
            visible: debug.enabled
            text: "Debug"
            color: "#8b90a0"; font.pixelSize: 11
            MouseArea { anchors.fill: parent; onClicked: debug.toggle() }
        }
    }
}
