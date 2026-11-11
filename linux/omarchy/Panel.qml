import QtQuick
import "common"
import qs.Commons as Commons
MiniPanel {
    id: root
    moduleName: "saito.heartwire"
    appTitle: "HeartWire"
    summary: service && service.state.status && service.state.status.running ? strings.t("Connected devices").arg(service.state.status.connectedPeers || 0) : strings.t("Computer and phone audio")
    iconSource: Qt.resolvedUrl("assets/heartwire-symbolic.svg")
    body: Component { HeartwireBody { service: root.service; panelOpen: root.opened } }
    Commons.Translations { id: strings; directory: Qt.resolvedUrl("i18n") }
}
