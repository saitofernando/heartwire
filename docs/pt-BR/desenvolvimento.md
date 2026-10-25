# Desenvolvimento

**Português (Brasil)** · [English](../en/development.md) · [Índice](README.md)

## Ambiente

Windows 10/11 x64, Rust **MSVC 1.90+**, Visual Studio C++ Build Tools, Windows SDK e CMake. O instalador exige NSIS 3 (`makensis` no PATH ou na pasta padrão); os runners `windows-latest` do GitHub já o incluem. Node.js **22.12+** é usado somente nas ferramentas do repositório e nos testes do miniapp, que usam Microsoft Edge. Execute os comandos na raiz que contém `Cargo.toml` e `package.json`.

A versão mínima declarada de Rust não equivale a uma matriz de versões testadas. O Opus é compilado nativamente; a primeira compilação pode demorar. O aplicativo instalado não precisa dessas ferramentas.

`rust-toolchain.toml` fixa Rust **1.97.1**, rustfmt e Clippy para esta versão do projeto. Os dois workflows usam a mesma versão explícita. Atualizações do compilador são feitas junto à validação dos lints e testes; o mínimo declarado continua em 1.90.

Para compilar o Opus vendorizado por `audiopus_sys 0.2.2` com CMake 4, `.cargo/config.toml` define `CMAKE_POLICY_VERSION_MINIMUM=3.5`. Esse [ajuste oficial de políticas do CMake](https://cmake.org/cmake/help/latest/envvar/CMAKE_POLICY_VERSION_MINIMUM.html) é aplicado aos subprocessos de compilação pelo Cargo, tanto no desenvolvimento quanto nos workflows Windows.

```powershell
cargo run -p heartwire
```

`iniciar.cmd` executa o mesmo comando. Para que uma versão de desenvolvimento não altere as preferências do app instalado, defina uma pasta de dados própria antes de executar:

```powershell
$env:HEARTWIRE_DATA_DIR = "$PWD\target\dev-data"
cargo run -p heartwire
```

Se o CMake do Visual Studio Build Tools não estiver no PATH, configure o caminho instalado:

```powershell
$env:CMAKE = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe'
```

## Estrutura

| Caminho | Responsabilidade |
| --- | --- |
| `crates/bridge-core/src/audio.rs` | Enumeração, captura e reprodução WASAPI. |
| `crates/bridge-core/src/scheduler.rs` | Agendamento de quadros reais e silêncio. |
| `crates/bridge-core/src/server.rs` | HTTPS, recursos móveis, sinalização e peers. |
| `crates/bridge-core/src/cert.rs` | Certificado local e renovação. |
| `crates/heartwire/src/main.rs` | Inicialização, logs, instância única e opções `--minimized`, `--quit` e `--demo`. |
| `crates/heartwire/src/app.rs` | Painel egui: abas Conexão/Ajustes/Ajuda, modo, dispositivo, QR, rede e ajustes. |
| `crates/heartwire/src/controller.rs` | `BridgeService` e preferências. |
| `crates/heartwire/src/tray.rs` | Ícone e menu da bandeja. |
| `crates/heartwire/src/platform.rs` | Pasta de dados, instância única, início com o Windows, links e posição do painel. |
| `crates/heartwire/src/qr.rs` | QR local com ícone original, correção H e margem de quatro módulos. |
| `crates/heartwire/src/widgets.rs` | Botão do painel com estados de repouso, hover, seleção, desabilitado e foco, acessível a leitores de tela. |
| `crates/heartwire/src/i18n.rs` | Textos PT-BR/English do painel. |
| `crates/heartwire/installer/` | Script NSIS e imagens do instalador. |
| `web/` | Cliente móvel, manifesto, ícones e service worker. |
| `web/media-session.js` | Metadados e comandos do sistema para reprodução em Fone. |
| `scripts/` | Empacotamento, testes, verificações de versões, documentação e licenças. |

## Verificações

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm run test:ui
npm run test:pwa
npm run release:check
```

Os testes do motor cobrem fila de reprodução, descarte, cadência, certificados, validação de origem e conexão HTTPS/WebRTC com Opus. O integrado verifica continuidade RTP durante tom e ausência de captura. Leia os [invariantes](arquitetura.md) antes de alterar o relógio de áudio.

Os testes da crate `heartwire` cobrem preferências, posição do painel, links permitidos e o QR: a imagem renderizada é decodificada em Rust para conferir o conteúdo, a margem e a leitura com o ícone no centro. O resultado não substitui um teste com a câmera de um iPhone físico.

`test:ui` testa o miniapp móvel em Edge sem interface visível e também executa `npm run test:webclip`, que confere o pedido explícito de instalação, idiomas, instruções e identificação da abertura pela Tela de Início. A simulação de standalone verifica a interface, não a existência de um perfil no iOS. Os testes Rust verificam o payload WebClip e o download HTTPS; isso não instala o perfil em um iPhone.

`test:pwa` reúne os testes Node do cache e do helper Media Session. Os comandos de mídia precisam respeitar a sessão existente: pausa não desconecta; parada limpa a integração; Microfone e sessão encerrada não podem ser retomados pelos controles. Testes automatizados do helper não comprovam que o iOS mostrará os controles.

As sondas abaixo abrem dispositivos reais e podem reproduzir som:

```powershell
cargo run -p bridge-core --example audio_probe -- --tone
cargo run -p bridge-core --example mic_probe -- 18081
cargo run -p bridge-core --example headless -- --list-devices
cargo run -p bridge-core --example headless -- --port 18080 --seconds 120
```

Use portas separadas ao executar exemplos junto ao desktop.

## Teste do painel

O painel nativo é conferido manualmente no Windows. Use `HEARTWIRE_DATA_DIR` para não alterar as preferências do app instalado e feche-o antes: as duas cópias compartilham a instância única. **Iniciar com o Windows** usa a mesma entrada do registro; desative ao terminar para que ela não aponte para o executável de desenvolvimento.

1. Abra pelo coração da bandeja; o painel aparece junto à bandeja. Clique fora ou pressione Esc e confirme que o áudio continua.
2. No menu da bandeja, confira **Abrir HeartWire**, o estado, **Iniciar**, **Parar** e **Sair**.
3. Inicie, leia o QR, conecte um celular e confira o contador, o seletor de rede, **Copiar** e **Abrir**.
4. Troque Fone/Microfone, idioma e porta com a transmissão parada; ative e desative **Iniciar com o Windows**.
5. Execute o HeartWire de novo: o painel existente deve abrir. `heartwire.exe --quit` deve parar o áudio e encerrar o app.
6. Inicie com `--minimized`: o áudio começa na bandeja e o painel só abre se houver falha.

Para testar a PWA contra um serviço em execução, substitua o endereço pelo IP e porta apresentados no HeartWire:

```powershell
node scripts/pwa-smoke.mjs https://192.168.1.10:8081
```

A sonda verifica manifesto, ícones, idiomas, service worker, cache, indisponibilidade e recuperação em Edge. A exceção TLS de teste é restrita à chave pública desse servidor; não instala certificados no sistema. Service worker exige certificado confiável; esse teste não comprova instalação no Safari. Teste físico em iPhone é uma etapa separada.

No iPhone, teste o aviso de instalação, permita o download, confirme o perfil em Ajustes e abra o ícone. Confira **Aberto pela Tela de Início** e a alternativa para corrigir ou reinstalar o ícone. Em Fone, inicie áudio, volte à Tela de Início, bloqueie a tela e verifique som e controles. Teste pausar/retomar, desconectar e mudar de idioma; confira que os controles não retomam uma sessão encerrada nem ativam o microfone. Registre versão do iOS, duração observada e saída com fio/Bluetooth. Um teste breve não garante execução prolongada em segundo plano.

## Ícones e instalador

A arte principal está em `design/heartwire/heartwire.svg`, com a variante monocromática ao lado. Os ícones do app e da bandeja ficam em `crates/heartwire/icons/`. O gerador em Rust (`xtask/`) amostra a grade 16×16 sem redesenhar a arte e grava os recursos de desktop, bandeja, PWA e o `icon.ico`:

```powershell
cargo xtask icons
pwsh -NoProfile -File scripts/installer-assets.ps1
```

O segundo comando gera `header.bmp` e `sidebar.bmp` em `crates/heartwire/installer/`. O instalador é definido em `crates/heartwire/installer/heartwire.nsi`, por usuário, sem administrador, em `%LOCALAPPDATA%\HeartWire`. Ao alterá-lo, teste novamente a instalação limpa, a instalação sobre uma versão anterior e a remoção.

## Distribuição

```powershell
npm ci
npm run licenses:generate
npm run release:check
pwsh -NoProfile -File scripts/package.ps1
```

`scripts/package.ps1` executa `cargo build --release --locked -p heartwire` e depois o `makensis`. Executável: `target/release/heartwire.exe`. Instalador: `target/release/bundle/nsis/HeartWire_<versão>_x64-setup.exe`. O pacote inclui licenças e documentação. Veja [lançamentos](versoes-e-lancamentos.md) e [validação](../../VALIDACAO.md).
