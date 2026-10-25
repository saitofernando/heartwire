# Changelog

[Português (Brasil)](CHANGELOG.md) · **English**

Notable HeartWire changes. Based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/).

## 0.1.0 — 2026-10-24

First HeartWire release.

### Added

- Rust engine with WASAPI capture and playback, Opus and local WebRTC.
- 20 ms frames, stale capture discard and continuous sending during silence, with a continuous RTP clock and a bounded playback queue.
- Native Rust tray panel built with egui/eframe (OpenGL), `tray-icon`/`muda` and AccessKit accessibility: compact panel without a title bar, opened against the taskbar on any edge (including with the icon among hidden icons), hidden on outside click or Esc, and audio preserved while hidden. Buttons with hover, keyboard focus and a hand cursor. Tray menu with Open, status, Start, Stop and Quit.
- Single process: on one machine with an NVIDIA GPU, about 90 MB private memory and 0.5% CPU while streaming to one phone, with most of the memory in the OpenGL driver.
- HeartWire pixel art identity, green palette and icons for the application, tray, installer and phone, generated from the original artwork by `cargo xtask icons`.
- Headphones and Microphone modes, named device selection and separately installed VB-CABLE integration.
- Start/stop, prominent connected-device indicator and compact QR code rendered in Rust, with the original icon in its center, H error correction and a four-module quiet zone. It collapses after connecting and can be reopened.
- PT-BR/English, configurable port and optional start with Windows resuming the saved mode.
- Single instance: launching again shows the existing panel; `--quit` asks the running app to stop audio and exit; `HEARTWIRE_DATA_DIR` overrides the data folder for development; `--demo` shows sample data for screenshots.
- HTML and JavaScript mobile miniapp with Connection, Settings and Help tabs, a PWA manifest, icons and Home Screen launch.
- Web Clip profile with the original embedded PNG, validated local address, full-screen launch and removable shortcut, without VPN, certificates or MDM.
- Guided iPhone installation: notice before download, pending instructions and **Opened from the Home Screen** when running in that mode.
- Media Session integration in Headphones mode, with name, icon, playback state and play, pause and stop commands when offered by the system.
- Service worker restricted to public interface resources and an unavailable-PC page, without caching audio, signaling or connected state.
- Per-user NSIS installer without administrator rights, with a PT-BR/EN interface and a welcome page that introduces the app before installing; illustrated README, guides, tests — including decoding the generated QR — and third-party license inventory.
- MIT license and authorship by saitofernando.

### Known limitations

- Trusted local networks only; devices are not individually authenticated.
- HTTPS uses a local self-signed certificate. Browser trust affects microphone and service workers; the Web Clip does not change that trust.
- The website cannot inspect installed profiles. The Home Screen indication identifies the current display mode; Safari and the app may keep separate storage.
- iOS decides whether to show media controls and keep background audio running; the controls still need prolonged physical testing.
- Total latency depends on the network, browser and devices; 20 ms is frame size, not an end-to-end measurement.
- Installer code signing and automatic updates are not configured.
