use bridge_core::ServiceStatus;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

use crate::preferences::Language;

pub const SHOW: &str = "show";
pub const START: &str = "start";
pub const STOP: &str = "stop";
pub const QUIT: &str = "quit";

/// Tray icon and its menu. Lives on the UI thread, which pumps its messages.
pub struct Tray {
    icon: TrayIcon,
    show: MenuItem,
    status: MenuItem,
    start: MenuItem,
    stop: MenuItem,
    quit: MenuItem,
    shown: String,
}

impl Tray {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let show = MenuItem::with_id(SHOW, "Abrir HeartWire", true, None);
        let status = MenuItem::with_id("status", "Parado · 0 aparelhos", false, None);
        let start = MenuItem::with_id(START, "Iniciar", true, None);
        let stop = MenuItem::with_id(STOP, "Parar", false, None);
        let quit = MenuItem::with_id(QUIT, "Sair", true, None);
        let menu = Menu::with_items(&[&show, &status, &start, &stop, &quit])?;
        let image = image::load_from_memory_with_format(
            include_bytes!("../icons/tray.png"),
            image::ImageFormat::Png,
        )?
        .into_rgba8();
        let (width, height) = image.dimensions();
        let icon = TrayIconBuilder::new()
            .with_id("heartwire")
            .with_icon(Icon::from_rgba(image.into_raw(), width, height)?)
            .with_tooltip("HeartWire")
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .build()?;
        Ok(Self {
            icon,
            show,
            status,
            start,
            stop,
            quit,
            shown: String::new(),
        })
    }

    pub fn update(&mut self, snapshot: &ServiceStatus, language: Language) {
        let label = if snapshot.running {
            language.text("Transmitindo", "Streaming")
        } else {
            language.text("Parado", "Stopped")
        };
        let count = snapshot.connected_peers;
        let unit = if count == 1 {
            language.text("aparelho", "device")
        } else {
            language.text("aparelhos", "devices")
        };
        let text = format!("{label} · {count} {unit}");
        // Avoid a shell tray refresh every second: some Windows versions
        // activate the taskbar on refresh and dismiss transient windows.
        let key = format!("{text}:{language:?}");
        if key == self.shown {
            return;
        }
        self.status.set_text(&text);
        self.start.set_enabled(!snapshot.running);
        self.stop.set_enabled(snapshot.running);
        self.show
            .set_text(language.text("Abrir HeartWire", "Open HeartWire"));
        self.start.set_text(language.text("Iniciar", "Start"));
        self.stop.set_text(language.text("Parar", "Stop"));
        self.quit.set_text(language.text("Sair", "Quit"));
        let _ = self.icon.set_tooltip(Some(format!("HeartWire · {text}")));
        self.shown = key;
    }
}
