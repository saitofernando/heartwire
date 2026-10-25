<div align="center">
  <img src="docs/assets/heartwire.svg" width="112" height="112" alt="Ícone HeartWire em pixel art">
  <h1>HeartWire</h1>
  <p>Áudio local, baixa latência</p>
  <p>
    <a href="CHANGELOG.md"><img src="https://img.shields.io/badge/HeartWire-0.1.0-79edbb?style=flat-square&amp;labelColor=171b18" alt="HeartWire 0.1.0"></a>
    <a href="docs/pt-BR/desenvolvimento.md"><img src="https://img.shields.io/badge/Windows-10%2F11%20x64-79edbb?style=flat-square&amp;labelColor=171b18" alt="Windows 10/11 x64"></a>
    <a href="Cargo.toml"><img src="https://img.shields.io/badge/Rust-2024-79edbb?style=flat-square&amp;labelColor=171b18" alt="Rust 2024"></a>
    <a href="crates/heartwire/Cargo.toml"><img src="https://img.shields.io/badge/egui-0.37-79edbb?style=flat-square&amp;labelColor=171b18" alt="egui 0.37"></a>
    <a href="docs/pt-BR/arquitetura.md"><img src="https://img.shields.io/badge/Opus-WebRTC-79edbb?style=flat-square&amp;labelColor=171b18" alt="Opus e WebRTC"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/licen%C3%A7a-MIT-79edbb?style=flat-square&amp;labelColor=171b18" alt="Licença MIT"></a>
  </p>
  <p>
    <a href="https://github.com/saitofernando/heartwire/releases/download/v0.1.0/HeartWire_0.1.0_x64-setup.exe">Baixar para Windows</a> ·
    <a href="#instalação-e-uso">Usar</a> ·
    <a href="#iphone-pwa-e-web-clip">iPhone e PWA</a> ·
    <a href="#tecnologia">Tecnologia</a> ·
    <a href="docs/pt-BR/README.md">Documentação</a> ·
    <a href="CONTRIBUTING.md">Contribuir</a>
  </p>
</div>

**Português (Brasil)** · [English](README.en.md)

**Versão: 0.1.0** · Windows 10/11 x64 · [MIT](LICENSE)

Transmita o áudio do Windows para o iPhone ou use o celular como microfone pela rede local. No Windows, o HeartWire é um app nativo em Rust que fica na bandeja; no celular, abre como um miniapp web em HTML e JavaScript, servido pelo próprio PC. Sem conta, sem nuvem e sem app para instalar no celular.

## Interface

<table>
  <tr>
    <th>Windows · bandeja</th>
    <th>Celular · conexão</th>
    <th>Celular · instalação</th>
  </tr>
  <tr>
    <td valign="top"><img src="docs/assets/desktop.png" width="280" alt="Painel HeartWire no Windows com um celular conectado e QR code de demonstração"></td>
    <td valign="top"><img src="docs/assets/mobile.png" width="280" alt="Miniapp web HeartWire em viewport móvel, com áudio conectado"></td>
    <td valign="top"><img src="docs/assets/home-screen.png" width="280" alt="Ajustes do miniapp, com botão Instalar com ícone e instruções antes da instalação"></td>
  </tr>
</table>

<sub>A captura do Windows usa o modo `--demo`, com dados de exemplo e um QR de demonstração; as móveis são prévias no navegador, sem captura de iPhone físico. Para conectar, leia o QR do HeartWire aberto no seu PC.</sub>

- **Painel nativo:** dispositivos por nome, iniciar/parar, aparelhos conectados e QR code recolhível com o ícone original no centro. Com suporte a leitores de tela.
- **Dois modos:** ouvir o PC no celular ou enviar o microfone do celular ao Windows.
- **Bandeja:** clicar fora ou pressionar Esc recolhe sem interromper a transmissão.
- **Ajustes:** PT-BR/English, porta e opção de iniciar com o Windows.

## Instalação e uso

