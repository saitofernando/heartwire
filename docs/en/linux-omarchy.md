# HeartWire on Linux / Omarchy

[Português](../pt-BR/linux-omarchy.md) · [Index](README.md)

Linux port prepared on 2026-11-10 from 0.1.0. This delivery is checked on Windows; Linux execution remains unverified. The published Windows installer has not been replaced.

```bash
sudo pacman -S --needed base-devel git
git clone https://github.com/saitofernando/heartwire.git
cd heartwire
bash scripts/package-omarchy.sh --install
```

The script archives the current commit, installs build dependencies with makepkg, builds, tests and installs an Arch package. Omit `--install` to only build. Packages and SHA-256 hashes are in `dist/linux`. Update with `git pull --ff-only` and repeat; remove with `sudo pacman -R heartwire`. Use the pinned Rust toolchain or a compatible compiler.

The tray uses KSNI/StatusNotifierItem on D-Bus, without GTK. Select Open in the tray menu; close/Esc hides the panel, Quit stops audio and exits. Linux fonts come from egui. Wayland decides focus and placement; exact Windows taskbar anchoring is not promised. The app id is `com.heartwire.desktop`.

Data and certificates live in `$XDG_DATA_HOME/com.heartwire.desktop` or `~/.local/share/com.heartwire.desktop`; HEARTWIRE_DATA_DIR overrides this. XDG autostart uses `--minimized` to resume saved audio. A second launch shows the running instance; `heartwire --quit` requests shutdown without starting another instance.

## Audio and a virtual microphone

Requires pipewire-pulse and libpulse utilities. pactl enumerates sinks, parec captures the selected sink's monitor, and pacat plays incoming microphone audio. PCM is stereo 48 kHz in 20 ms frames. Capture retains the latest frame; playback queues at most two. Server/client/browser buffering adds latency. Shutdown kills only the child clients owned by HeartWire and releases blocked PCM workers.

Create temporary session devices to route your phone microphone to another application:

```bash
pactl load-module module-null-sink sink_name=heartwire_mic sink_properties=device.description=HeartWire_Virtual
pactl load-module module-remap-source master=heartwire_mic.monitor source_name=heartwire_microphone source_properties=device.description=HeartWire_Microphone
```

Record the returned module IDs. Refresh HeartWire devices and choose Mic → HeartWire Virtual. Choose HeartWire Microphone as the other app's input. Do not make the virtual sink your system's default output. To undo, stop HeartWire and run `pactl unload-module ID` for the source first, then the sink. Recreate after session restart; do not repeatedly load devices with the same names.

## Validation

New unit tests cover device/monitor selection, removed outputs, virtual-device labeling, PCM byte order and bounded playback. The Linux PCM module is type-checked by Windows tests without starting audio clients. Real PipeWire audio, latency, Wayland tray/focus, autostart, suspension and both phone modes still need validation on Omarchy. See [usage](usage.md) for network/certificate requirements and [Linux notices](../../THIRD_PARTY_NOTICES.linux.md) for dependencies.

Local result: 30 Rust tests, 9 miniapp tests, Clippy, formatting and release checks passed. [Validation record](../linux-validation.json).
