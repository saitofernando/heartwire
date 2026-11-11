import QtQuick
import QtQuick.Controls as Controls
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Commons as Commons
import qs.Ui

Panel {
    id: root
    Commons.Translations { id: strings; directory: Qt.resolvedUrl("../i18n") }
    readonly property var shell: bar ? bar.shell : null
    property string appTitle: ""
    property string summary: ""
    property url iconSource
    property Component body
    property int panelWidth: 380
    property int maximumHeight: 780
    readonly property var service: shell ? shell.serviceFor(moduleName) : null
    property string viewId: String(Math.random())
    manageIpc: false
    ipcTarget: moduleName
    function inspection() { return {opened: opened, screen: popup.screen ? popup.screen.name : "", x: popup.cardOrigin.x, y: popup.cardOrigin.y, width: popup.contentWidth, height: popup.drawnHeight, runtime: service && service.appId === "camera-naomi" ? {status: service.camera.status, playing: service.camera.playing, shown: service.camera.shown} : null} }
    ShellIpc {
        target: root.moduleName
        function inspect(): string {
            var widgets = root.bar && root.bar.moduleWidgets ? root.bar.moduleWidgets(root.moduleName) : [root]
            return JSON.stringify(widgets.map(function(widget) { return widget.inspection() }))
        }
    }
    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight
    onOpenedChanged: {
        if (service) service.setPanel(viewId, opened)
        if (opened && service && !service.running) service.start()
    }
    Component.onDestruction: { if (service) service.setPanel(viewId, false) }
    BarIconButton {
        id: button; anchors.fill: parent; bar: root.bar
        tooltipText: root.appTitle
        iconComponent: Symbol { source: root.iconSource; color: root.barForeground; width: 24; height: 24 }
        onPressed: function(code) { if (code === Qt.LeftButton || code === Qt.RightButton) root.toggle() }
    }
    KeyboardPanel {
        id: popup
        anchorItem: button; owner: root; bar: root.bar; open: root.opened
        contentWidth: fittedContentWidth(Style.space(root.panelWidth))
        contentHeight: fittedContentHeight(loadedContentHeight, Style.space(root.maximumHeight))
        focusTarget: loadedContent
        content: PanelKeyCatcher {
            id: keys
            implicitHeight: column.implicitHeight
            anchors.fill: parent
            blocked: loader.item && loader.item.editing === true
            onCloseRequested: root.close()
            onTabRequested: function(direction) { if (loader.item && loader.item.stepFocus) loader.item.stepFocus(direction) }
            onMoveRequested: function(dx, dy) { if (loader.item && loader.item.stepFocus) loader.item.stepFocus(dy || dx) }
            onActivateRequested: { if (loader.item && loader.item.activate) loader.item.activate() }
            Flickable {
                id: flick; anchors.fill: parent; clip: true
                contentWidth: width; contentHeight: column.implicitHeight
                boundsBehavior: Flickable.StopAtBounds; flickableDirection: Flickable.VerticalFlick
                interactive: contentHeight > height
                Controls.ScrollBar.vertical: Controls.ScrollBar { policy: Controls.ScrollBar.AsNeeded }
                Column {
                    id: column; width: flick.width; spacing: Style.space(16)
                    PanelHero {
                        title: root.appTitle; meta: root.summary
                        iconComponent: Symbol { source: root.iconSource; width: Style.space(30); height: width }
                        trailingControl: Button { iconText: "×"; tooltipText: ""; focusable: true; onClicked: root.close() }
                    }
                    PanelSeparator { width: parent.width }
                    Loader { id: loader; width: parent.width; sourceComponent: root.body; active: true }
                    PlainText { width: parent.width; visible: root.service && root.service.lastError !== ""; text: root.service ? strings.t(root.service.lastError) : ""; color: Commons.Color.urgent }
                    Button { visible: root.service && !root.service.running; text: strings.t("Retry"); focusable: true; onClicked: root.service.start() }
                }
            }
        }
    }
}
