# Development

[Português (Brasil)](../pt-BR/desenvolvimento.md) · **English** · [Index](README.md)

## Environment

Windows 10/11 x64, Rust **MSVC 1.90+**, Visual Studio C++ Build Tools, Windows SDK and CMake. The installer requires NSIS 3 (`makensis` on PATH or in its default folder); GitHub's `windows-latest` runners include it. Node.js **22.12+** is used only for repository tooling and the miniapp tests, which use Microsoft Edge. Run commands at the root containing `Cargo.toml` and `package.json`.

The declared Rust minimum is not a tested version matrix. Opus compiles natively; the first build can take time. The installed app does not need development tools.

`rust-toolchain.toml` pins Rust **1.97.1**, rustfmt and Clippy for this project version. Both workflows use the same explicit version. Compiler updates accompany lint and test validation; the declared minimum remains 1.90.

To compile the Opus source vendored by `audiopus_sys 0.2.2` with CMake 4, `.cargo/config.toml` sets `CMAKE_POLICY_VERSION_MINIMUM=3.5`. Cargo applies this [documented CMake policy override](https://cmake.org/cmake/help/latest/envvar/CMAKE_POLICY_VERSION_MINIMUM.html) to build subprocesses in both local development and the Windows workflows.

```powershell
cargo run -p heartwire
```

`iniciar.cmd` runs the same command. To keep a development build away from the installed app's preferences, set a separate data folder first:

```powershell
$env:HEARTWIRE_DATA_DIR = "$PWD\target\dev-data"
cargo run -p heartwire
```

If Visual Studio Build Tools CMake is not on PATH, set its installed path:

```powershell
$env:CMAKE = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe'
```

## Structure

| Path | Responsibility |
| --- | --- |
| `crates/bridge-core/src/audio.rs` | WASAPI enumeration, capture and playback. |
| `crates/bridge-core/src/scheduler.rs` | Real-frame and silence scheduling. |
| `crates/bridge-core/src/server.rs` | HTTPS, mobile resources, signaling and peers. |
| `crates/bridge-core/src/cert.rs` | Local certificate and renewal. |
| `crates/heartwire/src/main.rs` | Startup, logging, single instance and the `--minimized`, `--quit` and `--demo` options. |
| `crates/heartwire/src/app.rs` | egui panel: Connection/Settings/Help tabs, mode, device, QR, network and settings. |
| `crates/heartwire/src/controller.rs` | `BridgeService` and preferences. |
| `crates/heartwire/src/tray.rs` | Tray icon and menu. |
| `crates/heartwire/src/platform.rs` | Data folder, single instance, start with Windows, links and panel placement. |
| `crates/heartwire/src/qr.rs` | Local QR with the original icon, H error correction and a four-module quiet zone. |
| `crates/heartwire/src/widgets.rs` | Panel button with rest, hover, selected, disabled and focus states, accessible to screen readers. |
| `crates/heartwire/src/i18n.rs` | PT-BR/English panel text. |
| `crates/heartwire/installer/` | NSIS script and installer images. |
| `web/` | Mobile client, manifest, icons and service worker. |
| `web/media-session.js` | System metadata and commands for Headphones playback. |
| `scripts/` | Packaging, tests and version, documentation and license checks. |

## Checks

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm run test:ui
npm run test:pwa
npm run release:check
```

Engine tests cover playback queues, discard, cadence, certificates, origin checks and an HTTPS/WebRTC Opus connection. The integrated test checks continuous RTP during tone and absent capture. Read the [invariants](architecture.md) before changing the audio clock.

`heartwire` crate tests cover preferences, panel placement, allowed links and the QR: the rendered image is decoded in Rust to check its content, quiet zone and readability with the icon in its center. The result does not replace a physical iPhone camera test.

`test:ui` tests the mobile miniapp in headless Edge and also runs `npm run test:webclip`, checking the explicit installation request, languages, instructions and Home Screen launch detection. Standalone emulation verifies the interface, not whether an iOS profile exists. Rust tests verify the WebClip payload and HTTPS download; they do not install a profile on an iPhone.

`test:pwa` includes Node tests for caching and the Media Session helper. Media commands must respect the existing session: pause does not disconnect; stop clears integration; Microphone mode and an ended session must not resume through controls. Automated helper tests do not prove that iOS will display the controls.

The following probes open real devices and may play sound:

```powershell
cargo run -p bridge-core --example audio_probe -- --tone
cargo run -p bridge-core --example mic_probe -- 18081
cargo run -p bridge-core --example headless -- --list-devices
cargo run -p bridge-core --example headless -- --port 18080 --seconds 120
```

Use separate ports when running examples alongside the desktop.

## Panel test

The native panel is checked manually on Windows. Use `HEARTWIRE_DATA_DIR` to leave the installed app's preferences untouched, and close the installed app first: both copies share the single instance. **Start with Windows** uses the same registry entry; turn it off afterwards so it does not point to the development executable.

1. Open from the tray heart; the panel appears next to the tray. Click outside or press Esc and confirm audio continues.
2. In the tray menu, check **Open HeartWire**, the status, **Start**, **Stop** and **Quit**.
3. Start, scan the QR, connect a phone and check the counter, network selector, **Copy** and **Open**.
4. Switch Headphones/Microphone, language and port while stopped; turn **Start with Windows** on and off.
5. Launch HeartWire again: the existing panel should open. `heartwire.exe --quit` should stop audio and exit the app.
6. Start with `--minimized`: audio starts in the tray and the panel opens only if starting fails.

To test the PWA against a running service, replace the address with the IP and port shown by HeartWire:

```powershell
node scripts/pwa-smoke.mjs https://192.168.1.10:8081
```

The probe checks manifest, icons, languages, service worker, cache, unavailability and recovery in Edge. Its test TLS exception is scoped to that server's public key; it does not install system certificates. Service workers require trusted certificates; this test does not prove Safari installation. A physical iPhone test is a separate step.

On iPhone, test the installation notice, allow the download, confirm the profile in Settings and open the icon. Check **Opened from the Home Screen** and the option to repair or reinstall the icon. In Headphones mode, start playback, return to the Home Screen, lock the phone and check sound and controls. Test pause/resume, disconnect and language changes; ensure controls cannot resume an ended session or enable the microphone. Record the iOS version, observed duration and wired/Bluetooth output. A brief test does not guarantee prolonged background execution.

## Icons and installer

The source artwork is `design/heartwire/heartwire.svg`, with its monochrome variant alongside. App and tray icons live in `crates/heartwire/icons/`. The Rust generator (`xtask/`) samples the 16×16 grid without redrawing the art and writes the desktop, tray and PWA assets plus `icon.ico`:

```powershell
cargo xtask icons
pwsh -NoProfile -File scripts/installer-assets.ps1
```

The second command generates `header.bmp` and `sidebar.bmp` in `crates/heartwire/installer/`. The installer is defined in `crates/heartwire/installer/heartwire.nsi`: per user, without administrator rights, in `%LOCALAPPDATA%\HeartWire`. When changing it, retest a clean installation, installing over a previous version and removal.

## Distribution

```powershell
npm ci
npm run licenses:generate
npm run release:check
pwsh -NoProfile -File scripts/package.ps1
```

`scripts/package.ps1` runs `cargo build --release --locked -p heartwire` and then `makensis`. Executable: `target/release/heartwire.exe`. Installer: `target/release/bundle/nsis/HeartWire_<version>_x64-setup.exe`. Packages include notices and documentation. See [releases](versioning-and-releases.md) and [validation](../../VALIDACAO.en.md).
