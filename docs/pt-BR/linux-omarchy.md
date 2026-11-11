# HeartWire no Linux / Omarchy

[English](../en/linux-omarchy.md) · [Índice](README.md)

Em 10/11/2026, o Linux passou a usar o plugin nativo `saito.heartwire`. Abra pelo ícone do gatinho na barra ou pelo lançador HeartWire. O painel oferece Ouvir, Microfone, seleção de dispositivos, porta, link/QR e início com a sessão. O núcleo Rust de áudio continua o mesmo; egui e a bandeja KSNI não são compilados no pacote Omarchy.

## Construir e atualizar

Execute na sessão do Omarchy 4.5/Quickshell com os pacotes de `depends` e `makedepends` indicados em `packaging/arch/PKGBUILD` instalados:

```bash
bash linux/build
bash linux/test
bash scripts/package-omarchy.sh --install
```

Sem `--install`, o último comando gera o pacote Arch e SHA-256 em `dist/linux/`. O snapshot inclui alterações locais e arquivos novos não ignorados; não inclui caches nem configurações privadas ignoradas. Instalação usa pacman, guarda uma cópia do plugin anterior em `.build/plugin-backups`, habilita o painel à direita da barra e reinicia o shell uma vez. Para remover o painel, desabilite o plugin com `omarchy plugin disable saito.heartwire` e remova o pacote com `sudo pacman -R heartwire`.

A interface fica em `linux/omarchy`: alterações QML não compilam Rust. O HeartWire usa o perfil `omarchy-dev` com otimização nível 1, compilação incremental, quatro jobs por padrão e cache `target/` preservado inclusive entre pacotes. O build omite Slint/egui, GTK e a bandeja antiga; `CFLAGS=-fno-lto` evita incompatibilidade entre objetos GCC LTO e o linker Rust LLD. Para entrega otimizada integral, use `bash scripts/package-omarchy.sh --release`; esse perfil demora mais. O primeiro build ainda precisa compilar dependências.

## Integração e referências

O painel usa `qs.Ui.KeyboardPanel`, `PanelHero`, controles e cores `qs.Commons`, com conteúdo criado sob demanda. A barra usa `linux/omarchy/assets/heartwire-symbolic.svg`, derivado de `design/heartwire/heartwire-mono.svg` com a geometria original e preenchimento branco para a máscara da cor do tema. O lançador usa diretamente `design/heartwire/app.svg`, colorido, instalado em `hicolor/scalable/apps/heartwire.svg`. Há um único serviço por aplicativo, compartilhado pelos monitores. O lançador abre o painel com `omarchy-shell shell summon`; não cria janela desktop adicional. Esc, clique no ícone e clique fora recolhem o painel pelos mecanismos do shell.

Referências: [guia oficial de plugins](https://plugins.omarchy.org/develop.html), [shell do Omarchy](https://github.com/omacom/omarchy/blob/quattro/docs/omarchy-shell.md), painel Agentes instalado em `/usr/share/omarchy/shell/plugins/agents`, [omaudiopanel](https://github.com/thevideinfra/omaudiopanel) e [omarchy-tray-panel](https://github.com/omakits-x/omarchy-tray-panel). Nenhuma configuração de monitores ou arquivo de `/usr/share/omarchy` é alterado.

## Áudio e dados

Requer pipewire-pulse e libpulse. Enumera saídas com pactl; captura o monitor selecionado com parec e reproduz o microfone com pacat. PCM estéreo 48 kHz/20 ms, fila limitada; encerrar o backend libera seus clientes de áudio. Fechar o painel mantém uma transmissão iniciada pelo usuário; o botão Parar áudio encerra a transmissão. `heartwire --quit` interrompe o áudio e recolhe o painel. O serviço fica disponível no shell para a próxima abertura. `heartwire --status` informa o estado.

Dados e certificado ficam em `$XDG_DATA_HOME/com.heartwire.desktop` ou `~/.local/share/com.heartwire.desktop`; HEARTWIRE_DATA_DIR permanece disponível. O início automático retoma as preferências salvas. Use o celular na mesma rede e aceite o certificado local na primeira conexão.

Para usar o celular como entrada de outro app, crie os módulos temporários abaixo uma vez na sessão e anote os números retornados:

```bash
pactl load-module module-null-sink sink_name=heartwire_mic sink_properties=device.description=HeartWire_Virtual
pactl load-module module-remap-source master=heartwire_mic.monitor source_name=heartwire_microphone source_properties=device.description=HeartWire_Microphone
```

Selecione Microfone e destino HeartWire Virtual no painel; no app de chamada, HeartWire Microphone. Para desfazer, pare o áudio e descarregue com `pactl unload-module NUMERO`, primeiro a fonte e depois a saída. Os módulos desaparecem ao reiniciar a sessão.

## QR lido, mas o link não abre

Na ocorrência de 11/11/2026, o QR estava correto e o endereço respondia no próprio PC, mas o UFW descartava as conexões do celular para TCP 8081. Acesso local pelo PC não comprova acesso pela rede. Confira a porta exibida pelo painel, a interface e a sub-rede antes de criar uma regra:

```bash
ip -brief address
sudo ufw status numbered
journalctl -k -b --no-pager | rg 'UFW BLOCK.*DPT=8081'
```

Exemplo para uma rede **192.168.1.0/24**, interface **enp5s0** e porta **8081**: substitua os três valores pelos da sua rede confiável. O PC pode usar cabo Ethernet e o celular Wi-Fi, desde que estejam na mesma rede sem isolamento de clientes.

```bash
sudo ufw allow in on enp5s0 from 192.168.1.0/24 to any port 8081 proto tcp comment 'HeartWire local network'
sudo ufw status numbered
```

Leia novamente o QR atual, confira IP/porta e aceite o certificado local no navegador. Toque em Conectar no miniapp e confira os dispositivos conectados no painel. A regra é persistente; se mudar de rede, interface ou porta, revise-a. Para desfazer o exemplo:

```bash
sudo ufw delete allow in on enp5s0 from 192.168.1.0/24 to any port 8081 proto tcp
```

O instalador não abre portas automaticamente. Essa regra permite a página HTTPS e a sinalização; WebRTC também negocia tráfego UDP. Se a página abrir sem áudio, confira o dispositivo de saída e novos bloqueios nos logs antes de alterar outras regras. Não é necessário desativar o firewall ou encaminhar portas do roteador para a internet.

## Validação em Linux

Em 11/11/2026, o pacote 0.1.0-3 foi instalado sobre 0.1.0-2, com o gatinho conferido no painel e nos arquivos do lançador. O QR foi decodificado da tela; HTTPS respondeu com 200 e o serviço registrou conexões WebRTC. Depois de liberar a porta no UFW, o usuário confirmou que funcionou no celular. Esse relato não informa modelo, sistema, duração ou latência. Microfone no celular, retomada ao entrar na sessão e uso prolongado ainda não foram validados. A preparação da versão 0.2.0 está registrada em [VALIDACAO.md](../../VALIDACAO.md). [Registro](../linux-validation.json). Inventário de licenças Linux: [avisos](../../THIRD_PARTY_NOTICES.linux.md).
