// What the installer shows while it works: the logo and one line.
import QtQuick 2.0;
import calamares.slideshow 1.0;

Presentation {
    id: presentation

    Slide {
        Rectangle {
            anchors.fill: parent
            color: "#0d0f14"
        }
        Image {
            id: logo
            source: "kavat.svg"
            width: 160; height: 160
            sourceSize.width: 160; sourceSize.height: 160
            fillMode: Image.PreserveAspectFit
            anchors.centerIn: parent
        }
        Text {
            anchors.horizontalCenter: logo.horizontalCenter
            anchors.top: logo.bottom
            anchors.topMargin: 24
            text: qsTr("Installing Willwitcher-OS Kavat Edition")
            color: "#e8e6df"
            font.pixelSize: 20
        }
    }

    function onActivate() { presentation.currentSlide = 0; }
    function onLeave() { }
}
