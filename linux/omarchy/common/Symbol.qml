import QtQuick
import QtQuick.Effects
import qs.Commons as Commons
Item {
    id: root
    property url source
    property color color: Commons.Color.foreground
    implicitWidth: 24; implicitHeight: 24
    Image { id: glyph; anchors.fill: parent; source: root.source; sourceSize.width: 64; sourceSize.height: 64; visible: false; layer.enabled: true; fillMode: Image.PreserveAspectFit }
    Rectangle {
        anchors.fill: parent; color: root.color; layer.enabled: true
        layer.effect: MultiEffect { maskEnabled: true; maskSource: glyph; maskThresholdMin: 0.5; maskSpreadAtMin: 1 }
    }
}
