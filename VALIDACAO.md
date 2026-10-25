# Validação — HeartWire

**Português (Brasil)** · [English](VALIDACAO.en.md)

## 0.1.0 — 24/10/2026

Resultados do executável HeartWire 0.1.0, do instalador Windows e das verificações do primeiro lançamento.

Ambiente: Windows 11 Pro x64 com GPU NVIDIA, Rust/Cargo 1.97.1, Node.js 24.21.0, npm 11.19.0 e NSIS 3.13.

A versão do compilador é fixada por `rust-toolchain.toml` e pelos workflows em Rust 1.97.1. A conversão de PCM usa blocos de dois bytes com `as_chunks_mut::<2>()`; a API é estável desde Rust 1.88 e respeita o mínimo declarado do projeto.

| Verificação | Resultado |
| --- | --- |
| `cargo test --workspace --locked` | 27 testes aprovados: 17 do motor e 10 do painel (preferências, idioma dos erros, posição junto à barra de tarefas em todas as bordas e com ícones ocultos, links permitidos e QR decodificado em Rust). |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Aprovado, sem avisos. |
| `cargo fmt --all --check` | Aprovado. |
| Opus com CMake 4.4.4 | Configuração e compilação limpas aprovadas com a política 3.5 definida pelo projeto. |
| `npm run test:ui` | Aprovado: miniapp e instalação guiada em Edge, sem exceções JavaScript. |
| `npm run test:pwa` | 9 testes Node aprovados: 5 de Media Session e 4 de PWA. |
| `npm run release:check` | Versão 0.1.0 consistente; 28 arquivos, 12 pares PT-BR/EN e 402 pacotes Cargo inventariados, sem pendências. |
| `scripts/pwa-smoke.mjs` contra o app | Aprovado: service worker, manifesto e ícones, PT-BR/EN, modo do PC, cache público, página offline e recuperação. |
| Sonda WebRTC real contra o app | Aprovada: HTTP/2, WebRTC com áudio, Media Session, pausa/retomada na mesma sessão, idiomas e parada. |
| `scripts/package.ps1` | Aprovado; `heartwire.exe` com 19,4 MB e `HeartWire_0.1.0_x64-setup.exe` com 5,58 MB. |
| Painel nativo | Aprovado no executável release e no app instalado: clique real no ícone da bandeja, início oculto com `--minimized`, segunda execução, abas, hover dos botões, Esc, clique fora, `--quit` e `--demo`. |

## Motor e HTTPS

Os testes cobriram filas de reprodução, descarte de áudio antigo, cadência, continuidade RTP durante tom e ausência de captura, certificados e validação de origem. A sessão integrada verificou transporte HTTPS/WebRTC com Opus.

A compatibilidade de compilação do Opus vendorizado foi verificada com CMake 4.4.4 e MSVC 2022 em árvores isoladas. Sem o ajuste de políticas, foi reproduzido o erro de compatibilidade anterior a CMake 3.5. Com `CMAKE_POLICY_VERSION_MINIMUM=3.5`, a configuração e a compilação de `opus.lib` passaram. Uma sonda de build do Cargo também confirmou que `.cargo/config.toml` fornece essa variável aos scripts de compilação. Esses resultados não substituem a execução do workflow hospedado.

O download do Web Clip foi verificado em HTTP/1.1 e HTTP/2: MIME, `no-store`, nome do arquivo, URL local e bytes exatos do PNG embutido. Idioma inválido e parâmetros desconhecidos retornaram 400; Host forjado e origem externa retornaram 403. Os identificadores permanecem estáveis por origem e o perfil contém somente o atalho, sem VPN, CA ou MDM.

## Painel da bandeja

O executável release foi testado com pasta de dados e porta próprias (`HEARTWIRE_DATA_DIR`). Com `--minimized`, iniciou a ponte sem mostrar o painel; uma segunda execução abriu o painel junto à bandeja com foco. As abas Conexão, Ajustes e Ajuda foram conferidas por captura; a altura acompanhou o conteúdo (586 px com QR, 417 px conectado, 367 px em Ajustes, 556 px em Ajuda) e a janela permaneceu ancorada. Esc e clique fora ocultaram o painel sem parar o áudio; `--quit` parou a ponte e encerrou o processo.

O QR é gerado em Rust com correção H, margem de quatro módulos e o ícone no centro. Os testes decodificam a imagem gerada, com e sem o ícone, e conferem o endereço. Isso não equivale a uma leitura com câmera de iPhone físico.

Consumo medido nesta máquina (12 núcleos), no app instalado transmitindo áudio para um celular com o painel aberto: cerca de 90 MB de memória privada em um único processo, sem processos auxiliares, e 0,5% de CPU, cerca de 6,5% de um núcleo, durante 15 segundos. A maior parte da memória vem do driver OpenGL da GPU. É uma medição pontual, não uma comparação entre máquinas.

A captura do Windows no README usa o modo `--demo`, com dados de exemplo e um QR de demonstração. As imagens móveis são prévias no navegador, sem captura de iPhone físico.

## Interface móvel e instalação guiada

O teste de navegador conferiu aviso antes do download, solicitação explícita, instruções pendentes após recarregar, reconhecimento de standalone mesmo com armazenamento vazio, reinstalação e idiomas PT-BR/EN. Também preservou o modo do PC e o cancelamento de permissões tardias do microfone.

A integração Media Session publicou metadados e estado de reprodução, pausou e retomou mantendo o mesmo token e peer, recuperou erro de reprodução e removeu metadados e comandos ao parar. Um comando antigo de reproduzir não reconectou a sessão encerrada. Os testes Node também cobriram comandos e limpeza. Isso não simula o bloqueio físico do iOS.

## PWA em HTTPS

A sonda contra o app em execução passou em Edge: manifesto, ícones PNG/Apple, idioma PT-BR/EN persistido ao abrir `/`, modo obtido do PC, service worker, cache restrito a recursos públicos, página offline e recuperação. Status e SDP ficaram fora do cache.

A exceção TLS da sonda foi limitada à chave pública do servidor. Não houve instalação de certificados no sistema. Esse teste não comprova a confiança do Safari no certificado ou o cache no iPhone.

## Relato do usuário em iPhone físico

Em 24/10/2026, o usuário confirmou que a instalação pelo perfil exibiu o ícone correto. Relatou também que o áudio continuou depois de voltar à Tela de Início e bloquear o celular, identificando o sistema como iOS 27.0. Modelo e duração não foram informados. Essas observações são separadas dos testes automatizados; não comprovam continuidade prolongada, cache offline ou funcionamento dos controles Media Session.

## Limites

- O teste integrado não mede a latência ponta a ponta até os fones.
- Testes em Edge não comprovam comportamento no Safari ou no iPhone físico.
- Confiança do certificado, cache offline, permissão do microfone, retomada e controles Media Session ainda exigem validação física sistemática.
- O iOS decide sobre controles de mídia e execução com a tela bloqueada; reprodução curta não garante continuidade prolongada.
- O leitor de tela (Narrador), um login completo do Windows, Bluetooth e VB-CABLE não foram testados. O driver VB-CABLE não estava instalado no ambiente de teste.
- Outras versões de Windows, outras GPUs, redes de convidados e a versão mínima de Rust declarada não formam uma matriz testada.
- Assinatura digital do instalador e atualização automática não estão configuradas.

Comandos e sondas estão no [guia de desenvolvimento](docs/pt-BR/desenvolvimento.md). O [modelo de segurança](docs/pt-BR/seguranca.md) explica o acesso sem autenticação individual de aparelhos.
