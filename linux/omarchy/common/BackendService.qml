import QtQuick
import Quickshell
import Quickshell.Io
import qs.Commons

Item {
    id: root
    required property string appId
    required property var backendCommand
    property var shell: null
    property var manifest: null
    property var state: ({})
    property string lastError: ""
    property bool ready: false
    property bool running: process.running
    property var _panels: ({})
    property var _pending: []
    property bool _resumeChecked: false
    signal shutdownRequested()
    ShellIpc {
        target: "saito." + root.appId + ".service"
        function status(): string {
            var safe = JSON.parse(JSON.stringify(root.state))
            if (safe.settings && safe.settings.url !== undefined) { safe.configured = !!safe.settings.url && String(safe.settings.url).indexOf("USUARIO") < 0; delete safe.settings.url }
            return JSON.stringify({ready: root.ready, running: root.running, state: safe})
        }
        function stop(): void { root.shutdownRequested(); root.send(root.appId === "heartwire" ? "stop" : "panel", false) }
    }
    function start() { if (!process.running) { lastError = ""; process.running = true } }
    function send(command, args) {
        if (typeof command !== "string" || command.length > 64) return
        var line = JSON.stringify({command: command, args: args === undefined ? null : args}) + "\n"
        if (line.length > 65536) return
        if (!process.running) { _pending = _pending.slice(-15).concat([line]); start(); return }
        process.write(line)
    }
    function setPanel(id, visible) {
        var panels = Object.assign({}, _panels)
        if (visible) panels[id] = true
        else delete panels[id]
        _panels = panels
        send("panel", Object.keys(panels).length > 0)
    }
    function receive(line) {
        if (line.length > 8 * 1024 * 1024) { lastError = "Invalid service response"; return }
        try {
            var message = JSON.parse(line)
            if (message && message.event === "message") { lastError = String(message.text || "").slice(0, 2048); return }
            if (!message || !message.state || typeof message.state !== "object" || Array.isArray(message.state)) return
            if (JSON.stringify(state) !== JSON.stringify(message.state)) state = message.state
            ready = true
            if (!_resumeChecked) {
                _resumeChecked = true
                if (appId === "heartwire" && state.preferences && state.preferences.autostart === true) send("resume")
            }
        } catch (_) { lastError = "Invalid service response" }
    }
    function copy(text) { clipboard.command = ["wl-copy", "--", String(text).slice(0, 8192)]; clipboard.running = true }
    Process {
        id: process
        command: root.backendCommand
        running: false
        stdinEnabled: true
        stdout: SplitParser { onRead: function(line) { root.receive(line) } }
        stderr: SplitParser { onRead: function(line) {} }
        onStarted: {
            var pending = root._pending; root._pending = []
            for (var i = 0; i < pending.length; i++) write(pending[i])
        }
        onExited: { root.ready = false; root.lastError = "Service stopped. Open the panel and choose Retry." }
    }
    Process { id: clipboard; running: false }
    Component.onCompleted: start()
}
