# Segurança e privacidade

**Português (Brasil)** · [English](../en/security.md) · [Índice](README.md)

## Acesso

HeartWire destina-se a uma rede local confiável. **Iniciar** abre a página e a sinalização HTTPS na porta configurada. Não há conta, senha, token de pareamento ou aprovação individual dos aparelhos. O QR contém endereço e modo; não é uma credencial. Um aparelho que alcance a porta pode tentar conectar no modo ativo.

A validação de `Origin` recusa páginas de outra origem, comparando protocolo, host e porta em HTTP/1.1 e HTTP/2. Ela não autentica um cliente: programas fora do navegador podem enviar ou omitir cabeçalhos.

Use **Parar** ao terminar. Autorize o aplicativo no Firewall somente nos perfis necessários. O app não configura UPnP nem encaminha portas do roteador. Internet pública, túneis e redes com participantes desconhecidos estão fora do uso previsto.

## Certificado HTTPS local

O PC gera um certificado autoassinado com seus IPs. TLS protege o canal, mas um certificado autoassinado não fornece a identidade pública verificada de uma autoridade certificadora. Antes de aceitar um aviso, confira o IP e a porta no próprio HeartWire.

O certificado é reutilizado enquanto válido e renovado para novos endereços, expiração ou arquivos inválidos. Isso pode exigir nova confirmação no celular. Não instale certificados obtidos de fontes desconhecidas.

Uma exceção aceita ao navegar não garante que o iOS confie no certificado para todas as APIs. Microfone e registro do service worker dependem das regras de contexto seguro do navegador. O HeartWire não instala um perfil de confiança no iPhone nem contorna essas regras. O ícone instalado por WebClip não comprova confiança HTTPS ou funcionamento do cache offline.

## Perfil do atalho no iPhone

**Instalar com ícone** baixa um `.mobileconfig` não assinado com somente um payload WebClip. Ele inclui o endereço local, o PNG do HeartWire, tela cheia e permissão para remover o atalho. Não contém CA, certificado, MDM, VPN ou política de administração, e não muda a confiança HTTPS.

Confira o endereço e o conteúdo antes de instalar pelo app Ajustes do iPhone. O perfil pode ser removido em **Geral → VPN e Gerenciamento de Dispositivo**, e o atalho também pode ser excluído da Tela de Início. O iOS agrupa perfis nessa área, inclusive os que contêm apenas um atalho; a presença ali não representa uma configuração de VPN ou inscrição em gerenciamento remoto.

O aviso de instalação explica o download e as etapas manuais. O pedido permanece pendente até a abertura pela Tela de Início; o site não consulta os perfis instalados no aparelho. A indicação de abertura identifica o contexto atual, não a assinatura do perfil nem a confiança no certificado. Instalar esse perfil não dispensa as verificações do certificado local.

## Dados e transporte

A mídia usa Opus em WebRTC com DTLS/SRTP. O app não configura STUN/TURN, relay de áudio ou conta em nuvem. Sinalização e candidatos ICE contêm endereços dos participantes.

O código do app não implementa telemetria nem grava o fluxo de áudio em arquivo. Preferências, certificados, chave e logs são locais. Sistema operacional, navegador e outros programas têm configurações próprias.

O service worker limita o cache à interface permitida e à página de indisponibilidade. Não guarda ofertas, respostas, áudio ou estado conectado. Cache offline não fornece transmissão com o PC desligado.

## Permissões e ciclo de vida

Fone captura a saída selecionada no Windows. Microfone pede permissão no celular e reproduz na saída escolhida no PC. Evite reproduzir voz nos mesmos alto-falantes próximos ao microfone.

Em Fone, a Media Session API pode mostrar nome e ícone do HeartWire e controles na Tela Bloqueada. Pausar interrompe só a reprodução local; a conexão permanece ativa. Parar desconecta. Os comandos não ativam o microfone nem abrem uma sessão nova. O iOS controla a execução em segundo plano e pode interromper áudio ou captura.

Recolher o painel mantém a transmissão. **Parar** encerra as sessões; **Sair** encerra o processo. **Iniciar com o Windows** é opcional e desligado por padrão. Aplica-se ao usuário atual e não instala um serviço do Windows. O modo salvo é retomado ao iniciar automaticamente.

VB-CABLE é um driver externo, instalado e licenciado separadamente; não é incluído pelo HeartWire.

## Arquivos e relatos

Dados do desktop: `%LOCALAPPDATA%\com.heartwire.desktop\` (preferências, certificado e `logs\`). A desinstalação mantém essa pasta; apague-a manualmente para remover esses dados. Logs podem conter IPs e nomes de dispositivos. Revise antes de compartilhar. Nunca publique `bridge-key.pem`: é a chave privada HTTPS.

Vulnerabilidades devem ser enviadas a [fernandosaito@hotmail.com.br](mailto:fernandosaito@hotmail.com.br), conforme [SECURITY.md](../../SECURITY.md). Não envie chaves, áudio privado ou detalhes de exploração em issues públicas.

O painel desktop é Rust nativo e chama o controlador diretamente no mesmo processo. O painel só entrega ao navegador os endereços raiz da ponte em execução; outros links são recusados. Os scripts de CDP servem apenas aos testes de navegador do miniapp; o app instalado não precisa de administrador nem de depuração remota.
