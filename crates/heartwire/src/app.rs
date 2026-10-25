use std::{
    future::Future,
    sync::{
        Arc,
        mpsc::{Receiver, Sender, channel},
    },
    time::{Duration, Instant},
};

use bridge_core::{AudioDevice, BridgeConfig, BridgeMode, ServiceStatus};
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Key, Layout, Margin, RichText, Role, Stroke,
    TextureHandle, TextureOptions, Vec2, ViewportCommand, WidgetInfo,
};
use tracing_appender::non_blocking::WorkerGuard;
use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent, menu::MenuEvent};
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};

use crate::{
    controller::Core,
    i18n::{Texts, texts, translate_error},
    platform::{self, Bounds, Instance, Signal},
    preferences::{Language, Preferences},
    qr::{self, Mark},
    theme::{self, *},
    tray::{self, Tray},
    widgets::{Look, PanelButton},
};

const WIDTH: f32 = 380.0;
const MIN_HEIGHT: f32 = 350.0;
const MAX_HEIGHT: f32 = 600.0;
const POLL: Duration = Duration::from_secs(1);
const TOAST_TIME: Duration = Duration::from_millis(2800);
const QR_SIDE: f32 = 128.0;

enum Event {
    TrayClick(Bounds),
    Menu(String),
    Signal(Signal),
    Devices(Result<Vec<AudioDevice>, String>, bool),
    Started(Result<ServiceStatus, String>, bool),
    Stopped(ServiceStatus),
    Saved(Result<(), String>),
    Language(Result<Language, String>),
    Autostart(Result<bool, String>),
    Failed(Result<(), String>),
    Exit,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Connection,
    Settings,
    Help,
}

/// Blur can arrive before the corresponding icon click. Keep that click from
/// reopening the panel that the user was trying to dismiss.
#[derive(Default)]
struct Popup {
    shown: bool,
    focused: bool,
    shown_at: Option<Instant>,
    hidden_at: Option<Instant>,
    anchor: Option<Bounds>,
}

/// Closing the hidden-icons flyout takes focus right after the panel opens;
/// during this window the panel takes it back instead of hiding.
const FOCUS_GRACE: Duration = Duration::from_millis(600);

pub struct HeartWireApp {
    core: Arc<Core>,
    runtime: tokio::runtime::Runtime,
    sender: Sender<Event>,
    events: Receiver<Event>,
    ctx: egui::Context,
    window: Arc<Window>,
    tray: Option<Tray>,
    log_guard: Option<WorkerGuard>,
    popup: Popup,
    height: f32,
    preferences: Preferences,
    snapshot: ServiceStatus,
    devices: Vec<AudioDevice>,
    busy: bool,
    quitting: bool,
    tab: Tab,
    error: Option<String>,
    seen_error: Option<String>,
    toast: Option<(String, Instant)>,
    port: String,
    network: String,
    qr_expanded: bool,
    qr: Option<(String, TextureHandle)>,
    previous_running: bool,
    previous_peers: usize,
    mark: Option<Mark>,
    brand: TextureHandle,
    last_poll: Instant,
    demo: bool,
}

/// What the process was launched to do.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Launch {
    /// Tray only; the panel opens from the icon.
    Normal,
    /// Start audio with the saved settings (start with Windows).
    Minimized,
    /// Sample data and a demonstration QR for documentation screenshots.
    /// No audio starts and nothing is saved.
    Demo,
}

