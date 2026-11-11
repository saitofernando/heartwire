//! Desktop services for the headless audio process. The Qt frontend owns windows and tray.
use std::{fs, path::PathBuf, process::Command};
pub fn data_dir() -> PathBuf {
    std::env::var_os("HEARTWIRE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::data_local_dir()
                .expect("XDG data directory")
                .join("com.heartwire.desktop")
        })
}
fn autostart_path() -> PathBuf {
    dirs::config_dir()
        .expect("XDG config directory")
        .join("autostart/heartwire.desktop")
}
pub fn autostart_enabled() -> bool {
    autostart_path().is_file()
}
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let path = autostart_path();
    if enabled {
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let temporary = path.with_extension("desktop.tmp");
        fs::write(&temporary,"[Desktop Entry]\nType=Application\nName=HeartWire\nExec=/usr/bin/heartwire --minimized\nIcon=heartwire\n").map_err(|e|e.to_string())?;
        fs::rename(temporary, path).map_err(|e| e.to_string())
    } else if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}
pub fn open_url(url: &str) -> std::io::Result<()> {
    if Command::new("xdg-open").arg(url).status()?.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("Cannot open the browser"))
    }
}
