# Versions and releases

[Português (Brasil)](../pt-BR/versoes-e-lancamentos.md) · **English** · [Index](README.md)

## Version

Current version: **0.1.0**. The source is `[workspace.package].version` in `Cargo.toml`. Internal crates, npm, lockfiles, the PWA cache, READMEs and the first versioned changelog entry must match. This is the first release; unreleased changes stay under **Unreleased** until they receive the next version.

```powershell
npm run version:check
npm run version:set -- 0.1.0
```

The second command sets the first-release version. It synchronizes files and badges without committing, tagging or publishing. Edit both changelog notes manually.

We use [Semantic Versioning](https://semver.org/). During 0.x, compatible corrections increment patch; significant capabilities or incompatible changes increment minor. Changelogs follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), with PT-BR first and equivalent English. Never rewrite a published version.

## Prepare

1. Synchronize versions and update notes and guides in both languages.
2. Review dependencies and lockfiles; regenerate third-party notices.
3. Run relevant checks and record limits in [VALIDACAO.en.md](../../VALIDACAO.en.md).
4. Build the installer on Windows x64.
5. Test the final artifact and installation, including installing over the previous version, icons, tray, languages, start with Windows and uninstall.

```powershell
npm ci
npm run licenses:generate
npm run release:check
npm run test:ui
npm run test:pwa
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pwsh -NoProfile -File scripts/package.ps1
```

Audio changes require tone and silence tests with continuous RTP. Edge tests do not replace Safari and a physical iPhone. State what actually ran; frame size is not an end-to-end latency measurement.

## Artifacts

- Installer: `target/release/bundle/nsis/HeartWire_<version>_x64-setup.exe`.
- Executable: `target/release/heartwire.exe`.
- The package includes `LICENSE`, `NOTICE`, documentation and `licenses/`.
- Portable distributions must also carry notices and documentation; do not publish the EXE alone.

The NSIS installer (`crates/heartwire/installer/heartwire.nsi`) applies to the current user and needs no administrator rights. Code signing and automatic updates are not configured.

## GitHub

Public repository: [saitofernando/heartwire](https://github.com/saitofernando/heartwire). Maintainer: [saitofernando](https://github.com/saitofernando).

`.github/workflows/checks.yml` checks pushes and pull requests. `.github/workflows/package.yml` generates the installer and SHA-256 as workflow artifacts when triggered manually; it does not publish releases automatically.

Releases use a `v<version>` tag, PT-BR/EN notes, installer and hashes. The first release is `v0.1.0`. Never include private certificates, personal logs or test profiles in published files.