impl HeartWireApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        core: Arc<Core>,
        runtime: tokio::runtime::Runtime,
        instance: Instance,
        log_guard: WorkerGuard,
        preferences: Preferences,
        launch: Launch,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ctx = cc.egui_ctx.clone();
        theme::apply(&ctx);
        let window = cc.winit_window().cloned().ok_or("no native window")?;
        if let Ok(handle) = window.window_handle()
            && let RawWindowHandle::Win32(handle) = handle.as_raw()
        {
            platform::round_corners(handle.hwnd.get());
        }

        let (sender, events) = channel();
        {
            let (sender, ctx) = (sender.clone(), ctx.clone());
            TrayIconEvent::set_event_handler(Some(move |event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    rect,
                    ..
                } = event
                {
                    let left = rect.position.x as i32;
                    let top = rect.position.y as i32;
                    let _ = sender.send(Event::TrayClick(Bounds {
                        left,
                        top,
                        right: left + rect.size.width as i32,
                        bottom: top + rect.size.height as i32,
                    }));
                    ctx.request_repaint();
                }
            }));
        }
        {
            let (sender, ctx) = (sender.clone(), ctx.clone());
            MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
                let _ = sender.send(Event::Menu(event.id.0));
                ctx.request_repaint();
            }));
        }
        {
            let (sender, ctx) = (sender.clone(), ctx.clone());
            instance.listen(move |signal| {
                let _ = sender.send(Event::Signal(signal));
                ctx.request_repaint();
            });
        }

        let brand = ctx.load_texture(
            "brand",
            decode_png(include_bytes!("../icons/128x128.png")),
            TextureOptions::NEAREST,
        );
        let snapshot = core.try_snapshot().unwrap_or(ServiceStatus {
            running: false,
            urls: Vec::new(),
            connected_peers: 0,
            mode: preferences.mode,
            port: preferences.port,
            last_error: None,
        });
        let mut app = Self {
            port: preferences.port.to_string(),
            core,
            runtime,
            sender,
            events,
            ctx,
            window,
            tray: Some(Tray::new()?),
            log_guard: Some(log_guard),
            popup: Popup::default(),
            height: 0.0,
            preferences,
            snapshot,
            devices: Vec::new(),
            busy: false,
            quitting: false,
            tab: Tab::Connection,
            error: None,
            seen_error: None,
            toast: None,
            network: String::new(),
            qr_expanded: false,
            qr: None,
            previous_running: false,
            previous_peers: 0,
            mark: Mark::decode(include_bytes!("../icons/128x128.png")),
            brand,
            last_poll: Instant::now(),
            demo: launch == Launch::Demo,
        };
        match launch {
            Launch::Demo => {
                app.apply_snapshot(ServiceStatus {
                    running: true,
                    urls: vec!["https://192.168.0.10:8081".into()],
                    connected_peers: 1,
                    mode: BridgeMode::Listen,
                    port: 8081,
                    last_error: None,
                });
                app.qr_expanded = true;
                app.show(None);
            }
            Launch::Minimized => {
                app.load_devices(false);
                let core = app.core.clone();
                app.spawn(async move { Event::Started(core.start_saved().await, true) });
            }
            Launch::Normal => app.load_devices(false),
        }
        Ok(app)
    }

    fn t(&self) -> &'static Texts {
        texts(self.preferences.language)
    }

    fn spawn(&self, task: impl Future<Output = Event> + Send + 'static) {
        if self.demo {
            return;
        }
        let (sender, ctx) = (self.sender.clone(), self.ctx.clone());
        self.runtime.spawn(async move {
            let _ = sender.send(task.await);
            ctx.request_repaint();
        });
    }

    // ----- Popup window -------------------------------------------------

    fn show(&mut self, anchor: Option<Bounds>) {
        if anchor.is_some() {
            self.popup.anchor = anchor;
        }
        self.place();
        self.window.set_visible(true);
        self.window.focus_window();
        self.popup.shown = true;
        self.popup.focused = false;
        self.popup.shown_at = Some(Instant::now());
        self.ctx.request_repaint();
    }

    fn hide(&mut self) {
        if self.popup.shown {
            self.popup.hidden_at = Some(Instant::now());
        }
        self.popup.shown = false;
        self.popup.focused = false;
        self.window.set_visible(false);
    }

    fn toggle(&mut self, anchor: Bounds) {
        let just_hidden = self
            .popup
            .hidden_at
            .is_some_and(|instant| instant.elapsed() < Duration::from_millis(300));
        if self.popup.shown || just_hidden {
            self.hide();
        } else {
            self.show(Some(anchor));
        }
    }

    fn place(&self) {
        let size = self.window.outer_size();
        let point = self
            .popup
            .anchor
            .map_or((0, 0), |icon| (icon.left, icon.top));
        if let Some(screen) = platform::screen_at(point.0, point.1) {
            let (x, y) = platform::popup_position(
                self.popup.anchor,
                size.width as i32,
                size.height as i32,
                screen,
            );
            self.window.set_outer_position(PhysicalPosition::new(x, y));
        }
    }

    /// Follow the content height, like the original panel, and keep the
    /// panel anchored to the tray while it grows or shrinks.
    fn fit(&mut self, content: f32) {
        let height = content.ceil().clamp(MIN_HEIGHT, MAX_HEIGHT);
        if (height - self.height).abs() < 1.0 {
            return;
        }
        self.height = height;
        let _ = self
            .window
            .request_inner_size(LogicalSize::new(WIDTH, height));
        if self.popup.shown {
            self.place();
        }
    }

    // ----- State --------------------------------------------------------

    fn apply_snapshot(&mut self, snapshot: ServiceStatus) {
        if snapshot.last_error.is_some() && snapshot.last_error != self.seen_error {
            self.error.clone_from(&snapshot.last_error);
        }
        self.seen_error.clone_from(&snapshot.last_error);
        if snapshot.running && self.preferences.mode != snapshot.mode {
            self.preferences.mode = snapshot.mode;
        }
        let count = if snapshot.running {
            snapshot.connected_peers
        } else {
            0
        };
        // Show the QR while waiting for the first device; tuck it away once
        // one connects and bring it back when the last one leaves.
        if !snapshot.running {
            self.qr_expanded = false;
        } else if !self.previous_running {
            self.qr_expanded = count == 0;
        } else if self.previous_peers == 0 && count > 0 {
            self.qr_expanded = false;
        } else if self.previous_peers > 0 && count == 0 {
            self.qr_expanded = true;
        }
        self.previous_running = snapshot.running;
        self.previous_peers = count;
        if !snapshot.urls.contains(&self.network) {
            self.network = snapshot.urls.first().cloned().unwrap_or_default();
        }
        self.snapshot = snapshot;
    }

    fn handle(&mut self, event: Event) {
        match event {
            Event::TrayClick(anchor) => self.toggle(anchor),
            Event::Menu(id) => match id.as_str() {
                tray::SHOW => self.show(None),
                tray::START => {
                    let core = self.core.clone();
                    self.busy = true;
                    self.spawn(async move { Event::Started(core.start_saved().await, true) });
                }
                tray::STOP => self.stop(),
                tray::QUIT => self.quit(),
                _ => {}
            },
            Event::Signal(Signal::Show) => self.show(None),
            Event::Signal(Signal::Quit) => self.quit(),
            Event::Devices(result, announce) => {
                if announce {
                    self.busy = false;
                }
                match result {
                    Ok(devices) => {
                        self.devices = devices;
                        if announce {
                            self.notify(self.t().refreshed);
                        }
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Event::Started(result, from_tray) => {
                self.busy = false;
                match result {
                    Ok(snapshot) => self.apply_snapshot(snapshot),
                    Err(error) => {
                        self.error = Some(error);
                        if from_tray {
                            self.show(None);
                        }
                    }
                }
            }
            Event::Stopped(snapshot) => {
                self.busy = false;
                self.apply_snapshot(snapshot);
            }
            Event::Language(result) => {
                self.busy = false;
                match result {
                    Ok(language) => self.preferences.language = language,
                    Err(error) => self.error = Some(error),
                }
            }
            Event::Autostart(result) => {
                self.busy = false;
                match result {
                    Ok(enabled) => {
                        self.preferences.autostart = enabled;
                        let t = self.t();
                        self.notify(if enabled {
                            t.autostart_enabled
                        } else {
                            t.autostart_disabled
                        });
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Event::Saved(result) | Event::Failed(result) => {
                if let Err(error) = result {
                    self.error = Some(error);
                }
            }
            Event::Exit => {
                // Remove the icon now: a killed process leaves a ghost in the tray.
                self.tray.take();
                self.log_guard.take();
                std::process::exit(0);
            }
        }
    }

    fn notify(&mut self, message: &str) {
        self.toast = Some((message.to_owned(), Instant::now()));
    }

    fn load_devices(&mut self, announce: bool) {
        if announce {
            self.busy = true;
        }
        self.spawn(async move {
            let devices = tokio::task::spawn_blocking(bridge_core::list_output_devices)
                .await
                .map_err(|error| error.to_string())
                .and_then(|result| {
                    result.map_err(|error| {
                        format!("Não foi possível listar os dispositivos de áudio: {error:#}")
                    })
                });
            Event::Devices(devices, announce)
        });
    }

    fn config(&mut self) -> Option<BridgeConfig> {
        match self.port.trim().parse::<u16>() {
            Ok(port) if port > 0 => {
                self.preferences.port = port;
                Some(self.preferences.config())
            }
            _ => {
                self.error = Some(self.t().invalid_port.to_owned());
                None
            }
        }
    }

    fn save(&mut self) {
        if self.config().is_none() {
            return;
        }
        let (core, preferences) = (self.core.clone(), self.preferences.clone());
        self.spawn(async move { Event::Saved(core.save_preferences(preferences).await) });
    }

    fn start(&mut self) {
        let Some(config) = self.config() else { return };
        self.busy = true;
        self.error = None;
        let core = self.core.clone();
        self.spawn(async move { Event::Started(core.start(config).await, false) });
    }

    fn stop(&mut self) {
        self.busy = true;
        self.error = None;
        let core = self.core.clone();
        self.spawn(async move { Event::Stopped(core.stop().await) });
    }

    fn quit(&mut self) {
        if std::mem::replace(&mut self.quitting, true) {
            return;
        }
        self.hide();
        if self.demo {
            // Nothing runs in demo mode, so there is no audio to stop.
            self.handle(Event::Exit);
        }
        let core = self.core.clone();
        self.spawn(async move {
            core.stop().await;
            Event::Exit
        });
    }

    fn choose_mode(&mut self, mode: BridgeMode) {
        if self.locked() || self.preferences.mode == mode {
            return;
        }
        self.preferences.mode = mode;
        if mode == BridgeMode::Mic && self.preferences.mic_device_id.is_none() {
            self.preferences.mic_device_id = self
                .devices
                .iter()
                .find(|device| device.name.to_lowercase().contains("cable input"))
                .map(|device| device.id.clone());
        }
        self.save();
    }

    fn locked(&self) -> bool {
        self.snapshot.running || self.busy
    }

    fn selected_url(&self) -> Option<String> {
        if !self.snapshot.running {
            return None;
        }
        if self.demo {
            return Some(self.t().demo_qr.to_owned());
        }
        let mut url = url::Url::parse(&self.network).ok()?;
        url.query_pairs_mut()
            .append_pair(
                "mode",
                match self.snapshot.mode {
                    BridgeMode::Listen => "listen",
                    BridgeMode::Mic => "mic",
                },
            )
            .append_pair(
                "lang",
                match self.preferences.language {
                    Language::Portuguese => "pt-BR",
                    Language::English => "en",
                },
            );
        Some(url.to_string())
    }

    fn qr_texture(&mut self, url: &str) -> Option<TextureHandle> {
        if let Some((rendered, texture)) = &self.qr
            && rendered == url
        {
            return Some(texture.clone());
        }
        let pixels = (QR_SIDE * self.ctx.pixels_per_point()).ceil() as usize;
        match qr::pairing_qr(url, pixels, self.mark.as_ref()) {
            Ok(image) => {
                // Rendered at or above the display size; the slight
                // downscale to the fixed 128 pt square stays sharp.
                let texture = self.ctx.load_texture("qr", image, TextureOptions::LINEAR);
                self.qr = Some((url.to_owned(), texture.clone()));
                Some(texture)
            }
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }

    // ----- Drawing ------------------------------------------------------

    fn header(&self, ui: &mut egui::Ui) {
        let t = self.t();
        ui.horizontal(|ui| {
            ui.set_min_height(68.0);
            ui.add(egui::Image::new((self.brand.id(), Vec2::splat(32.0))));
            ui.vertical(|ui| {
                ui.add_space(14.0);
                ui.label(RichText::new("HeartWire").font(semibold(18.0)).color(TEXT));
                ui.label(RichText::new(t.subtitle).size(10.5).color(MUTED));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let count = if self.snapshot.running {
                    self.snapshot.connected_peers
                } else {
                    0
                };
                let label = if count > 0 {
                    format!(
                        "{count} {}",
                        if count == 1 {
                            t.connected
                        } else {
                            t.connected_plural
                        }
                    )
                } else if self.snapshot.running {
                    t.waiting.to_owned()
                } else {
                    t.stopped.to_owned()
                };
                let (fill, text, dot) = if count > 0 {
                    (PILL_CONNECTED, PILL_CONNECTED_TEXT, DOT_CONNECTED)
                } else if self.snapshot.running {
                    (Color32::TRANSPARENT, PILL_RUNNING_TEXT, GREEN)
                } else {
                    (Color32::TRANSPARENT, MUTED, DOT_IDLE)
                };
                egui::Frame::new()
                    .fill(fill)
                    .corner_radius(7)
                    .inner_margin(Margin::symmetric(7, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 5.0;
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(5.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 2.5, dot);
                            ui.label(RichText::new(label).font(semibold(11.0)).color(text));
                        });
                    });
            });
        });
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        let t = self.t();
        egui::Frame::new()
            .fill(INSET)
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x2e, 0x37, 0x30)))
            .corner_radius(10)
            .inner_margin(3)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    let width = (ui.available_width() - 6.0) / 3.0;
                    for (tab, label) in [
                        (Tab::Connection, t.connection),
                        (Tab::Settings, t.settings),
                        (Tab::Help, t.help),
                    ] {
                        let selected = self.tab == tab;
                        let look = if selected {
                            Look::filled(TAB_SELECTED, TAB_SELECTED, TAB_SELECTED_LINE, TEXT)
                        } else {
                            Look::plain(MUTED, TEXT, Color32::TRANSPARENT)
                        };
                        let response = ui.add(
                            PanelButton::new(label, look)
                                .size(width, 30.0)
                                .selected(Role::Tab, selected),
                        );
                        if response.clicked() {
                            self.tab = tab;
                        }
                    }
                });
            });
    }

    fn notice(&mut self, ui: &mut egui::Ui) {
        let Some(error) = &self.error else { return };
        let message = translate_error(self.preferences.language, error);
        let mut dismiss = false;
        egui::Frame::new()
            .fill(ERROR)
            .stroke(Stroke::new(1.0, ERROR_LINE))
            .corner_radius(8)
            .inner_margin(Margin::symmetric(10, 9))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let button = ui.add(
                        PanelButton::new(
                            self.t().dismiss,
                            Look::plain(ERROR_TEXT, Color32::WHITE, HOVER_TINT),
                        )
                        .font(semibold(11.0))
                        .size(0.0, 22.0)
                        .radius(5),
                    );
                    dismiss = button.clicked();
                    ui.add(
                        egui::Label::new(RichText::new(message).size(12.0).color(ERROR_TEXT))
                            .wrap(),
                    );
                });
            });
        ui.add_space(11.0);
        if dismiss {
            self.error = None;
        }
    }

    fn connection(&mut self, ui: &mut egui::Ui) {
        let t = self.t();
        let locked = self.locked();
        let mic = self.preferences.mode == BridgeMode::Mic;

        egui::Frame::new()
            .fill(INSET)
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(10)
            .inner_margin(3)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    let width = (ui.available_width() - 3.0) / 2.0;
                    for (mode, glyph, label) in [
                        (BridgeMode::Listen, HEADPHONES, t.headphones),
                        (BridgeMode::Mic, MICROPHONE, t.microphone),
                    ] {
                        let active = self.preferences.mode == mode;
                        let look = if active {
                            Look::filled(
                                MODE_ACTIVE,
                                MODE_ACTIVE,
                                MODE_ACTIVE_LINE,
                                MODE_ACTIVE_TEXT,
                            )
                        } else {
                            Look::plain(MUTED, TEXT, HOVER_TINT)
                        };
                        let response = ui.add_enabled(
                            !locked,
                            PanelButton::new(label, look)
                                .icon(Some(glyph))
                                .size(width, 31.0)
                                .radius(6)
                                .selected(Role::Button, active),
                        );
                        if response.clicked() {
                            self.choose_mode(mode);
                        }
                    }
                });
            });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(if mic { t.mic_destination } else { t.output })
                    .size(12.0)
                    .color(LABEL),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Icon-only: the name is announced and shown as a tooltip.
                let refresh = ui
                    .add_enabled(
                        !locked,
                        PanelButton::new("", Look::plain(MUTED, Color32::WHITE, HOVER_TINT))
                            .icon(Some(REFRESH))
                            .size(24.0, 24.0)
                            .radius(5),
                    )
                    .on_hover_text(t.refresh);
                refresh.widget_info(|| WidgetInfo::labeled(Role::Button, !locked, t.refresh));
                if refresh.clicked() {
                    self.load_devices(true);
                }
            });
        });

        let selected = if mic {
            self.preferences.mic_device_id.clone()
        } else {
            self.preferences.capture_device_id.clone()
        };
        let label_of = |id: &Option<String>| match id {
            None => t.system_default.to_owned(),
            Some(id) => self
                .devices
                .iter()
                .find(|device| &device.id == id)
                .map_or_else(
                    || t.unavailable.to_owned(),
                    |device| device_label(t, device),
                ),
        };
        let mut choice = selected.clone();
        ui.add_enabled_ui(!locked, |ui| {
            egui::ComboBox::from_id_salt("device")
                .width(ui.available_width())
                .height(260.0)
                .selected_text(label_of(&selected))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut choice, None, t.system_default);
                    for device in &self.devices {
                        ui.selectable_value(
                            &mut choice,
                            Some(device.id.clone()),
                            device_label(t, device),
                        );
                    }
                });
        });
        if choice != selected {
            if mic {
                self.preferences.mic_device_id = choice;
            } else {
                self.preferences.capture_device_id = choice;
            }
            self.save();
        }

        ui.add_space(4.0);
        let running = self.snapshot.running;
        let look = if running {
            Look::filled(RUNNING, RUNNING_HOVER, RUNNING_LINE, RUNNING_TEXT)
        } else {
            Look::filled(PRIMARY, PRIMARY_HOVER, PRIMARY_LINE, PRIMARY_TEXT)
        };
        let (glyph, caption) = if self.busy {
            (None, t.pending)
        } else if running {
            (Some(STOP), t.stop)
        } else {
            (Some(PLAY), t.start)
        };
        let toggle = ui.add_enabled(
            !self.busy,
            PanelButton::new(caption, look)
                .icon(glyph)
                .font(semibold(13.0))
                .size(ui.available_width(), 37.0)
                .radius(9),
        );
        if toggle.clicked() {
            if running {
                self.stop();
            } else {
                self.start();
            }
        }
        if !running {
            ui.label(RichText::new(t.start_hint).size(11.0).color(MUTED));
        }

        let count = if running {
            self.snapshot.connected_peers
        } else {
            0
        };
        let (fill, line, color, font) = if count > 0 {
            (
                PEER_CONNECTED,
                PEER_CONNECTED_LINE,
                PEER_CONNECTED_TEXT,
                semibold(15.0),
            )
        } else {
            (FIELD, FIELD_LINE, MUTED, egui::FontId::proportional(13.0))
        };
        let peers = if count > 0 {
            format!("{count} {}", if count == 1 { t.peer } else { t.peers })
        } else if running {
            t.waiting_peer.to_owned()
        } else {
            t.no_peers.to_owned()
        };
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(fill)
            .stroke(Stroke::new(1.0, line))
            .corner_radius(9)
            .inner_margin(11)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(icon(if count > 0 { CHECK } else { PHONE }, 18.0, color));
                    ui.label(RichText::new(peers).font(font).color(color));
                });
            });

        if running && !self.snapshot.urls.is_empty() {
            ui.add_space(2.0);
            let expanded = self.qr_expanded;
            if secondary(
                ui,
                None,
                if expanded { t.hide_qr } else { t.show_qr },
                ui.available_width(),
            )
            .clicked()
            {
                self.qr_expanded = !expanded;
            }
            if self.qr_expanded {
                self.pairing(ui);
            }
        }
    }

    fn pairing(&mut self, ui: &mut egui::Ui) {
        let t = self.t();
        let Some(url) = self.selected_url() else {
            return;
        };
        ui.add_space(4.0);
        ui.separator();
        ui.horizontal(|ui| {
            if let Some(texture) = self.qr_texture(&url) {
                let size = Vec2::splat(QR_SIDE);
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .corner_radius(8)
                    .show(ui, |ui| {
                        let image = ui.add(egui::Image::new((texture.id(), size)));
                        image.widget_info(|| WidgetInfo::labeled(Role::Image, true, t.qr_alt));
                    });
            }
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(t.scan)
                        .size(11.0)
                        .color(Color32::from_rgb(0xc1, 0xce, 0xc5)),
                );
                if self.snapshot.urls.len() > 1 {
                    let mut network = self.network.clone();
                    egui::ComboBox::from_id_salt("network")
                        .width(ui.available_width())
                        .selected_text(host(&network))
                        .show_ui(ui, |ui| {
                            for candidate in &self.snapshot.urls {
                                ui.selectable_value(
                                    &mut network,
                                    candidate.clone(),
                                    host(candidate),
                                );
                            }
                        })
                        .response
                        .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, t.network));
                    self.network = network;
                } else {
                    ui.add(
                        egui::Label::new(
                            RichText::new(host(&self.network))
                                .monospace()
                                .color(Color32::from_rgb(0xbe, 0xcf, 0xc3)),
                        )
                        .selectable(true),
                    )
                    .on_hover_text(&url);
                }
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    let width = (ui.available_width() - 5.0) / 2.0;
                    if secondary(ui, Some(COPY), t.copy, width)
                        .on_hover_text(t.copy_title)
                        .clicked()
                    {
                        self.ctx.copy_text(url.clone());
                        self.notify(t.copied);
                    }
                    if secondary(ui, Some(OPEN), t.open, width)
                        .on_hover_text(t.open_title)
                        .clicked()
                    {
                        let (core, url) = (self.core.clone(), url.clone());
                        self.spawn(async move { Event::Failed(core.open_link(&url).await) });
                    }
                });
            });
        });
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        let t = self.t();
        ui.label(RichText::new(t.language).size(12.0).color(LABEL));
        let mut language = self.preferences.language;
        ui.add_enabled_ui(!self.busy, |ui| {
            egui::ComboBox::from_id_salt("language")
                .width(ui.available_width())
                .selected_text(language_name(language))
                .show_ui(ui, |ui| {
                    for option in [Language::Portuguese, Language::English] {
                        ui.selectable_value(&mut language, option, language_name(option));
                    }
                });
        });
        if language != self.preferences.language {
            self.busy = true;
            let core = self.core.clone();
            self.spawn(async move {
                Event::Language(core.set_language(language).await.map(|()| language))
            });
        }

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(t.port).color(Color32::from_rgb(0xd1, 0xdc, 0xd4)));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let field = ui.add_enabled(
                    !self.locked(),
                    egui::TextEdit::singleline(&mut self.port)
                        .desired_width(94.0)
                        .char_limit(5)
                        .margin(Margin::symmetric(9, 7)),
                );
                field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, t.port));
                self.port.retain(|c| c.is_ascii_digit());
                if field.lost_focus() && self.port != self.preferences.port.to_string() {
                    self.save();
                }
            });
        });
        ui.label(RichText::new(t.stopped_settings).size(11.0).color(MUTED));
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        let mut autostart = self.preferences.autostart;
        let response = ui.add_enabled(
            !self.busy,
            egui::Checkbox::new(&mut autostart, RichText::new(t.autostart).color(TEXT)),
        );
        ui.label(RichText::new(t.autostart_hint).size(11.0).color(MUTED));
        if response.changed() {
            self.busy = true;
            let core = self.core.clone();
            self.spawn(async move {
                Event::Autostart(core.set_autostart(autostart).await.map(|()| autostart))
            });
        }
    }

    fn help(&self, ui: &mut egui::Ui) {
        for (title, body) in self.t().help_items {
            ui.label(
                RichText::new(title)
                    .font(semibold(12.0))
                    .color(Color32::from_rgb(0xdb, 0xe6, 0xdf)),
            );
            ui.label(RichText::new(body).size(11.5).color(MUTED));
            ui.add_space(5.0);
        }
        ui.separator();
        ui.label(
            RichText::new("HeartWire · saitofernando · MIT")
                .size(10.0)
                .color(Color32::from_rgb(0x85, 0x95, 0x8a)),
        );
    }

    fn toast(&mut self, ctx: &egui::Context) {
        let Some((message, since)) = &self.toast else {
            return;
        };
        let remaining = TOAST_TIME.saturating_sub(since.elapsed());
        if remaining.is_zero() {
            self.toast = None;
            return;
        }
        ctx.request_repaint_after(remaining);
        egui::Area::new(egui::Id::unique("toast"))
            .anchor(Align2::CENTER_BOTTOM, [0.0, -13.0])
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(TOAST)
                    .stroke(Stroke::new(1.0, TOAST_LINE))
                    .corner_radius(9)
                    .inner_margin(Margin::symmetric(12, 9))
                    .show(ui, |ui| {
                        ui.label(RichText::new(message.as_str()).size(12.0).color(TOAST_TEXT));
                    });
            });
    }
}

