//! Private JSON-lines transport. This process owns audio; Qt owns every desktop surface.
use bridge_core::{BridgeConfig, ServiceStatus, list_output_devices};
use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Write},
    sync::Arc,
};
mod controller;
#[path = "backend_platform.rs"]
mod platform;
mod preferences;
use controller::Core;
use preferences::Preferences;

fn qr(url: &str) -> String {
    let Ok(code) = qrcode::QrCode::new(url.as_bytes()) else {
        return String::new();
    };
    let width = code.width();
    let mut path = String::new();
    for y in 0..width {
        for x in 0..width {
            if code[(x, y)] == qrcode::Color::Dark {
                path.push_str(&format!("M{} {}h1v1h-1z", x + 4, y + 4));
            }
        }
    }
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {0} {0}\"><path fill=\"white\" d=\"M0 0h{0}v{0}H0z\"/><path fill=\"black\" d=\"{path}\"/></svg>",
        width + 8
    )
}
fn output(value: Value) {
    let mut out = io::stdout().lock();
    let _ = writeln!(out, "{value}");
    let _ = out.flush();
}
fn message(error: impl ToString) {
    output(json!({"event":"message","text":error.to_string(),"error":true}));
}
fn snapshot(status: ServiceStatus, prefs: &Preferences, devices: &Value) -> Value {
    let url = status
        .urls
        .first()
        .map(|url| {
            format!(
                "{}/?mode={}&lang={}",
                url.trim_end_matches('/'),
                if status.mode == bridge_core::BridgeMode::Mic {
                    "mic"
                } else {
                    "listen"
                },
                if prefs.language == preferences::Language::English {
                    "en"
                } else {
                    "pt-BR"
                }
            )
        })
        .unwrap_or_default();
    json!({"status":status,"preferences":prefs,"devices":devices,"link":url,"qr":if url.is_empty(){String::new()}else{qr(&url)}})
}
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = platform::data_dir();
    std::fs::create_dir_all(&data)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o700))?;
    }
    let _ = tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter("warn")
        .try_init();
    let mut prefs: Preferences = std::fs::read(data.join("preferences.json"))
        .ok()
        .and_then(|x| serde_json::from_slice(&x).ok())
        .unwrap_or_default();
    prefs.autostart = platform::autostart_enabled();
    let core = Arc::new(Core::new(prefs.clone(), data));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Value>(32);
    std::thread::spawn(move || {
        for line in io::stdin().lock().lines().map_while(Result::ok) {
            if line.len() > 65536 {
                continue;
            }
            match serde_json::from_str(&line) {
                Ok(v) => {
                    if tx.blocking_send(v).is_err() {
                        break;
                    }
                }
                Err(_) => message("Invalid command"),
            }
        }
    });
    let mut devices = Value::Array(Vec::new());
    match tokio::task::spawn_blocking(list_output_devices).await? {
        Ok(list) => devices = serde_json::to_value(list)?,
        Err(e) => message(format!("Cannot list audio devices: {e}")),
    }
    let mut last = Value::Null;
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(2));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    loop {
        tokio::select! {
            _=terminate.recv()=>{core.stop().await;break;},
            _=tick.tick()=>{},
            request=rx.recv()=> {
                let Some(request)=request else{core.stop().await;break};
                let arg=&request["args"];
                match request["command"].as_str().unwrap_or("") {
                    "quit"=>{core.stop().await;break;},
                    "panel"=>{},
                    "devices"=> match tokio::task::spawn_blocking(list_output_devices).await? {Ok(list)=>devices=serde_json::to_value(list)?,Err(e)=>message(e)},
                    "start"=> match serde_json::from_value::<BridgeConfig>(arg.clone()) {Ok(config)=>match core.start(config.clone()).await {Ok(_)=>prefs.apply_config(&config),Err(e)=>message(e)},Err(e)=>message(e)},
                    "resume"=>{if let Err(e)=core.start_saved().await {message(e);}},
                    "stop"=>{core.stop().await;},
                    "save"=>match serde_json::from_value::<Preferences>(arg.clone()) {Ok(value)=>match core.save_preferences(value.clone()).await {Ok(())=>prefs=value,Err(e)=>message(e)},Err(e)=>message(e)},
                    "autostart"=>{let enabled=arg.as_bool().unwrap_or(false);match core.set_autostart(enabled).await{Ok(())=>prefs.autostart=enabled,Err(e)=>message(e)}},
                    "open"=>{if let Some(url)=arg.as_str() && let Err(e)=core.open_link(url).await{message(e);}},
                    _=>message("Unknown command"),
                }
            }
        }
        let state = snapshot(core.snapshot().await, &prefs, &devices);
        if state != last {
            output(json!({"state":state}));
            last = state;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn qr_is_vector_and_has_a_quiet_zone() {
        let value = qr("https://127.0.0.1:8081/");
        assert!(value.starts_with("<svg"));
        assert!(value.contains("fill=\"white\""));
        assert!(value.contains("M4 "));
    }
}
