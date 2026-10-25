# Contributing

[Português (Brasil)](CONTRIBUTING.md) · **English**

Reports and contributions may use Brazilian Portuguese or English. PT-BR is primary; behavior changes must update both guides.

## Report issues

Check [troubleshooting](docs/en/troubleshooting.md) and reproduce with the current version. Include the HeartWire version, Windows, browser/iOS, mode, device, steps and observed result. Remove personal data from logs; never send `bridge-key.pem`. For vulnerabilities, use the private contact in [SECURITY.md](SECURITY.md).

## Change code

1. Set up the [environment](docs/en/development.md).
2. Preserve [audio invariants](docs/en/architecture.md).
3. Run relevant checks. Audio changes require tone, silence and cadence tests.
4. Record the change under **Unreleased** in both changelogs and update affected guides.
5. For dependency changes, update lockfiles and regenerate license notices.

```powershell
npm run version:check
npm run docs:check
npm run test:pwa
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Miniapp changes must pass `npm run test:ui`; Windows panel changes require the [panel test](docs/en/development.md#panel-test). The [development guide](docs/en/development.md) describes probes that open real devices and may play sound.

## Pull requests

Describe the problem, resulting behavior and checks run. Include screenshots for interface changes. Distinguish automated browser tests from physical iPhone tests and state unverified behavior.

Preserve [LICENSE](LICENSE), [NOTICE](NOTICE) and third-party notices. Versions, tags and binaries follow the [release process](docs/en/versioning-and-releases.md).