impl eframe::App for HeartWireApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = self.events.try_recv() {
            self.handle(event);
        }
        if self.last_poll.elapsed() >= POLL {
            self.last_poll = Instant::now();
            if !self.busy
                && !self.demo
                && let Some(snapshot) = self.core.try_snapshot()
            {
                self.apply_snapshot(snapshot);
            }
        }
        if let Some(tray) = &mut self.tray {
            tray.update(&self.snapshot, self.preferences.language);
        }
        ctx.request_repaint_after(POLL.saturating_sub(self.last_poll.elapsed()));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let (focused, close, escape) = ctx.input(|input| {
            (
                input.viewport().focused,
                input.viewport().close_requested(),
                input.key_pressed(Key::Escape),
            )
        });
        if close {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        if self.popup.shown {
            let settling = self
                .popup
                .shown_at
                .is_some_and(|instant| instant.elapsed() < FOCUS_GRACE);
            match focused {
                Some(true) => self.popup.focused = true,
                Some(false) if settling => {
                    self.window.focus_window();
                    ctx.request_repaint_after(Duration::from_millis(100));
                }
                Some(false) if self.popup.focused => self.hide(),
                _ => {}
            }
        }
        let popup_open = egui::Popup::is_any_open(&ctx);
        if close || (escape && !popup_open) {
            self.hide();
        }

        ui.painter()
            .rect_filled(ui.max_rect(), CornerRadius::ZERO, BACKGROUND);
        let top = ui.max_rect().top();
        let mut bottom = top;
        egui::Frame::new()
            .inner_margin(Margin::symmetric(16, 0))
            .show(ui, |ui| {
                self.header(ui);
                self.tabs(ui);
                let output = egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.add_space(13.0);
                        self.notice(ui);
                        match self.tab {
                            Tab::Connection => self.connection(ui),
                            Tab::Settings => self.settings(ui),
                            Tab::Help => self.help(ui),
                        }
                        ui.add_space(16.0);
                    });
                bottom = output.inner_rect.top() + output.content_size.y;
            });
        self.fit(bottom - top);
        self.toast(&ctx);
    }
}

