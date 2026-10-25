# Versões e lançamentos

**Português (Brasil)** · [English](../en/versioning-and-releases.md) · [Índice](README.md)

## Versão

Versão atual: **0.1.0**. A fonte é `[workspace.package].version` de `Cargo.toml`. Crates internas, npm, lockfiles, cache da PWA, READMEs e primeira entrada versionada dos changelogs precisam coincidir. Este é o primeiro lançamento; mudanças ainda não lançadas ficam em **Não lançado** até receberem a próxima versão.

```powershell
npm run version:check
npm run version:set -- 0.1.0
```

O segundo comando define a versão do primeiro lançamento. Ele sincroniza arquivos e badges, sem criar commit, tag ou publicação. As notas dos dois changelogs são editadas manualmente.

Usamos [Versionamento Semântico](https://semver.org/lang/pt-BR/). Durante 0.x, uma correção compatível incrementa o patch; capacidades significativas ou mudanças incompatíveis incrementam o minor. Os changelogs seguem [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/), com PT-BR primeiro e EN equivalente. Nunca reescreva uma versão já publicada.

## Preparar

1. Sincronize versões e atualize notas e guias nos dois idiomas.
2. Revise dependências e lockfiles; regenere avisos de terceiros.
3. Execute os testes apropriados e registre limites em [VALIDACAO.md](../../VALIDACAO.md).
4. Gere o instalador no Windows x64.
5. Teste o artefato final e a instalação, incluindo instalação sobre a versão anterior, ícones, bandeja, idiomas, início com Windows e remoção.

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

Mudanças no áudio exigem tom e silêncio com continuidade RTP. Testes em Edge não substituem Safari e iPhone físico. Relate claramente o que foi executado; não declare uma medição de latência total a partir do tamanho do quadro.

## Artefatos

- Instalador: `target/release/bundle/nsis/HeartWire_<versão>_x64-setup.exe`.
- Executável: `target/release/heartwire.exe`.
- O pacote inclui `LICENSE`, `NOTICE`, documentação e a pasta `licenses/`.
- Uma distribuição portátil também deve incluir avisos e documentação; não publique apenas o EXE.

O instalador NSIS (`crates/heartwire/installer/heartwire.nsi`) é por usuário e não exige administrador. Não possui assinatura digital configurada nem mecanismo de atualização automática.

## GitHub

Repositório público: [saitofernando/heartwire](https://github.com/saitofernando/heartwire). Mantenedor: [saitofernando](https://github.com/saitofernando).

`.github/workflows/checks.yml` verifica pushes e pull requests. `.github/workflows/package.yml` gera o instalador e o SHA-256 por execução manual, como artefatos do workflow; não publica releases automaticamente.

As releases usam tag `v<versão>`, notas PT-BR/EN, instalador e hashes. O primeiro lançamento é `v0.1.0`. Não inclua certificados privados, logs pessoais ou perfis de testes nos arquivos publicados.
