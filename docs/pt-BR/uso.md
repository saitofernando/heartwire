# Uso

**Português (Brasil)** · [English](../en/usage.md) · [Índice](README.md)

## Windows

Instale o HeartWire para o usuário atual; nem a instalação nem a execução exigem administrador. O instalador cria atalhos no Menu Iniciar e na Área de Trabalho e pode ser executado sobre uma versão anterior, mantendo as preferências. Desinstalar remove o app, os atalhos e a entrada de início com o Windows; preferências, certificado e logs continuam em `%LOCALAPPDATA%\com.heartwire.desktop`.

O coração fica na bandeja. Clique para abrir o painel; clicar novamente, clicar fora ou pressionar Esc recolhe. O painel não tem botões clássicos de fechar/minimizar. O menu com botão direito mostra o estado e oferece **Abrir HeartWire**, **Iniciar**, **Parar** e **Sair**. Abrir o HeartWire de novo pelo atalho mostra o painel já em execução.

Na aba **Conexão**, escolha o modo e o dispositivo pelo nome. **Iniciar** abre o servidor HTTPS e o áudio; **Parar** encerra as sessões e libera os dispositivos. Para trocar a saída ou a porta, pare primeiro.

| Modo | Configuração |
| --- | --- |
| Fone | Selecione a saída do Windows que reproduz o som desejado. O celular recebe esse áudio. |
| Microfone | Selecione onde reproduzir a voz do celular no Windows. Para outros apps usarem como entrada, configure o VB-CABLE. |

## Conectar o celular

1. Conecte PC e celular à mesma rede.
2. Inicie o HeartWire e leia o QR com a câmera.
3. Confira o IP/porta no app. O Safari pode avisar sobre o certificado local.
4. Toque em **Conectar**. Em Microfone, permita o acesso ao microfone.
5. Confira o som e ajuste o volume do celular. Veja abaixo os limites com a tela bloqueada.

O cartão verde mostra os aparelhos com WebRTC conectado. Abrir a página sozinho não aumenta o contador. O QR recolhe após conectar; **Mostrar QR** permite adicionar outro aparelho.

Se existirem vários endereços de rede, escolha no seletor ao lado do QR o da mesma rede do celular. **Copiar** e **Abrir** usam esse endereço. Wi-Fi de convidados com isolamento, VPNs e regras de firewall podem impedir a conexão. A porta padrão é 8081.

## Instalar na Tela de Início

Para instalar o atalho com a imagem dentro do próprio perfil:

1. No Safari do iPhone, abra o endereço do PC e entre em **Ajustes → Instalar com ícone** no miniapp.
2. Permita o download do perfil.
3. Abra os **Ajustes do iPhone → Perfil Baixado → Instalar** e confirme as etapas do iOS.
4. Volte à Tela de Início e abra o novo ícone **HeartWire**.

O miniapp avisa antes do download e mantém as instruções pendentes após o toque. Baixar o arquivo não conclui a instalação: ainda é preciso confirmar em Ajustes e abrir o ícone. A indicação **Aberto pela Tela de Início** aparece quando o HeartWire está nesse modo de abertura. O site não pode consultar os perfis instalados no iPhone; Safari e app também podem ter armazenamento separado, então a página anterior pode continuar com o lembrete pendente.

O perfil local é não assinado; esse aviso pode aparecer na confirmação. Ele contém somente um WebClip HeartWire: endereço do PC, PNG embutido e abertura em tela cheia. O ícone não precisa ser baixado novamente por HTTPS durante essa instalação. Não há certificado, CA, MDM ou controle remoto do aparelho. A Apple descreve o [fluxo de instalação de perfis](https://support.apple.com/pt-br/102400).

O perfil aparece em **Ajustes → Geral → VPN e Gerenciamento de Dispositivo** porque essa área reúne os perfis de configuração do iOS; isso não significa que o HeartWire instalou uma VPN. Para remover, exclua o atalho na Tela de Início ou remova o perfil HeartWire nessa área.

Como alternativa, use **Compartilhar → Adicionar à Tela de Início → Adicionar** no Safari. Essa opção usa o manifesto e os ícones servidos pelo PC. Não é necessário passar pela App Store.

O service worker guarda somente a interface permitida e uma página de indisponibilidade. Ele exige HTTPS confiável; aceitar o aviso de navegação de um certificado autoassinado pode não permitir seu registro. A interface continua funcionando no navegador quando esse recurso não estiver disponível. Consulte [certificados](seguranca.md).

Com o PC desligado não há áudio. O atalho aponta para um IP e porta: se o endereço mudar, leia o novo QR e atualize o atalho. Reservar o IP do PC no roteador evita essa troca.

## Tela bloqueada e controles de áudio

No modo **Fone**, o HeartWire informa nome, ícone e estado de reprodução ao sistema pela Media Session API. Quando o iOS exibir os controles, **Pausar** silencia a reprodução local mantendo a conexão; **Reproduzir** retoma o áudio da sessão existente. **Parar**, quando disponível, desconecta. Esses controles não iniciam o microfone.

O áudio pode continuar após voltar à Tela de Início ou bloquear o celular. A continuidade e a exibição dos controles dependem do iOS; não há garantia de reprodução permanente em segundo plano. Se o som parar, abra o HeartWire e retome o áudio ou conecte novamente. No modo **Microfone**, o sistema também pode interromper a captura ao sair do app ou bloquear a tela.

## Ajustes

Escolha **Português (Brasil)** ou **English** no desktop. A preferência é salva; o miniapp móvel também permite escolher o idioma no navegador. A porta só muda com a transmissão parada.

**Iniciar com o Windows** é opcional, desligado por padrão e válido para o usuário atual. O app inicia o áudio na bandeja com o modo salvo e só abre o painel se a inicialização falhar. Não é instalado um serviço do sistema.

## VB-CABLE

VB-CABLE é um cabo de áudio virtual instalado separadamente, útil para enviar o microfone do celular a outro programa:

1. Instale o driver pelo [site do fabricante](https://vb-audio.com/Cable/).
2. Em Microfone no HeartWire, selecione **CABLE Input** como saída.
3. No Discord, OBS, Zoom ou outro programa, selecione **CABLE Output** como entrada.

Os nomes descrevem as pontas do cabo: o HeartWire escreve na Input e o outro programa lê da Output. Selecionar alto-falantes físicos reproduz a voz neles; isso pode gerar retorno acústico.
