use crate::preferences::Language;

/// Every panel string, checked at compile time: a missing translation is a
/// missing struct field.
pub struct Texts {
    pub subtitle: &'static str,
    pub connection: &'static str,
    pub settings: &'static str,
    pub help: &'static str,
    pub stopped: &'static str,
    pub waiting: &'static str,
    pub connected: &'static str,
    pub connected_plural: &'static str,
    pub headphones: &'static str,
    pub microphone: &'static str,
    pub output: &'static str,
    pub mic_destination: &'static str,
    pub refresh: &'static str,
    pub system_default: &'static str,
    pub default_suffix: &'static str,
    pub virtual_suffix: &'static str,
    pub unavailable: &'static str,
    pub start: &'static str,
    pub stop: &'static str,
    pub pending: &'static str,
    pub start_hint: &'static str,
    pub no_peers: &'static str,
    pub waiting_peer: &'static str,
    pub peer: &'static str,
    pub peers: &'static str,
    pub show_qr: &'static str,
    pub hide_qr: &'static str,
    pub qr_alt: &'static str,
    pub scan: &'static str,
    pub network: &'static str,
    pub copy: &'static str,
    pub open: &'static str,
    pub copy_title: &'static str,
    pub open_title: &'static str,
    pub dismiss: &'static str,
    pub language: &'static str,
    pub port: &'static str,
    pub stopped_settings: &'static str,
    pub autostart: &'static str,
    pub autostart_hint: &'static str,
    pub help_items: [(&'static str, &'static str); 6],
    pub refreshed: &'static str,
    pub autostart_enabled: &'static str,
    pub autostart_disabled: &'static str,
    pub copied: &'static str,
    pub invalid_port: &'static str,
    pub demo_qr: &'static str,
}

const PT: Texts = Texts {
    subtitle: "Áudio local, baixa latência",
    connection: "Conexão",
    settings: "Ajustes",
    help: "Ajuda",
    stopped: "Parado",
    waiting: "Aguardando",
    connected: "conectado",
    connected_plural: "conectados",
    headphones: "Fone",
    microphone: "Microfone",
    output: "Saída de áudio do PC",
    mic_destination: "Destino do microfone no PC",
    refresh: "Atualizar dispositivos",
    system_default: "Padrão do Windows",
    default_suffix: "padrão",
    virtual_suffix: "cabo virtual",
    unavailable: "Dispositivo indisponível — selecione outro",
    start: "Iniciar",
    stop: "Parar",
    pending: "Aguarde…",
    start_hint: "Inicie para conectar o celular.",
    no_peers: "Nenhum aparelho conectado",
    waiting_peer: "Aguardando aparelho",
    peer: "aparelho conectado",
    peers: "aparelhos conectados",
    show_qr: "Mostrar QR",
    hide_qr: "Ocultar QR",
    qr_alt: "QR code para conectar o celular",
    scan: "Leia com a câmera do iPhone.",
    network: "Endereço da rede",
    copy: "Copiar",
    open: "Abrir",
    copy_title: "Copiar endereço",
    open_title: "Abrir no navegador",
    dismiss: "OK",
    language: "Idioma",
    port: "Porta",
    stopped_settings: "Pare a conexão para alterar a porta e o áudio.",
    autostart: "Iniciar com o Windows",
    autostart_hint: "Retoma o áudio na bandeja.",
    help_items: [
        (
            "Conectar",
            "PC e celular na mesma rede. Inicie aqui, leia o QR e toque em Conectar no celular.",
        ),
        (
            "No iPhone",
            "Safari → Compartilhar → Adicionar à Tela de Início. O ícone abre o miniapp.",
        ),
        (
            "Certificado local",
            "Na primeira visita no Safari: Mostrar detalhes → visitar este site. O endereço pertence ao seu PC.",
        ),
        (
            "Microfone nos aplicativos",
            "Escolha CABLE Input aqui e CABLE Output como microfone no aplicativo. Requer VB-CABLE instalado.",
        ),
        (
            "Modo Leitor do Safari",
            "Menu da página → Ocultar Leitor. Em Ajustes do Site, desative Usar Leitor Automaticamente.",
        ),
        (
            "Bandeja",
            "Clique no coração para abrir. Clique fora ou pressione Esc para ocultar. Para encerrar, use Sair no menu da bandeja.",
        ),
    ],
    refreshed: "Dispositivos atualizados.",
    autostart_enabled: "Início com o Windows ativado.",
    autostart_disabled: "Início com o Windows desativado.",
    copied: "Endereço copiado.",
    invalid_port: "Escolha uma porta entre 1 e 65535.",
    demo_qr: "Boa mira! Você leu o QR do print. Para conectar, use o QR do HeartWire no seu PC. ♥",
};

