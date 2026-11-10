# HeartWire no Linux / Omarchy

[English](../en/linux-omarchy.md) · [Índice](README.md)

Porte preparado em 10/11/2026 a partir da versão 0.1.0. Os testes desta entrega rodam no Windows; a execução em Linux não foi validada. O instalador Windows publicado continua sendo a versão anterior. O pacote Linux é gerado do código atual com o procedimento abaixo.

## Instalar e atualizar

```bash
sudo pacman -S --needed base-devel git
git clone https://github.com/saitofernando/heartwire.git
cd heartwire
bash scripts/package-omarchy.sh --install
```

O script usa o commit atual, instala dependências com makepkg, compila, testa e instala um pacote Arch via pacman. Sem `--install`, só gera o pacote. Resultado e hashes em `dist/linux/`. Para atualizar, use `git pull --ff-only` e repita; para remover, `sudo pacman -R heartwire`. Requer Rust da versão indicada em rust-toolchain.toml ou compatível; se usar rustup, ele instala o toolchain fixado.

Abra HeartWire no lançador; use Abrir no menu da bandeja do Omarchy. Fechar ou Esc esconde; Sair encerra o áudio e o aplicativo. A bandeja usa StatusNotifierItem/D-Bus, backend KSNI, sem GTK. As fontes Linux vêm do egui. Wayland controla foco e posição da janela: o posicionamento exato junto ao ícone Windows não é garantido. O app id é `com.heartwire.desktop` para regras pessoais do compositor.

Dados e certificado ficam em `$XDG_DATA_HOME/com.heartwire.desktop` ou `~/.local/share/com.heartwire.desktop`; HEARTWIRE_DATA_DIR continua disponível. Iniciar com o sistema usa autostart XDG e `--minimized`, retomando o áudio com a configuração salva. Uma segunda abertura mostra a primeira instância; `heartwire --quit` solicita saída sem abrir outra.

## Áudio e microfone virtual

Requer pipewire-pulse ativo na sessão e utilitários do pacote libpulse. O app enumera saídas com pactl, captura o monitor da saída selecionada via parec e reproduz o microfone via pacat. Usa PCM estéreo 48 kHz / 20 ms; captura mantém só o quadro mais recente e a fila de reprodução mantém até dois quadros. Buffers do servidor, clientes e navegador também afetam latência. Encerrar mata apenas os clientes de áudio criados pelo app e libera as threads, inclusive após falha do servidor.

Para usar o celular como microfone em outro aplicativo, crie uma saída virtual e uma fonte associada na sessão:

```bash
pactl load-module module-null-sink sink_name=heartwire_mic sink_properties=device.description=HeartWire_Virtual
pactl load-module module-remap-source master=heartwire_mic.monitor source_name=heartwire_microphone source_properties=device.description=HeartWire_Microphone
```

Anote os dois números retornados. No HeartWire, atualize os dispositivos, selecione Microfone e a saída HeartWire Virtual. No aplicativo de chamada, escolha HeartWire Microphone como entrada. Não selecione esse dispositivo como saída geral do sistema. Para desfazer, pare o HeartWire e use `pactl unload-module NUMERO` primeiro na fonte e depois na saída. Esses módulos são temporários; recrie após reiniciar a sessão. Não execute os comandos repetidamente se os nomes já existirem.

## Validação e retomada

Testes novos verificam seleção de saída/monitor, saída removida, nomes e dispositivo virtual, PCM little-endian e fila limitada. O módulo PCM Linux também é compilado nos testes Windows sem iniciar parec/pacat. Verificações locais não comprovam PipeWire real, latência, bandeja Wayland, foco, cofre de certificados do navegador ou suspensão.

No Omarchy, validar pacote, bandeja, autostart, instância única, os dois modos com o celular, desconexão do dispositivo/servidor, fechar e reabrir durante transmissão. Permanecem as condições de rede local e certificado do [guia de uso](uso.md). Inventário de licenças Linux: [avisos](../../THIRD_PARTY_NOTICES.linux.md), gerados com `node scripts/third-party-notices.mjs --linux`.

Resultado local: 30 testes Rust, 9 testes do miniapp, Clippy, formatação e checks de entrega aprovados. [Registro de validação](../linux-validation.json).
