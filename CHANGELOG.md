# Changelog

**Português (Brasil)** · [English](CHANGELOG.en.md)

Mudanças relevantes do HeartWire. Organização baseada em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/); números seguem [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## 0.1.0 — 2026-10-24

Primeiro lançamento do HeartWire.

### Adicionado

- Motor Rust com captura e reprodução WASAPI, codec Opus e WebRTC na rede local.
- Quadros de 20 ms, descarte de captura antiga e envio contínuo durante silêncio, com relógio RTP sem saltos e fila de reprodução limitada.
- Painel nativo em Rust na bandeja, com egui/eframe (OpenGL), `tray-icon`/`muda` e acessibilidade por AccessKit: painel compacto sem barra de janela, aberto encostado na barra de tarefas em qualquer borda (inclusive com o ícone entre os ícones ocultos), recolhimento ao clicar fora ou pressionar Esc e áudio mantido com o painel oculto. Botões com hover, foco de teclado e cursor de mão. Menu da bandeja com Abrir, estado, Iniciar, Parar e Sair.
- Processo único: em uma máquina com GPU NVIDIA, cerca de 90 MB de memória privada e 0,5% de CPU transmitindo para um celular, a maior parte da memória do driver OpenGL.
- Identidade HeartWire em pixel art, paleta verde e ícones para aplicativo, bandeja, instalador e celular, gerados a partir da arte original por `cargo xtask icons`.
- Modos Fone e Microfone, seleção de dispositivos por nome e integração com VB-CABLE instalado separadamente.
- Iniciar/parar, indicador destacado de aparelhos conectados e QR code compacto renderizado em Rust, com o ícone original no centro, correção de erros H e margem de quatro módulos. Recolhe após conectar e pode ser reaberto.
- Idiomas PT-BR/English, porta configurável e opção de iniciar com o Windows retomando o modo salvo.
- Instância única: abrir de novo mostra o painel existente; `--quit` pede ao app em execução que pare o áudio e encerre; `HEARTWIRE_DATA_DIR` troca a pasta de dados em desenvolvimento; `--demo` mostra dados de exemplo para capturas de tela.
- Miniapp móvel em HTML e JavaScript com abas Conexão, Ajustes e Ajuda, manifesto PWA, ícones e abertura pela Tela de Início.
- Perfil Web Clip com o PNG original embutido, endereço local validado, abertura em tela cheia e atalho removível, sem VPN, certificados ou MDM.
- Instalação guiada no iPhone: aviso antes do download, instruções pendentes e indicação **Aberto pela Tela de Início** ao abrir nesse modo.
- Integração Media Session em Fone, com nome, ícone, estado de reprodução e comandos de reproduzir, pausar e parar quando oferecidos pelo sistema.
- Service worker com cache restrito aos recursos públicos da interface e página de indisponibilidade, sem guardar áudio, sinalização ou estado conectado.
- Instalador NSIS por usuário, sem administrador, com interface PT-BR/EN e tela de boas-vindas que apresenta o app antes de instalar; README ilustrado, guias, testes — incluindo a decodificação do QR gerado — e inventário de licenças de terceiros.
- Licença MIT e autoria de saitofernando.

### Limites conhecidos

- Rede local confiável; os aparelhos não têm autenticação individual.
- O certificado HTTPS é local e autoassinado. A confiança do navegador afeta microfone e service worker; o Web Clip não muda essa confiança.
- O site não consulta perfis instalados. A indicação da Tela de Início identifica o modo de abertura; Safari e app podem manter armazenamento separado.
- O iOS decide se exibe os controles de mídia e mantém o áudio em segundo plano; os controles ainda exigem teste físico prolongado.
- A latência total depende da rede, navegador e dispositivos; 20 ms é o tamanho do quadro, não uma medição ponta a ponta.
- Assinatura digital do instalador e atualização automática não estão configuradas.