const EN: Texts = Texts {
    subtitle: "Local audio, low latency",
    connection: "Connection",
    settings: "Settings",
    help: "Help",
    stopped: "Stopped",
    waiting: "Waiting",
    connected: "connected",
    connected_plural: "connected",
    headphones: "Headphones",
    microphone: "Microphone",
    output: "PC audio output",
    mic_destination: "Microphone destination on PC",
    refresh: "Refresh devices",
    system_default: "Windows default",
    default_suffix: "default",
    virtual_suffix: "virtual cable",
    unavailable: "Device unavailable — select another",
    start: "Start",
    stop: "Stop",
    pending: "Please wait…",
    start_hint: "Start to connect your phone.",
    no_peers: "No devices connected",
    waiting_peer: "Waiting for a device",
    peer: "device connected",
    peers: "devices connected",
    show_qr: "Show QR",
    hide_qr: "Hide QR",
    qr_alt: "QR code to connect your phone",
    scan: "Scan with your iPhone camera.",
    network: "Network address",
    copy: "Copy",
    open: "Open",
    copy_title: "Copy address",
    open_title: "Open in browser",
    dismiss: "OK",
    language: "Language",
    port: "Port",
    stopped_settings: "Stop the connection to change the port and audio.",
    autostart: "Start with Windows",
    autostart_hint: "Resumes audio in the tray.",
    help_items: [
        (
            "Connect",
            "Keep PC and phone on the same network. Start here, scan the QR, then tap Connect on your phone.",
        ),
        (
            "On iPhone",
            "Safari → Share → Add to Home Screen. The icon opens the miniapp.",
        ),
        (
            "Local certificate",
            "First Safari visit: Show Details → visit this website. The address belongs to your PC.",
        ),
        (
            "Microphone in apps",
            "Choose CABLE Input here and CABLE Output as the microphone in your app. Requires VB-CABLE installed.",
        ),
        (
            "Safari Reader",
            "Page menu → Hide Reader. In Website Settings, turn off Use Reader Automatically.",
        ),
        (
            "System tray",
            "Click the heart to open. Click outside or press Esc to hide. To quit, use Quit in the tray menu.",
        ),
    ],
    refreshed: "Devices refreshed.",
    autostart_enabled: "Start with Windows enabled.",
    autostart_disabled: "Start with Windows disabled.",
    copied: "Address copied.",
    invalid_port: "Choose a port between 1 and 65535.",
    demo_qr: "Nice shot! You scanned the screenshot QR. To connect, use the QR in HeartWire on your PC. ♥",
};

pub fn texts(language: Language) -> &'static Texts {
    match language {
        Language::Portuguese => &PT,
        Language::English => &EN,
    }
}

// Native errors are composed in Portuguese, with technical details appended.
// Translate the known prefixes and keep the details intact.
const NATIVE_ERRORS: [(&str, &str); 10] = [
    (
        "Escolha uma porta entre 1 e 65535.",
        "Choose a port between 1 and 65535.",
    ),
    (
        "Pare a transmissão antes de alterar os dispositivos ou a porta.",
        "Stop audio before changing devices or the port.",
    ),
    (
        "A ponte já está iniciada. Pare antes de trocar a configuração.",
        "Audio is already running. Stop before changing the configuration.",
    ),
    (
        "Não foi possível iniciar a ponte:",
        "Could not start audio:",
    ),
    (
        "Não foi possível iniciar o áudio:",
        "Could not start audio:",
    ),
    (
        "Não foi possível listar os dispositivos de áudio:",
        "Could not list audio devices:",
    ),
    (
        "Não foi possível salvar as preferências:",
        "Could not save preferences:",
    ),
    (
        "Não foi possível alterar a inicialização com o Windows:",
        "Could not change Windows startup:",
    ),
    ("Endereço inválido.", "Invalid address."),
    (
        "O endereço precisa ser um dos links da ponte iniciada.",
        "The address must belong to the running audio session.",
    ),
];

pub fn translate_error(language: Language, message: &str) -> String {
    match language {
        Language::Portuguese => message.to_owned(),
        Language::English => NATIVE_ERRORS
            .iter()
            .fold(message.to_owned(), |text, (source, target)| {
                text.replace(source, target)
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_keep_technical_details() {
        let message = "Não foi possível iniciar a ponte: porta 8081 em uso";
        assert_eq!(
            translate_error(Language::English, message),
            "Could not start audio: porta 8081 em uso"
        );
        assert_eq!(translate_error(Language::Portuguese, message), message);
    }
}
