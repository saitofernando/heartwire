# Avisos de terceiros / Third-party notices

O projeto tem sua própria [licença](LICENSE). As dependências mantêm suas
licenças, direitos autorais e avisos originais; este documento não as
relicencia. O inventário atual cobre **402 pacotes Cargo,
0 pacotes npm e os componentes nativos identificados**
para `x86_64-pc-windows-msvc`, incluindo ferramentas de compilação e testes
rotuladas separadamente. Uma ferramenta inventariada pode estar ausente do
executável distribuído.

- [Textos integrais e avisos para acompanhar a distribuição](licenses/THIRD_PARTY_NOTICES.txt).
- [Inventário, expressões de licença, versões, fontes, escopos e hashes](licenses/dependencies.json).
- [Proveniência dos textos omitidos pelos pacotes e complementos verificados](licenses/supplemental.json).

O libopus embutido em `audiopus_sys` tem sua própria licença BSD-3-Clause
e referências de patentes, preservadas nos avisos. Isso é separado da
licença ISC do wrapper. Drivers de vídeo e fontes do Windows usados em tempo
de execução são componentes do sistema, com termos próprios, e não são
distribuídos. VB-CABLE e seus binários não são distribuídos por este projeto.

Os componentes abaixo declaram MPL-2.0 e conservam essa licença. As fontes
exatas correspondentes às versões resolvidas estão disponíveis nos arquivos
de código-fonte indicados; os avisos completos acompanham a distribuição.
As demais dependências também têm links de fonte no inventário JSON.

| Componente / Component | Escopo / Scope | Código-fonte correspondente / Corresponding source |
| --- | --- | --- |


Para atualizar os avisos em uma máquina Windows x64 com Node e Cargo:

```powershell
npm ci
cargo fetch --locked
node scripts/third-party-notices.mjs
node scripts/third-party-notices.mjs --check
```

O gerador usa apenas módulos nativos do Node. Os arquivos são ordenados e
determinísticos, sem data de geração nem caminhos absolutos da máquina.
`--check` verifica conteúdo, hashes dos locks/manifests e complementos;
retorna erro se os arquivos estiverem desatualizados ou houver licença
original pendente. A opção de manutenção `--refresh-upstream` consulta
apenas os commits públicos registrados pelos próprios crates para recuperar
licenças omitidas e grava sua proveniência. Um SDK nativo atualizado exige
revisar e registrar os novos textos originais em `licenses/supplemental.json`.
Os textos originais mantêm seu idioma, inclusive avisos que proíbem tradução.

## English

The [project license](LICENSE) does not replace third-party licenses.
The [complete notices](licenses/THIRD_PARTY_NOTICES.txt) reproduce original
license and notice texts; the [inventory](licenses/dependencies.json)
identifies locked versions, source archives, license expressions, file hashes
and runtime/build/development scopes. Source archives for MPL-2.0 components
are linked in the table above. Build tools are included conservatively and
are not necessarily shipped with the application.

Bundled libopus has separate notices from its Rust wrapper. Graphics drivers
and Windows fonts used at run time are system components with their own terms
and are not redistributed; VB-CABLE is not redistributed either.
The inventory covers locked package source trees and explicitly identified
native components, rather than all software installed on a build machine.

Regenerate using the commands above and include the project LICENSE and the
complete notices with binary releases. The generator needs no npm production
dependency, normalizes line endings, preserves original copyright notices,
and fails its check when generated files drift or original notices are missing.
Verified upstream supplements are checked into `licenses/upstream/` with
public source URLs and SHA-256 hashes in `licenses/supplemental.json`.
