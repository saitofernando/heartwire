#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, path::Path, sync::Arc};

use eframe::egui;

mod app;
mod controller;
mod i18n;
mod platform;
mod preferences;
mod qr;
mod theme;
mod tray;
mod widgets;

use app::Launch;
use controller::Core;
use platform::{Instance, Signal};
use preferences::Preferences;

fn main() -> eframe::Result {
    let flag = |name: &str| std::env::args().any(|argument| argument == name);
    let quit = flag("--quit");
    let launch = if flag("--demo") {
        Launch::Demo
    } else if flag("--minimized") {
        Launch::Minimized
    } else {
        Launch::Normal
    };
    // `--quit` (used by the installer) asks a running HeartWire to stop audio
    // and exit; it never starts a new one.
    let signal = if quit { Signal::Quit } else { Signal::Show };
    let Some(instance) = Instance::acquire(signal) else {
        return Ok(());
    };
    if quit {
        return Ok(());
    }

    let data_dir = platform::data_dir();
    let _ = fs::create_dir_all(&data_dir);
    let writer = tracing_appender::rolling::daily(data_dir.join("logs"), "heartwire.log");
    let (writer, log_guard) = tracing_appender::non_blocking(writer);
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bridge_core=info,heartwire=info,warn".into()),
        )
        .with_ansi(false)
        .with_writer(writer)
        .try_init();

    let mut preferences = read_preferences(&data_dir);
    preferences.autostart = platform::autostart_enabled();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("heartwire")
        .build()
        .expect("Erro ao iniciar HeartWire");
    let core = Arc::new(Core::new(preferences.clone(), data_dir));

    // OpenGL measured lighter than DX12 or WARP for this small panel.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("HeartWire")
            .with_inner_size([380.0, 570.0])
            .with_decorations(false)
            .with_resizable(false)
            .with_taskbar(false)
            .with_always_on_top()
            .with_visible(false)
            .with_icon(window_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "HeartWire",
        options,
        Box::new(move |cc| {
            Ok(Box::new(app::HeartWireApp::new(
                cc,
                core,
                runtime,
                instance,
                log_guard,
                preferences,
                launch,
            )?))
        }),
    )
}

fn read_preferences(data_dir: &Path) -> Preferences {
    fs::read(data_dir.join("preferences.json"))
        .ok()
        .and_then(
            |bytes| match serde_json::from_slice::<Preferences>(&bytes) {
                Ok(preferences) if preferences.port > 0 => Some(preferences),
                _ => {
                    tracing::warn!("Preferências inválidas; usando padrão");
                    None
                }
            },
        )
        .unwrap_or_default()
}

fn window_icon() -> egui::IconData {
    let image = image::load_from_memory_with_format(
        include_bytes!("../icons/32x32.png"),
        image::ImageFormat::Png,
    )
    .expect("embedded PNG")
    .into_rgba8();
    egui::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    }
}
