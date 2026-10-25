# Troubleshooting

[Português (Brasil)](../pt-BR/solucao-de-problemas.md) · **English** · [Index](README.md)

| Symptom | Check |
| --- | --- |
| Panel disappeared | Click the tray heart. Clicking outside or Esc only hides the panel. Check Windows hidden tray icons too. |
| App does not appear | Open HeartWire again from a shortcut: the running copy shows its panel. Logs are in `%LOCALAPPDATA%\com.heartwire.desktop\logs\`. |
| Phone cannot open the address | Use the same network. Check IP, port, firewall, VPN and guest-network isolation. |
| Certificate warning | Verify IP/port in HeartWire before continuing. The local certificate can renew after IP changes. |
| Connection origin not allowed | Open the current app-generated link without a proxy or changed port. Origin must match the HTTPS destination. |
| Page opens, 0 devices | Tap Connect and wait for WebRTC. An open page is not a connected session. |
| Headphones silent | Choose the Windows output actually playing the sound. Check computer, browser and headphone volume. |
| Microphone cannot connect | Check browser permission, HTTPS and certificate trust. Close other microphone sessions. |
| Voice missing in Discord/OBS | Choose CABLE Input in HeartWire and CABLE Output in the other app. Install VB-CABLE separately. |
| Delay or dropouts | Test close to the router with wired headphones. Bluetooth, congestion and browser buffers add delay. |
| Port occupied | Stop another instance or change the port in Settings with streaming stopped. |

## Safari and Home Screen

If **Share → Add to Home Screen** produces a generic icon, use **Settings → Install with icon** in the miniapp opened in Safari. After downloading, finish under **iPhone Settings → Profile Downloaded → Install** and open the new icon. The WebClip embeds the PNG without another image download.

If installation remains pending, open HeartWire using its Home Screen icon. The website receives no profile-installation confirmation: downloading the file or returning to Safari proves nothing. **Opened from the Home Screen** identifies the current display mode; Safari may keep its reminder because these contexts can have separate storage.

If **Profile Downloaded** is missing, download the profile again in Safari and open Settings promptly. Review or remove an existing profile under **General → VPN & Device Management**. This area also lists profile-installed shortcuts; HeartWire does not install a VPN. The local profile is unsigned, contains only the shortcut and installs neither certificates nor device administration.

**Share → Add to Home Screen → Add** remains an alternative without a profile. The shortcut profile does not resolve HTTPS certificate warnings.

An old icon or interface can remain in a shortcut. Reload the current address and, if needed, remove and recreate the shortcut. A new IP or port requires updating the saved address.

Service-worker offline support requires trusted HTTPS. Accepting a self-signed navigation warning does not guarantee registration. Worker failure must not block online use. The computer cannot send audio while switched off.

## Audio with the screen locked

In Headphones mode, the miniapp supplies its name, icon and commands through the Media Session API. The lock screen may display **Play/Pause** and, when supported, **Stop**. iOS decides whether to show these controls; their absence does not mean the connection dropped. Play resumes only the existing session, without enabling the microphone or creating a new connection.

If sound stops after locking, open HeartWire and resume playback or reconnect. Record the iOS version, mode, Bluetooth use, time until the interruption and whether the computer still shows a connected session. Brief locked-screen playback does not prove indefinite continuity. In Microphone mode, iOS may suspend capture.

## Report

Include app version, Windows, iOS/browser, mode, selected output and minimal steps. Review logs before attaching and never send private keys. See [contributing](../../CONTRIBUTING.en.md) and [security](security.md).
