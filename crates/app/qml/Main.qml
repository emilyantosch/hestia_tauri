pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import com.hestia.app

ApplicationWindow {
    id: window
    width: 1100
    height: 720
    visible: true
    title: qsTr("Hestia")

    property int selectedFolderId: -1
    property int selectedFileId: -1

    HestiaBackend {
        id: backend
        Component.onCompleted: refreshLibraries()
        onOperationFinished: {
            if (ready) {
                folders.refresh()
                files.refresh(window.selectedFolderId, search.text)
                tags.refresh()
            }
        }
    }
    FolderModel { id: folders }
    FileModel { id: files }
    TagModel { id: tags }

    FolderDialog {
        id: folderDialog
        title: qsTr("Choose the folder whose files Hestia should manage")
        onAccepted: backend.createLibrary(libraryName.text, selectedFolder)
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 10

        Label {
            Layout.fillWidth: true
            visible: backend.error.length > 0
            color: "crimson"
            text: backend.error
            wrapMode: Text.Wrap
        }

        BusyIndicator {
            Layout.alignment: Qt.AlignHCenter
            running: backend.busy
            visible: running
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !backend.ready

            Label {
                text: qsTr("Choose a Hestia library")
                font.pixelSize: 28
                font.bold: true
            }

            ListView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: backend.libraryNames
                delegate: Button {
                    required property int index
                    required property string modelData
                    width: ListView.view.width
                    text: modelData
                    onClicked: backend.openLibrary(index)
                }
            }

            RowLayout {
                Layout.fillWidth: true
                TextField {
                    id: libraryName
                    Layout.fillWidth: true
                    placeholderText: qsTr("New library name")
                    enabled: !backend.busy
                }
                Button {
                    text: qsTr("Choose folder and create")
                    enabled: libraryName.text.trim().length > 0 && !backend.busy
                    onClicked: folderDialog.open()
                }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: backend.ready

            RowLayout {
                Layout.fillWidth: true
                Label {
                    Layout.fillWidth: true
                    text: backend.status
                }
                TextField {
                    id: search
                    placeholderText: qsTr("Filter files or tags")
                    onAccepted: files.refresh(window.selectedFolderId, text)
                }
                Button {
                    text: qsTr("Scan")
                    enabled: !backend.busy
                    onClicked: backend.scan()
                }
            }

            SplitView {
                Layout.fillWidth: true
                Layout.fillHeight: true

                Pane {
                    SplitView.preferredWidth: 220
                    ColumnLayout {
                        anchors.fill: parent
                        Label { text: qsTr("Folders"); font.bold: true }
                        Button {
                            Layout.fillWidth: true
                            text: qsTr("All files")
                            onClicked: {
                                window.selectedFolderId = -1
                                files.refresh(-1, search.text)
                            }
                        }
                        ListView {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            clip: true
                            model: folders
                            delegate: ItemDelegate {
                                required property int id
                                required property string name
                                required property string path
                                width: ListView.view.width
                                text: name
                                ToolTip.text: path
                                ToolTip.visible: hovered
                                onClicked: {
                                    window.selectedFolderId = id
                                    files.refresh(id, search.text)
                                }
                            }
                        }
                    }
                }

                Pane {
                    SplitView.fillWidth: true
                    GridView {
                        anchors.fill: parent
                        clip: true
                        model: files
                        cellWidth: 160
                        cellHeight: 180
                        delegate: ItemDelegate {
                            required property int id
                            required property string name
                            required property url thumbnailUrl
                            width: 150
                            height: 170
                            onClicked: window.selectedFileId = id
                            contentItem: Column {
                                spacing: 6
                                Image {
                                    width: 140
                                    height: 135
                                    fillMode: Image.PreserveAspectFit
                                    source: thumbnailUrl
                                }
                                Label {
                                    width: 140
                                    text: name
                                    elide: Text.ElideMiddle
                                    horizontalAlignment: Text.AlignHCenter
                                }
                            }
                        }
                    }
                }

                Pane {
                    SplitView.preferredWidth: 230
                    ColumnLayout {
                        anchors.fill: parent
                        Label { text: qsTr("Tags"); font.bold: true }
                        Label {
                            Layout.fillWidth: true
                            visible: tags.error.length > 0
                            color: "crimson"
                            text: tags.error
                            wrapMode: Text.Wrap
                        }
                        ListView {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            clip: true
                            model: tags
                            delegate: RowLayout {
                                required property int id
                                required property string name
                                width: ListView.view.width
                                Label { Layout.fillWidth: true; text: name }
                                Button {
                                    text: "+"
                                    enabled: window.selectedFileId >= 0
                                    Accessible.name: qsTr("Assign %1").arg(name)
                                    onClicked: tags.assign(window.selectedFileId, id)
                                }
                                Button {
                                    text: "−"
                                    enabled: window.selectedFileId >= 0
                                    Accessible.name: qsTr("Remove %1").arg(name)
                                    onClicked: tags.unassign(window.selectedFileId, id)
                                }
                                Button {
                                    text: "×"
                                    Accessible.name: qsTr("Delete %1").arg(name)
                                    onClicked: tags.remove(id)
                                }
                            }
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            TextField {
                                id: newTag
                                Layout.fillWidth: true
                                placeholderText: qsTr("New tag")
                                onAccepted: {
                                    tags.create(text)
                                    clear()
                                }
                            }
                            Button {
                                text: qsTr("Add")
                                enabled: newTag.text.trim().length > 0
                                onClicked: {
                                    tags.create(newTag.text)
                                    newTag.clear()
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
