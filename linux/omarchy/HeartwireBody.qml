import QtQuick
import QtQuick.Layouts
import qs.Commons
import qs.Commons as Commons
import qs.Ui
import "common"

ColumnLayout {
    id: root
    property var service: null
    property bool panelOpen: false
    readonly property var snapshot: service ? service.state : ({})
    readonly property var status: snapshot.status || ({})
    readonly property var preferences: snapshot.preferences || ({})
    readonly property bool streaming: status.running === true
    readonly property bool editing: capture.popupOpen || playback.popupOpen || port.activeFocus
    property string captureId: preferences.captureDeviceId || ""
    property string playbackId: preferences.micDeviceId || ""
    property string mode: preferences.mode || "listen"
    property bool settingsOpen: false
    property int cursor: 0
    property var actions: settingsOpen ? [settings, capture, playback, port, save, refresh] : [listen, mic, power, settings, copyLink, openLink]
    function stepFocus(direction) { cursor = Math.max(0, Math.min(actions.length - 1, cursor + direction)); actions[cursor].forceActiveFocus() }
    function activate() { var item = actions[cursor]; if (!item || !item.enabled || !item.visible) return; if (item.clicked) item.clicked(); else if (item.open) item.open() }
    function configuration() { return {mode: mode, port: Number(port.text) || 8081, captureDeviceId: captureId || null, micDeviceId: playbackId || null} }
    function deviceOptions() { return [{value: "", label: strings.t("System default")}].concat((snapshot.devices || []).slice(0, 128).map(function(d) { return {value: String(d.id), label: String(d.name).slice(0, 160)} })) }
    spacing: Style.space(12)
    Commons.Translations { id: strings; directory: Qt.resolvedUrl("i18n") }
    RowLayout {
        Layout.fillWidth: true
        Button { id: listen; text: strings.t("Listen"); Layout.fillWidth: true; selected: root.mode === "listen"; enabled: !root.streaming; focusable: true; onClicked: root.mode = "listen" }
        Button { id: mic; text: strings.t("Microphone"); Layout.fillWidth: true; selected: root.mode === "mic"; enabled: !root.streaming; focusable: true; onClicked: root.mode = "mic" }
    }
    RowLayout {
        Layout.fillWidth: true
        PlainText { text: root.streaming ? strings.t("Running") : strings.t("Stopped"); Layout.fillWidth: true }
        ToggleSwitch { checked: root.streaming; busy: !root.service || !root.service.ready; onToggled: root.service.send(root.streaming ? "stop" : "start", root.configuration()) }
    }
    Button { id: power; text: root.streaming ? strings.t("Stop audio") : strings.t("Start audio"); Layout.fillWidth: true; selected: root.streaming; bordered: true; focusable: true; enabled: root.service && root.service.ready; onClicked: root.service.send(root.streaming ? "stop" : "start", root.configuration()) }
    PlainText { Layout.fillWidth: true; opacity: 0.65; text: root.mode === "listen" ? strings.t("Listen to this computer on your phone over the local network.") : strings.t("Use your phone as a microphone for this computer.") }
    PanelSeparator { Layout.fillWidth: true }
    RowLayout {
        Layout.fillWidth: true
        PanelSectionHeader { text: strings.t("AUDIO DEVICES"); Layout.fillWidth: true }
        Button { id: settings; text: root.settingsOpen ? strings.t("Done") : strings.t("Settings"); focusable: true; onClicked: root.settingsOpen = !root.settingsOpen }
    }
    ColumnLayout {
        visible: root.settingsOpen; Layout.fillWidth: true; spacing: Style.space(10)
        Dropdown { id: capture; Layout.fillWidth: true; label: strings.t("Computer audio"); value: root.captureId; options: root.deviceOptions(); enabled: !root.streaming; onChanged: function(value) { root.captureId = value } }
        Dropdown { id: playback; Layout.fillWidth: true; label: strings.t("Microphone destination"); value: root.playbackId; options: root.deviceOptions(); enabled: !root.streaming; onChanged: function(value) { root.playbackId = value } }
        RowLayout {
            Layout.fillWidth: true
            PlainText { text: strings.t("Start with the session"); Layout.fillWidth: true }
            ToggleSwitch { checked: root.preferences.autostart === true; onToggled: root.service.send("autostart", !root.preferences.autostart) }
        }
        PlainText { text: strings.t("Network port") }
        TextField { id: port; Layout.fillWidth: true; text: String(root.preferences.port || 8081); enabled: !root.streaming; validator: IntValidator { bottom: 1024; top: 65535 } onAccepted: { focus = false; root.service.send("save", Object.assign({}, root.preferences, root.configuration())) } }
        RowLayout {
            Layout.fillWidth: true
            Button { id: refresh; text: strings.t("Refresh"); focusable: true; Layout.fillWidth: true; onClicked: root.service.send("devices") }
            Button { id: save; text: strings.t("Save"); focusable: true; Layout.fillWidth: true; enabled: !root.streaming; onClicked: root.service.send("save", Object.assign({}, root.preferences, root.configuration())) }
        }
    }
    PlainText { visible: !root.settingsOpen; text: (root.snapshot.devices || []).length + " " + strings.t("outputs available"); opacity: 0.65 }
    PanelSeparator { Layout.fillWidth: true; visible: root.streaming }
    ColumnLayout {
        Layout.fillWidth: true; visible: root.streaming; spacing: Style.space(10)
        PanelSectionHeader { text: strings.t("OPEN ON YOUR PHONE") }
        Image { Layout.alignment: Qt.AlignHCenter; Layout.preferredWidth: Style.space(160); Layout.preferredHeight: Style.space(160); source: root.snapshot.qr ? "data:image/svg+xml," + encodeURIComponent(root.snapshot.qr) : ""; sourceSize.width: 320; sourceSize.height: 320; smooth: false }
        PlainText { Layout.fillWidth: true; text: root.snapshot.link || ""; font.pixelSize: Style.font.caption; horizontalAlignment: Text.AlignHCenter; wrapMode: Text.WrapAnywhere }
        RowLayout {
            Layout.fillWidth: true
            Button { id: copyLink; text: strings.t("Copy link"); focusable: true; Layout.fillWidth: true; onClicked: root.service.copy(root.snapshot.link || "") }
            Button { id: openLink; text: strings.t("Open"); focusable: true; Layout.fillWidth: true; onClicked: root.service.send("open", root.snapshot.link) }
        }
        PlainText { Layout.fillWidth: true; text: strings.t("Use the same Wi-Fi. Accept the local certificate on the first connection."); font.pixelSize: Style.font.caption; opacity: 0.65 }
    }
}