Baixe o **[HeartWire 0.1.0 para Windows x64](https://github.com/saitofernando/heartwire/releases/download/v0.1.0/HeartWire_0.1.0_x64-setup.exe)** e execute o instalador. A instalação é para o usuário atual e não exige administrador. Rust e NSIS são necessários somente para [compilar](#compilar). [Notas do lançamento](https://github.com/saitofernando/heartwire/releases/tag/v0.1.0).

1. Instale o HeartWire e clique no ícone da bandeja.
2. Escolha **Fone** ou **Microfone** e o dispositivo de áudio.
3. Clique em **Iniciar** e leia o QR code com a câmera do iPhone.
4. Abra o endereço HTTPS e toque em **Conectar** no celular.

PC e celular precisam estar na mesma rede. No primeiro acesso, confira o IP mostrado no app antes de aceitar o aviso do certificado local. Porta padrão: **8081**.

| Modo | Caminho do áudio |
| --- | --- |
| Fone | Saída selecionada do Windows → celular → fones. |
| Microfone | Microfone do celular → saída selecionada do Windows. Para Discord, OBS ou Zoom, use **CABLE Input** no HeartWire e **CABLE Output** no outro app. O VB-CABLE é instalado separadamente. |

O indicador verde conta conexões WebRTC ativas. **Parar** encerra a transmissão; **Sair**, no menu da bandeja, encerra o app. [Guia de uso](docs/pt-BR/uso.md).

## iPhone: PWA e Web Clip

| Recurso | Papel no HeartWire |
| --- | --- |
| **PWA** | Miniapp web com manifesto, ícones e service worker para recursos públicos da interface. |
| **Web Clip** | Atalho da Apple com endereço, ícone embutido e abertura em tela cheia, instalado por um perfil `.mobileconfig`. |

Para instalar com o ícone incluído:

1. No Safari, abra o HeartWire e vá a **Ajustes → Instalar com ícone**.
2. Leia o aviso e permita o download do perfil.
3. Abra **Ajustes do iPhone → Perfil Baixado → Instalar**.
4. Volte à Tela de Início e abra o novo ícone **HeartWire**.

O miniapp mantém as instruções pendentes após o toque e mostra **Aberto pela Tela de Início** quando aberto nesse modo. Baixar o arquivo sozinho não confirma a instalação; o site não consegue consultar os perfis do iPhone.

O perfil aparece em **VPN e Gerenciamento de Dispositivo** porque o iOS reúne os perfis de configuração nessa área. Ele contém somente o atalho: não instala VPN, certificados ou gerenciamento remoto. **Compartilhar → Adicionar à Tela de Início** continua disponível sem perfil. [Instalação e remoção](docs/pt-BR/uso.md#instalar-na-tela-de-início).

O cache da PWA depende de HTTPS confiável; aceitar o aviso de navegação pode não bastar. Áudio exige PC ligado e conexão local. Em Fone, a integração Media Session oferece nome, ícone e comandos ao sistema; o iOS decide se mostra controles na Tela Bloqueada e mantém o áudio em segundo plano. [Limites e soluções](docs/pt-BR/solucao-de-problemas.md).

## Como funciona

**WASAPI → Opus → WebRTC → celular.** No modo Microfone, o fluxo volta do celular para uma saída do Windows.

O motor usa quadros de **20 ms**, descarta captura antiga e mantém a cadência durante o silêncio. A latência total depende da rede, navegador e fones, especialmente Bluetooth; o tamanho do quadro não representa a latência final. [Arquitetura e invariantes](docs/pt-BR/arquitetura.md).

O app não implementa conta, telemetria ou servidor de áudio em nuvem. Destina-se a uma **rede local confiável**, sem autenticação individual dos aparelhos. [Segurança e privacidade](docs/pt-BR/seguranca.md).

## Tecnologia

Um workspace Rust 2024 com o motor de áudio, o app do Windows e as tarefas do repositório. O celular usa somente HTML, CSS e JavaScript, sem framework nem etapa de build.

| Onde | Tecnologia |
| --- | --- |
| Painel e bandeja do Windows | Rust nativo: egui/eframe (OpenGL), `tray-icon`/`muda`, AccessKit e Win32 pela crate `windows`. |
| Motor de áudio | Rust: WASAPI, Opus, WebRTC (`webrtc-rs`), tokio e servidor HTTPS com axum/rustls. |
| Celular | HTML, CSS e JavaScript: WebRTC, Media Session, Wake Lock, service worker e manifesto PWA. |
| Qualidade e entrega | `cargo test`, clippy, testes Node, Edge sem interface via CDP, GitHub Actions e instalador NSIS. |

### Windows: painel e bandeja (`crates/heartwire`)

- **egui/eframe 0.37** em modo imediato, com renderização OpenGL. A escolha veio de medição: cerca de 80–90 MB com OpenGL contra mais de 330 MB com DX12 ou WARP neste painel.
- **Widget próprio** (`PanelButton`) com os estados de repouso, hover, seleção, desabilitado e foco de teclado; **AccessKit** informa nome, papel (botão, aba) e seleção aos leitores de tela.
- **Bandeja** com `tray-icon` e `muda`. O painel abre encostado na borda da barra de tarefas, em cima, embaixo ou nas laterais, calculada pela área de trabalho do monitor, e funciona também com o ícone na área de ícones ocultos.
- **Win32** pela crate `windows`: mutex e eventos nomeados para instância única e `--quit`; DWM para cantos arredondados; registro (`windows-registry`) para iniciar com o Windows, respeitando o Gerenciador de Tarefas; `ShellExecuteW` para abrir links.
- **Concorrência:** runtime tokio multi-thread. A interface nunca bloqueia: lê o estado com `try_lock`, e as tarefas assíncronas devolvem eventos por canal e acordam o egui com `request_repaint`, inclusive com a janela oculta.
- **QR** gerado em Rust (`qrcode`, correção H, ícone no centro) e validado em teste decodificando a imagem com `rqrr`.
- Fontes e ícones do próprio Windows (Segoe UI e Segoe Fluent Icons), sem arquivos de fonte embutidos. `--demo` mostra dados de exemplo para capturas de tela.

### Motor de áudio (`crates/bridge-core`)

- **WASAPI** em loopback orientado a eventos, em threads COM dedicadas. Quadros de 20 ms (960 amostras estéreo a 48 kHz) passam por um canal `watch`: sempre o quadro mais recente, nunca uma fila crescente.
- **Cadência:** agenda monotônica de 20 ms que envia silêncio quando a captura para, relógio RTP sem saltos e fila de reprodução do microfone limitada a 40 ms. `timeBeginPeriod(1)`, prioridade de thread alta e power throttling desativado mantêm o ritmo estável.
- **Opus** (libopus compilada com CMake): estéreo, FEC em banda e até 128 kbps.
- **WebRTC** com `webrtc-rs` (ICE e DTLS-SRTP), só candidatos da rede local, sem STUN/TURN; sinalização HTTP com oferta e resposta completas.
- **HTTPS:** axum 0.8, axum-server e rustls com HTTP/2; certificado autoassinado gerado com `rcgen` e renovado quando o IP muda; CSP restrita, validação de Host e Origin e limite de oito aparelhos.
- **Web Clip:** perfil `.mobileconfig` gerado com `plist`, com identificadores UUID v5 estáveis por endereço.

### Celular (`web/`)

- HTML, CSS e JavaScript puros, sem dependências npm nem build; os arquivos são embutidos no executável com `include_bytes!`.
- `RTCPeerConnection` com `playoutDelayHint` e `jitterBufferTarget` em zero, para o menor buffer do Safari; `getUserMedia` no modo Microfone.
- Media Session para os controles da Tela Bloqueada, Wake Lock, service worker que guarda só recursos públicos e manifesto PWA.

### Qualidade e entrega

- Testes Rust de cadência, continuidade RTP, certificados, origem, Web Clip, servidor em HTTP/2, posição do painel e QR; `clippy -D warnings` e `rustfmt`.
- Testes Node (`node:test`) do service worker e da Media Session; testes de navegador no Edge sem interface via CDP, sem Puppeteer; sonda WebRTC real contra o app.
- GitHub Actions em `windows-latest`, inventário de licenças de mais de 400 pacotes com hashes e checagem de links e versões.
- `cargo xtask icons` gera todos os ícones a partir da arte 16×16; instalador NSIS por usuário com página de boas-vindas.

```text
crates/bridge-core   motor de áudio, servidor HTTPS e WebRTC
crates/heartwire     app do Windows: painel egui, bandeja e instalador
web/                 miniapp do celular (HTML, CSS e JavaScript)
xtask/               tarefas do repositório, como cargo xtask icons
scripts/             verificações, testes de navegador e empacotamento
docs/                documentação PT-BR e EN
```

## Compilar

Requisitos: Windows x64, Rust MSVC **1.90+**, Visual Studio C++ Build Tools, Windows SDK e CMake. O instalador exige NSIS 3. Node.js **22.12+** é usado somente nas verificações do repositório e nos testes do miniapp; não faz parte do app.

```powershell
cargo run -p heartwire
cargo test --workspace
```

Para gerar o instalador:

```powershell
npm ci
npm run licenses:generate
npm run release:check
pwsh -NoProfile -File scripts/package.ps1
```

Saídas: `target/release/heartwire.exe` e `target/release/bundle/nsis/HeartWire_<versão>_x64-setup.exe`. O pacote inclui licenças e documentação. [Ambiente e testes](docs/pt-BR/desenvolvimento.md) · [Resultados de validação](VALIDACAO.md).

## Documentação e contribuição

[Uso](docs/pt-BR/uso.md) · [Solução de problemas](docs/pt-BR/solucao-de-problemas.md) · [Arquitetura](docs/pt-BR/arquitetura.md) · [Versões](docs/pt-BR/versoes-e-lancamentos.md) · [Changelog](CHANGELOG.md) · [Contribuir](CONTRIBUTING.md) · [Segurança](SECURITY.md)

PT-BR é a documentação principal. [English documentation](docs/en/README.md).

## Licença e autoria

Criado e mantido por **[saitofernando](https://github.com/saitofernando)**. HeartWire é distribuído sob [MIT](LICENSE). Componentes de terceiros conservam seus próprios avisos em [NOTICE](NOTICE) e [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

[fernandosaito@hotmail.com.br](mailto:fernandosaito@hotmail.com.br)
