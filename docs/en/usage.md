# Usage

[Português (Brasil)](../pt-BR/uso.md) · **English** · [Index](README.md)

## Windows

Install HeartWire for the current user; neither installation nor normal use requires administrator privileges. The installer creates Start Menu and Desktop shortcuts and can run over a previous version, keeping preferences. Uninstalling removes the app, shortcuts and the start-with-Windows entry; preferences, certificate and logs remain in `%LOCALAPPDATA%\com.heartwire.desktop`.

The heart lives in the system tray. Click to open the panel; click again, click outside or press Esc to hide it. There are no classic close/minimize buttons. The right-click menu shows the status and offers **Open HeartWire**, **Start**, **Stop** and **Quit**. Opening HeartWire again from a shortcut shows the panel that is already running.

Under **Connection**, select the mode and named device. **Start** opens HTTPS and audio; **Stop** ends sessions and releases devices. Stop before changing output or port.

| Mode | Setup |
| --- | --- |
| Headphones | Select the Windows output playing the desired audio. The phone receives it. |
| Microphone | Select the Windows output for the phone microphone. Configure VB-CABLE to feed other apps as an input. |

## Connect the phone

1. Join the same network on computer and phone.
2. Start HeartWire and scan the QR code with the camera.
3. Verify the IP/port shown in the app. Safari may warn about the local certificate.
4. Tap **Connect**. In Microphone mode, allow microphone access.
5. Check playback and adjust the phone volume. See the locked-screen limitations below.

The green card counts devices with connected WebRTC sessions. Opening the page alone does not increase it. The QR collapses after connecting; **Show QR** adds another device.

If several network addresses appear, select the one on the phone's network in the selector next to the QR. **Copy** and **Open** use that address. Isolated guest Wi-Fi, VPNs and firewall rules can prevent connection. The default port is 8081.

## Install on the Home Screen

To install the shortcut with its image inside the profile:

1. Open the computer address in iPhone Safari and choose **Settings → Install with icon** in the miniapp.
2. Allow the profile download.
3. Open **iPhone Settings → Profile Downloaded → Install** and confirm the iOS steps.
4. Return to the Home Screen and open the new **HeartWire** icon.

The miniapp explains the download beforehand and keeps the instructions pending after the tap. Downloading the file does not finish installation: confirm in Settings and open the icon. **Opened from the Home Screen** appears when HeartWire is running in that display mode. The website cannot inspect installed iPhone profiles; Safari and the app can also have separate storage, so the previous page may keep its pending reminder.

The local profile is unsigned; iOS may show that notice during confirmation. It contains only a HeartWire WebClip: computer address, embedded PNG and full-screen launch. The icon needs no extra HTTPS download during this installation. There is no certificate, CA, MDM or remote device control. Apple documents the [profile installation flow](https://support.apple.com/en-gb/102400).

The profile appears under **Settings → General → VPN & Device Management** because that area lists iOS configuration profiles; this does not mean HeartWire installed a VPN. To remove it, delete the Home Screen shortcut or remove the HeartWire profile there.

Alternatively, use **Share → Add to Home Screen → Add** in Safari. This option uses the computer-served manifest and icons. No App Store package is needed.

The service worker stores only allowed interface assets and an unavailable-PC page. It needs trusted HTTPS; accepting a self-signed navigation warning may not permit registration. The browser interface continues working when that feature is unavailable. See [certificates](security.md).

There is no audio while the computer is off. The shortcut points to an IP and port: if they change, scan the new QR and update the shortcut. Reserving the computer's IP in the router avoids address changes.

## Locked screen and audio controls

In **Headphones** mode, HeartWire supplies its name, icon and playback state through the Media Session API. When iOS displays the controls, **Pause** pauses local playback while keeping the connection; **Play** resumes audio from the existing session. **Stop**, when available, disconnects. These controls do not start the microphone.

Audio may continue after returning to the Home Screen or locking the phone. Continued playback and visible controls depend on iOS; uninterrupted background playback is not guaranteed. If sound stops, open HeartWire and resume audio or reconnect. In **Microphone** mode, the system may also interrupt capture when leaving the app or locking the screen.

## Settings

Choose **Português (Brasil)** or **English** on the desktop. This preference is saved; the mobile miniapp also has a browser language selector. Port changes require the stream to be stopped.

**Start with Windows** is optional, off by default and applies to the current user. At Windows login, the app starts audio in the tray with the saved mode and opens the panel only if starting fails. It does not install a system service.

## VB-CABLE

VB-CABLE is a separately installed virtual audio cable for sending the phone microphone to another program:

1. Install the driver from the [vendor](https://vb-audio.com/Cable/).
2. Select **CABLE Input** as the HeartWire Microphone output.
3. Select **CABLE Output** as the input in Discord, OBS, Zoom or another program.

The names describe the cable endpoints: HeartWire writes to Input and the other program reads Output. Selecting physical speakers plays your voice through them and may cause acoustic feedback.
