# HeartWire on Linux / Omarchy

[Português (Brasil)](../pt-BR/linux-omarchy.md) · [Index](README.md)

HeartWire 0.2.0 uses the native `saito.heartwire` Omarchy plugin. Open it from the cat icon in the bar or the HeartWire launcher. The panel provides Listen and Microphone modes, device selection, port, link/QR and session startup. One Rust audio backend is shared across monitors; egui and the KSNI tray are not built into this package.

## Build and update

Run from an Omarchy 4.5/Quickshell session with the dependencies listed in `packaging/arch/PKGBUILD` installed:

```bash
bash linux/build
bash linux/test
bash scripts/package-omarchy.sh --install
```

Without `--install`, packages and SHA-256 checksums are written to `dist/linux/`. The snapshot includes tracked edits and non-ignored new source files. Installation uses pacman, backs up the user plugin under `.build/plugin-backups`, enables its bar widget and restarts the shell once. To remove it, run `omarchy plugin disable saito.heartwire` and `sudo pacman -R heartwire`.

QML files are in `linux/omarchy`; interface edits require no Rust build. The default `omarchy-dev` profile uses optimization level 1, incremental compilation, four jobs and a reusable `target/` cache. `CFLAGS=-fno-lto` avoids GCC LTO/LLD incompatibility. Use `bash scripts/package-omarchy.sh --release` for full optimization. Packages include license notices and documentation.

## Integration and icons

The panel uses the shell's `qs.Ui.KeyboardPanel`, `PanelHero`, controls and theme colors. `linux/omarchy/assets/heartwire-symbolic.svg` preserves the geometry of `design/heartwire/heartwire-mono.svg`, with white fill for the theme mask. The launcher uses the original color `design/heartwire/app.svg`, installed as `hicolor/scalable/apps/heartwire.svg`.

The launcher calls `omarchy-shell shell summon`; it opens no separate desktop window. Esc, the bar icon and outside clicks hide the panel. Closing it preserves an active stream; Stop audio stops it. `heartwire --quit` stops audio and hides the panel, while the service remains available. `heartwire --status` reports its state.

References: [official plugin guide](https://plugins.omarchy.org/develop.html), [Omarchy shell](https://github.com/omacom/omarchy/blob/quattro/docs/omarchy-shell.md), the installed Agents panel at `/usr/share/omarchy/shell/plugins/agents`, [omaudiopanel](https://github.com/thevideinfra/omaudiopanel) and [omarchy-tray-panel](https://github.com/omakits-x/omarchy-tray-panel). Monitor settings and `/usr/share/omarchy` are not modified.

## Audio and data

Requires pipewire-pulse and libpulse. Outputs are listed with pactl, selected output monitors are captured with parec and phone microphone audio is played with pacat. Audio uses stereo 48 kHz PCM, 20 ms frames and bounded queues. Stopping the backend releases its audio clients.

Data and the local certificate live in `$XDG_DATA_HOME/com.heartwire.desktop` or `~/.local/share/com.heartwire.desktop`; `HEARTWIRE_DATA_DIR` overrides this in development. Session startup resumes saved preferences. Use the same local network and accept the local certificate on first access.

To route the phone microphone into another app, create these temporary modules once per session and note their returned IDs:

```bash
pactl load-module module-null-sink sink_name=heartwire_mic sink_properties=device.description=HeartWire_Virtual
pactl load-module module-remap-source master=heartwire_mic.monitor source_name=heartwire_microphone source_properties=device.description=HeartWire_Microphone
```

Select Microphone and HeartWire Virtual in the panel, then HeartWire Microphone in the receiving app. To undo this, stop streaming and run `pactl unload-module ID` for the source first, then the sink. These modules disappear after restarting the session.

## QR reads, but the link does not open

On 2026-11-11, the displayed QR decoded correctly and the PC could open its own address, but UFW dropped the phone's TCP 8081 connections. Local access does not prove network access. Check the panel's port, interface and subnet first:

```bash
ip -brief address
sudo ufw status numbered
journalctl -k -b --no-pager | rg 'UFW BLOCK.*DPT=8081'
```

Example for **192.168.1.0/24**, interface **enp5s0**, port **8081**: replace all three values with your trusted local network settings. Ethernet on the PC and Wi-Fi on the phone work when they share the same network without client isolation.

```bash
sudo ufw allow in on enp5s0 from 192.168.1.0/24 to any port 8081 proto tcp comment 'HeartWire local network'
sudo ufw status numbered
```

Scan the current QR, verify the IP/port, accept the local browser certificate and tap Connect in the miniapp. Check the connected-device count. The rule persists; review it when the network, interface or port changes. To undo the example:

```bash
sudo ufw delete allow in on enp5s0 from 192.168.1.0/24 to any port 8081 proto tcp
```

The installer does not open firewall ports. This rule allows HTTPS and signaling; WebRTC also negotiates UDP traffic. If the page opens without audio, check the selected output and new firewall log entries before changing other rules. Disabling the firewall or forwarding router ports to the internet is unnecessary.

## Linux validation

On 2026-11-11, package 0.1.0-3 was installed over 0.1.0-2 and the cat icon was verified in the panel and launcher files. The on-screen QR decoded successfully, HTTPS returned 200 and WebRTC connections were observed. After the UFW adjustment, the user confirmed phone operation. Model, OS, duration and latency were not reported. Phone microphone mode, session resume and prolonged use remain untested. Preparation of 0.2.0 is recorded in [VALIDACAO.en.md](../../VALIDACAO.en.md). [Machine-readable record](../linux-validation.json) · [Linux license notices](../../THIRD_PARTY_NOTICES.linux.md).
