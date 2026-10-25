# Validation — HeartWire

[Português (Brasil)](VALIDACAO.md) · **English**

## 0.1.0 — 2026-10-24

Results for the HeartWire 0.1.0 executable, Windows installer and first-release checks.

Environment: Windows 11 Pro x64 with an NVIDIA GPU, Rust/Cargo 1.97.1, Node.js 24.21.0, npm 11.19.0 and NSIS 3.13.

The compiler version is pinned by `rust-toolchain.toml` and the workflows to Rust 1.97.1. PCM conversion uses two-byte chunks with `as_chunks_mut::<2>()`; the API has been stable since Rust 1.88 and respects the project's declared minimum.

| Check | Result |
| --- | --- |
| `cargo test --workspace --locked` | 27 tests passed: 17 engine and 10 panel tests (preferences, error language, placement against the taskbar on every edge and with hidden icons, allowed links and QR decoded in Rust). |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed with no warnings. |
| `cargo fmt --all --check` | Passed. |
| Opus with CMake 4.4.4 | Clean configure and build passed with the project's 3.5 policy setting. |
| `npm run test:ui` | Passed: miniapp and guided installation in Edge, with no JavaScript exceptions. |
| `npm run test:pwa` | 9 Node tests passed: 5 Media Session and 4 PWA. |
| `npm run release:check` | Version 0.1.0 consistent; 28 files, 12 PT-BR/EN pairs and 402 Cargo packages inventoried, with no issues. |
| `scripts/pwa-smoke.mjs` against the app | Passed: service worker, manifest and icons, PT-BR/EN, PC mode, public cache, offline page and recovery. |
| Real WebRTC probe against the app | Passed: HTTP/2, WebRTC with audio, Media Session, pause/resume in the same session, languages and stop. |
| `scripts/package.ps1` | Passed; `heartwire.exe` at 19.4 MB and `HeartWire_0.1.0_x64-setup.exe` at 5.58 MB. |
| Native panel | Passed on the release executable and the installed app: a real click on the tray icon, hidden start with `--minimized`, second launch, tabs, button hover, Esc, outside click, `--quit` and `--demo`. |

## Engine and HTTPS

Tests covered playback queues, stale audio discard, cadence, RTP continuity during tone and capture absence, certificates and origin validation. The integrated session verified HTTPS/WebRTC transport with Opus.

Build compatibility for the vendored Opus source was verified with CMake 4.4.4 and MSVC 2022 in isolated trees. Without the policy setting, the CMake 3.5 compatibility error was reproduced. With `CMAKE_POLICY_VERSION_MINIMUM=3.5`, configuration and the `opus.lib` build passed. A Cargo build probe also confirmed that `.cargo/config.toml` provides this variable to build scripts. These results do not replace the hosted workflow run.

The Web Clip download was verified over HTTP/1.1 and HTTP/2: MIME type, `no-store`, filename, local URL and exact embedded PNG bytes. Invalid language and unknown parameters returned 400; forged Host and external origin returned 403. Identifiers remain stable per origin and the profile contains only the shortcut, without VPN, CA or MDM.

## Tray panel

The release executable was tested with its own data folder and port (`HEARTWIRE_DATA_DIR`). With `--minimized`, it started the bridge without showing the panel; a second launch opened the panel next to the tray with focus. The Connection, Settings and Help tabs were checked by screenshot; the height followed the content (586 px with QR, 417 px connected, 367 px in Settings, 556 px in Help) and the window stayed anchored. Esc and an outside click hid the panel without stopping audio; `--quit` stopped the bridge and ended the process.

The QR is generated in Rust with H error correction, a four-module quiet zone and the icon in its center. Tests decode the generated image, with and without the icon, and check the address. This is not a physical iPhone camera test.

Usage measured on this machine (12 cores), on the installed app streaming audio to one phone with the panel open: about 90 MB private memory in a single process, with no helper processes, and 0.5% CPU, about 6.5% of one core, over 15 seconds. Most of the memory comes from the GPU's OpenGL driver. This is a single measurement, not a cross-machine comparison.

The README Windows screenshot uses `--demo` mode, with sample data and a demonstration QR. The mobile images are browser previews, without a physical iPhone screenshot.

## Mobile interface and guided installation

The browser test checked the notice before download, explicit request, pending instructions after reload, standalone recognition even with empty storage, reinstallation and PT-BR/EN. It also preserved the PC mode and cancellation of late microphone permissions.

The Media Session integration published metadata and playback state, paused and resumed while keeping the same token and peer, recovered from a playback error, and removed metadata and actions on stop. A stale play command did not reconnect an ended session. Node tests also covered actions and cleanup. This does not simulate physical iOS locking.

## PWA over HTTPS

The probe against the running app passed in Edge: manifest, PNG/Apple icons, PT-BR/EN persisted when opening `/`, PC mode, service worker, cache limited to public resources, offline page and recovery. Status and SDP stayed out of the cache.

The probe's TLS exception was limited to the server's public key. No certificate was installed in the system. This test does not prove Safari certificate trust or iPhone caching.

## User report on a physical iPhone

On 2026-10-24, the user confirmed that profile installation showed the correct icon. They also reported that audio continued after returning to the Home Screen and locking the phone, identifying the system as iOS 27.0. Model and duration were not provided. These observations are separate from automated tests; they do not prove prolonged continuity, offline caching or Media Session controls.

## Limits

- The integrated test does not measure end-to-end latency to the headphones.
- Edge tests do not prove behavior in Safari or on a physical iPhone.
- Certificate trust, offline caching, microphone permission, resume and Media Session controls still require systematic physical validation.
- iOS decides on media controls and locked-screen execution; brief playback does not guarantee prolonged continuity.
- The screen reader (Narrator), a full Windows sign-in, Bluetooth and VB-CABLE were not tested. The VB-CABLE driver was not installed in the test environment.
- Other Windows versions, other GPUs, guest networks and the declared minimum Rust version do not form a tested matrix.
- Installer code signing and automatic updates are not configured.

Commands and probes are in the [development guide](docs/en/development.md). The [security model](docs/en/security.md) explains access without per-device authentication.
