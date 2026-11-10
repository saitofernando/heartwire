//! XDG session integration and a private socket for show/quit requests.
pub use crate::placement::{Bounds, Screen, popup_position};
use std::{
    fs::{self, File, OpenOptions},
    io,
    os::unix::{fs::PermissionsExt, net::UnixDatagram},
    path::PathBuf,
    process::Command,
};

pub fn data_dir() -> PathBuf {
    std::env::var_os("HEARTWIRE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::data_local_dir()
                .expect("XDG data directory")
                .join("com.heartwire.desktop")
        })
}
pub enum Signal {
    Show,
    Quit,
}
pub struct Instance {
    _lock: File,
    socket: UnixDatagram,
    path: PathBuf,
}
impl Instance {
    pub fn acquire(signal: Signal) -> io::Result<Option<Self>> {
        let dir = dirs::runtime_dir()
            .unwrap_or_else(data_dir)
            .join("heartwire-instance");
        fs::create_dir_all(&dir)?;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
        let path = dir.join("socket");
        let lock = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("lock"))?;
        match lock.try_lock() {
            Ok(()) => {
                if matches!(signal, Signal::Quit) {
                    return Ok(None);
                }
                match fs::remove_file(&path) {
                    Ok(()) => (),
                    Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                    Err(e) => return Err(e),
                }
                let socket = UnixDatagram::bind(&path)?;
                Ok(Some(Self {
                    _lock: lock,
                    socket,
                    path,
                }))
            }
            Err(fs::TryLockError::WouldBlock) => {
                let socket = UnixDatagram::unbound()?;
                let message: &[u8] = match signal {
                    Signal::Show => b"show",
                    Signal::Quit => b"quit",
                };
                for _ in 0..50 {
                    if socket.send_to(message, &path).is_ok() {
                        return Ok(None);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(io::Error::other("A instância existente não respondeu"))
            }
            Err(fs::TryLockError::Error(e)) => Err(e),
        }
    }
    pub fn listen(self, mut on_signal: impl FnMut(Signal) + Send + 'static) {
        std::thread::spawn(move || {
            let mut buf = [0; 16];
            while let Ok(n) = self.socket.recv(&mut buf) {
                match &buf[..n] {
                    b"show" => on_signal(Signal::Show),
                    b"quit" => on_signal(Signal::Quit),
                    _ => (),
                }
            }
        });
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
fn launcher() -> Result<auto_launch::AutoLaunch, String> {
    let path = std::env::current_exe().map_err(|e| e.to_string())?;
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name("HeartWire")
        .set_app_path(&path.to_string_lossy())
        .set_args(&["--minimized"])
        .build()
        .map_err(|e| e.to_string())
}
pub fn autostart_enabled() -> bool {
    launcher()
        .and_then(|a| a.is_enabled().map_err(|e| e.to_string()))
        .unwrap_or(false)
}
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let launcher = launcher()?;
    (if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    })
    .map_err(|e| e.to_string())
}
pub fn open_url(url: &str) -> io::Result<()> {
    let status = Command::new("xdg-open").arg(url).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("Não foi possível abrir o navegador"))
    }
}
pub fn screen_at(_x: i32, _y: i32, window: &winit::window::Window) -> Option<Screen> {
    let monitor = window.current_monitor()?;
    let pos = monitor.position();
    let size = monitor.size();
    let bounds = Bounds {
        left: pos.x,
        top: pos.y,
        right: pos.x + size.width as i32,
        bottom: pos.y + size.height as i32,
    };
    // Wayland doesn't expose global work areas; placement remains the compositor's decision.
    Some(Screen {
        monitor: bounds,
        work: bounds,
    })
}
