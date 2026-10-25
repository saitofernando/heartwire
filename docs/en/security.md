# Security and privacy

[Português (Brasil)](../pt-BR/seguranca.md) · **English** · [Index](README.md)

## Access

HeartWire is intended for a trusted local network. **Start** opens the page and HTTPS signaling on the configured port. There are no accounts, passwords, pairing tokens or individual device approvals. The QR contains address and mode, not a credential. A device able to reach the port can attempt to connect in the active mode.

`Origin` validation rejects other web origins, comparing protocol, host and port over HTTP/1.1 and HTTP/2. It does not authenticate clients: programs outside browsers can send or omit headers.

Use **Stop** when finished. Allow the app through the firewall only for necessary profiles. The app does not configure UPnP or router port forwarding. Public internet, tunnels and networks with unknown participants are outside the intended use.

## Local HTTPS certificate

The computer generates a self-signed certificate covering its IPs. TLS protects the channel, but self-signing does not provide a publicly verified identity. Before accepting a warning, verify the IP and port in HeartWire itself.

Certificates are reused while valid and renewed for new addresses, expiry or invalid files. This can require another phone confirmation. Do not install certificates from unknown sources.

A navigation exception does not guarantee iOS trusts the certificate for every API. Microphone and service worker registration depend on the browser's secure-context rules. HeartWire does not install an iPhone trust profile or bypass these rules. A WebClip-installed icon does not prove HTTPS trust or functioning offline caching.

## iPhone shortcut profile

**Install with icon** downloads an unsigned `.mobileconfig` containing only one WebClip payload. It includes the local address, HeartWire PNG, full-screen launch and shortcut removal permission. It contains no CA, certificate, MDM, VPN or administration policy and does not change HTTPS trust.

Check the address and contents before installing through iPhone Settings. Remove the profile under **General → VPN & Device Management**, or delete the Home Screen shortcut. iOS groups profiles there, including profiles containing only a shortcut; appearing there does not represent a VPN configuration or remote-management enrollment.

The installation notice explains the download and manual steps. The request remains pending until Home Screen launch; the website does not inspect installed device profiles. The launch indication identifies the current context, not profile signing or certificate trust. Installing this profile does not bypass local certificate checks.

## Data and transport

Media uses Opus in WebRTC with DTLS/SRTP. The app configures no STUN/TURN, audio relay or cloud account. Signaling and ICE candidates contain participant addresses.

The app implements no telemetry or audio-file recording. Preferences, certificates, key and logs remain local. The operating system, browser and other programs have their own settings.

The service worker limits caching to allowed interface assets and the unavailable-PC page. It does not store offers, answers, audio or connected state. Offline caching cannot stream with the computer off.

## Permissions and lifecycle

Headphones captures the selected Windows output. Microphone requests phone permission and plays to the chosen Windows output. Avoid playing your voice through speakers near the microphone.

In Headphones mode, the Media Session API may show the HeartWire name, icon and controls on the lock screen. Pause stops only local playback while the connection stays active. Stop disconnects. Commands do not enable the microphone or open a new session. iOS controls background execution and may interrupt audio or capture.

Hiding the panel keeps streaming. **Stop** ends sessions; **Quit** exits the process. **Start with Windows** is optional and off by default. It applies to the current user and does not install a Windows service. Automatic startup resumes the saved mode.

VB-CABLE is an external driver, installed and licensed separately; HeartWire does not bundle it.

## Files and reports

Desktop data: `%LOCALAPPDATA%\com.heartwire.desktop\` (preferences, certificate and `logs\`). Uninstalling keeps this folder; delete it manually to remove that data. Logs can contain IPs and device names. Review before sharing. Never publish `bridge-key.pem`, the HTTPS private key.

Send vulnerabilities to [fernandosaito@hotmail.com.br](mailto:fernandosaito@hotmail.com.br), following [SECURITY.md](../../SECURITY.md). Do not send private keys, audio or exploit details through public issues.

The desktop panel is native Rust and calls the controller directly in the same process. The panel hands only the running bridge's root addresses to the browser; other links are refused. CDP scripts serve only the miniapp browser tests; the installed app requires neither administrator privileges nor remote debugging.
