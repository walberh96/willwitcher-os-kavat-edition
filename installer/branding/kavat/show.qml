// What the installer shows while it works: pictures of the system that is being
// installed, one after another, each with a line that says what it is.
import QtQuick 2.15

Rectangle {
    id: show
    color: "#151821"

    // the pictures and what each one shows
    readonly property var pictures: [
        { file: "desktop", title: "The desktop", text: "Hyprland for the windows and Kavat Shell around them. Super+D opens the launcher." },
        { file: "launcher", title: "One launcher", text: "Apps, files, settings and sums, from the same line." },
        { file: "settings-icons", title: "One theme, every app", text: "Super+T changes the palette, the mode and the accent, and everything follows at once." },
        { file: "themes-light", title: "Dark or light", text: "Seven palettes, each with both sides." },
        { file: "files", title: "Files", text: "Super+E opens Yazi, which is also the window every app opens and saves files with." },
        { file: "find", title: "Find anything", text: "Super+Shift+F looks through your files, the whole disk and the text inside files." },
        { file: "notes-quick", title: "Notes", text: "Super+N saves a note from one line; a day and a time at its end set a reminder." },
        { file: "kav-dashboard", title: "Packages", text: "kav installs, updates and goes back. The AUR is reached only through a list that was read first." },
    ]
    property int at: 0
    property bool running: false

    Timer {
        interval: 7000
        repeat: true
        running: show.running
        onTriggered: { fade.restart(); }
    }
    // one picture fades out, the next fades in
    SequentialAnimation {
        id: fade
        NumberAnimation { target: content; property: "opacity"; to: 0; duration: 350 }
        ScriptAction { script: show.at = (show.at + 1) % show.pictures.length }
        NumberAnimation { target: content; property: "opacity"; to: 1; duration: 350 }
    }

    Item {
        id: content
        anchors.fill: parent
        anchors.margins: 18

        Image {
            id: picture
            source: "slides/" + show.pictures[show.at].file + ".png"
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: title.top
            anchors.bottomMargin: 14
            fillMode: Image.PreserveAspectFit
            smooth: true
            mipmap: true
        }
        Text {
            id: title
            anchors.bottom: line.top
            anchors.bottomMargin: 4
            anchors.horizontalCenter: parent.horizontalCenter
            text: show.pictures[show.at].title
            color: "#e8e6df"
            font.pixelSize: 20
            font.weight: Font.Bold
        }
        Text {
            id: line
            anchors.bottom: dots.top
            anchors.bottomMargin: 12
            anchors.horizontalCenter: parent.horizontalCenter
            text: show.pictures[show.at].text
            color: "#8b90a0"
            font.pixelSize: 13
        }
        Row {
            id: dots
            anchors.bottom: parent.bottom
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: 7
            Repeater {
                model: show.pictures.length
                Rectangle { required property int index; width: 7; height: 7; radius: 4; color: index === show.at ? "#f2a93b" : "#3a4054" }
            }
        }
    }

    function onActivate() { show.at = 0; show.running = true }
    function onLeave() { show.running = false }
}
