# Como contribuir

**Português (Brasil)** · [English](CONTRIBUTING.en.md)

Relatos e contribuições podem usar PT-BR ou inglês. PT-BR é o idioma principal; mudanças de comportamento devem atualizar os dois guias.

## Relatar problemas

Confira [problemas comuns](docs/pt-BR/solucao-de-problemas.md) e reproduza na versão atual. Informe versão do HeartWire, Windows, navegador/iOS, modo, dispositivo de áudio, passos e resultado observado. Remova dados pessoais dos logs; nunca envie `bridge-key.pem`. Para vulnerabilidades, use o contato privado de [SECURITY.md](SECURITY.md).

## Alterar código

1. Prepare o [ambiente](docs/pt-BR/desenvolvimento.md).
2. Preserve os [invariantes de áudio](docs/pt-BR/arquitetura.md).
3. Execute as verificações pertinentes. Mudanças de áudio exigem os testes de tom, silêncio e cadência.
4. Registre a mudança em **Não lançado** nos dois changelogs e atualize os guias afetados.
5. Para novas dependências, atualize lockfiles e regenere os avisos de licença.

```powershell
npm run version:check
npm run docs:check
npm run test:pwa
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Mudanças no miniapp devem passar em `npm run test:ui`; mudanças no painel do Windows exigem o [teste do painel](docs/pt-BR/desenvolvimento.md#teste-do-painel). O [guia de desenvolvimento](docs/pt-BR/desenvolvimento.md) descreve sondas que abrem dispositivos reais e podem reproduzir som.

## Pull requests

Descreva o problema, comportamento resultante e testes executados. Inclua captura quando a interface mudar. Diferencie um teste de navegador automatizado de um teste em iPhone físico e registre o que não foi validado.

Preserve [LICENSE](LICENSE), [NOTICE](NOTICE) e os avisos de terceiros. Versões, tags e binários seguem o [processo de lançamento](docs/pt-BR/versoes-e-lancamentos.md).