fn secondary(ui: &mut egui::Ui, glyph: Option<char>, label: &str, width: f32) -> egui::Response {
    ui.add(
        PanelButton::new(
            label,
            Look::filled(SECONDARY, SECONDARY_HOVER, SECONDARY_LINE, SECONDARY_TEXT),
        )
        .icon(glyph)
        .font(egui::FontId::proportional(11.0))
        .size(width, 29.0),
    )
}

fn device_label(t: &Texts, device: &AudioDevice) -> String {
    if device.is_virtual {
        format!("{} · {}", device.name, t.virtual_suffix)
    } else if device.is_default {
        format!("{} · {}", device.name, t.default_suffix)
    } else {
        device.name.clone()
    }
}

fn host(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|url| {
            let host = url.host_str()?.to_owned();
            Some(match url.port() {
                Some(port) => format!("{host}:{port}"),
                None => host,
            })
        })
        .unwrap_or_default()
}

fn language_name(language: Language) -> &'static str {
    match language {
        Language::Portuguese => "Português (Brasil)",
        Language::English => "English",
    }
}

fn decode_png(bytes: &[u8]) -> egui::ColorImage {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .expect("embedded PNG")
        .into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    egui::ColorImage::from_rgba_unmultiplied(size, &image.into_raw())
}
