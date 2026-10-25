# Solução de problemas

**Português (Brasil)** · [English](../en/troubleshooting.md) · [Índice](README.md)

| Sintoma | Verificação |
| --- | --- |
| O painel sumiu | Clique no coração da bandeja. Clicar fora ou Esc apenas recolhe o painel. Verifique também a área de ícones ocultos do Windows. |
| O app não aparece | Abra o HeartWire de novo pelo atalho: a cópia em execução mostra o painel. Os logs ficam em `%LOCALAPPDATA%\com.heartwire.desktop\logs\`. |
| O celular não abre o endereço | PC e celular devem estar na mesma rede. Confira IP, porta, firewall, VPN e isolamento da rede de convidados. |
| Aviso de certificado | Confira IP/porta no HeartWire antes de continuar. O certificado é local e pode renovar ao trocar IP. |
| Origem de conexão não permitida | Abra o link atual gerado pelo app, sem proxy ou outra porta. A origem precisa coincidir com o destino HTTPS. |
| Página abre, mas há 0 aparelhos | Toque em Conectar e espere a sessão WebRTC. Uma página aberta não conta como conexão. |
| Fone sem som | Escolha a saída do Windows onde o som está sendo reproduzido. Confira volume do PC, navegador e fones. |
| Microfone não conecta | Confira permissão do navegador, HTTPS e confiança do certificado. Feche sessões que estejam usando o microfone. |
| Voz não chega ao Discord/OBS | Selecione CABLE Input no HeartWire e CABLE Output no outro app. O VB-CABLE precisa estar instalado. |
| Atraso ou cortes | Teste Wi-Fi próximo ao roteador e fones com fio. Bluetooth, rede congestionada e buffers do navegador acrescentam atraso. |
| Porta ocupada | Pare outra instância ou escolha outra porta em Ajustes com a transmissão parada. |

## Safari e Tela de Início

Se **Compartilhar → Adicionar à Tela de Início** criar um ícone genérico, use **Ajustes → Instalar com ícone** no miniapp aberto no Safari. Depois do download, conclua em **Ajustes do iPhone → Perfil Baixado → Instalar** e abra o novo ícone. O WebClip carrega o PNG embutido, sem depender de um novo download da imagem.

Se a instalação continuar pendente, abra o HeartWire pelo ícone da Tela de Início. O site não recebe confirmação de instalação de perfis: apenas baixar o arquivo ou voltar ao Safari não confirma nada. A indicação **Aberto pela Tela de Início** identifica o modo de abertura atual; o lembrete do Safari pode permanecer porque os contextos podem ter armazenamento separado.

Se **Perfil Baixado** não aparecer, baixe o perfil novamente no Safari e abra Ajustes logo depois. Para conferir ou remover um perfil existente, use **Geral → VPN e Gerenciamento de Dispositivo**. Essa área também lista atalhos instalados por perfil; o HeartWire não instala VPN. O perfil é local e não assinado, contém somente o atalho e não instala certificados nem administração do aparelho.

**Compartilhar → Adicionar à Tela de Início → Adicionar** continua disponível como alternativa sem perfil. O perfil de atalho não corrige avisos de certificado HTTPS.

O ícone antigo ou uma interface desatualizada pode permanecer no atalho. Recarregue pelo endereço atual e, se necessário, remova o atalho e adicione novamente. IP ou porta novos exigem atualizar o endereço salvo.

O modo offline do service worker exige HTTPS confiável. Aceitar o aviso de navegação de um certificado autoassinado não garante o registro. Uma falha de registro não deve impedir o uso online. O PC desligado nunca fornece áudio.

## Áudio com a tela bloqueada

Em Fone, o miniapp oferece nome, ícone e comandos pela Media Session API. A tela bloqueada pode exibir **Reproduzir/Pausar** e, quando suportado, **Parar**. A decisão de mostrar esses controles pertence ao iOS; sua ausência não significa que a conexão caiu. Reproduzir retoma apenas a sessão existente, sem ativar microfone ou criar uma conexão nova.

Se o áudio parar ao bloquear, abra o HeartWire e retome a reprodução ou conecte novamente. Registre versão do iOS, modo, uso de Bluetooth, duração até o corte e se a sessão continua conectada no PC. Uma reprodução curta com a tela bloqueada não comprova continuidade por tempo indeterminado. No modo Microfone, o iOS pode suspender a captura.

## Relatar

Inclua versão, Windows, iOS/navegador, modo, saída escolhida e passos mínimos. Revise logs antes de anexar; não envie chaves privadas. Consulte [contribuição](../../CONTRIBUTING.md) e [segurança](seguranca.md).
