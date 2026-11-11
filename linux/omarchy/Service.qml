import QtQuick
import "common"

BackendService {
    appId: "heartwire"
    backendCommand: ["/usr/lib/heartwire/heartwire-backend", "--stdio"]
}
