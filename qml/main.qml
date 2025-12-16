import QtQuick
import QtQuick.Controls
import QtQuick.Window

// This must match the uri and version
// specified in the qml_module in the build.rs script.
import player

ApplicationWindow {
    height: 480
    title: qsTr("Prototype Music Player")
    visible: true
    width: 640
    color: palette.window

    MyObject {
        id: myObject
        number: 1
        string: qsTr("My String with my number: %1").arg(myObject.number)
    }

    Column {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        Label {
            text: qsTr("Number: %1").arg(myObject.number)
            color: palette.text
        }

        Label {
            text: qsTr("String: %1").arg(myObject.string)
            color: palette.text
        }

        Button {
            text: qsTr("Increment Number")

            onClicked: myObject.incrementNumber()
        }

        Button {
            text: qsTr("Say Hi!")

            onClicked: myObject.sayHi(myObject.string, myObject.number)
        }

        Button {
            text: qsTr("Quit")

            onClicked: Qt.quit()
        }
    }
}
