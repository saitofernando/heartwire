//! Windows integration: data folder, single instance, startup entry, browser
//! and tray-anchored popup placement.

use std::path::PathBuf;

use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, POINT, RECT},
        Graphics::{
            Dwm::{DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute},
            Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint},
        },
        System::Threading::{
            CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, INFINITE, OpenEventW, SetEvent,
            WaitForMultipleObjects,
        },
        UI::{
            Shell::ShellExecuteW,
            WindowsAndMessaging::{ASFW_ANY, AllowSetForegroundWindow, SW_SHOWNORMAL},
        },
    },
    core::{HSTRING, PCWSTR, w},
};
use windows_registry::{CURRENT_USER, Type};

/// Fixed folder, so preferences, certificate and logs survive reinstalls.
pub fn data_dir() -> PathBuf {
    if let Some(custom) = std::env::var_os("HEARTWIRE_DATA_DIR") {
        return PathBuf::from(custom);
    }
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("com.heartwire.desktop")
}

pub enum Signal {
    Show,
    Quit,
}

/// The first process owns the mutex; later launches signal it and exit.
pub struct Instance {
    _mutex: Handle,
    show: Handle,
    quit: Handle,
}

struct Handle(HANDLE);

// Kernel handles may be waited on and signalled from any thread.
unsafe impl Send for Handle {}

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

const MUTEX: PCWSTR = w!("Local\\com.heartwire.desktop.instance");
const SHOW: PCWSTR = w!("Local\\com.heartwire.desktop.show");
const QUIT: PCWSTR = w!("Local\\com.heartwire.desktop.quit");

impl Instance {
    /// `None` means another HeartWire is running and has been signalled.
    pub fn acquire(signal: Signal) -> Option<Self> {
        unsafe {
            let mutex = Handle(CreateMutexW(None, true, MUTEX).ok()?);
            if GetLastError() == ERROR_ALREADY_EXISTS {
                let name = match signal {
                    Signal::Show => {
                        // Let the running instance bring its panel to the front.
                        let _ = AllowSetForegroundWindow(ASFW_ANY);
                        SHOW
                    }
                    Signal::Quit => QUIT,
                };
                if let Ok(event) = OpenEventW(EVENT_MODIFY_STATE, false, name) {
                    let event = Handle(event);
                    let _ = SetEvent(event.0);
                }
                return None;
            }
            Some(Self {
                _mutex: mutex,
                show: Handle(CreateEventW(None, false, false, SHOW).ok()?),
                quit: Handle(CreateEventW(None, false, false, QUIT).ok()?),
            })
        }
    }

    /// Delivers signals from later launches (and the installer) on a thread.
    pub fn listen(self, mut on_signal: impl FnMut(Signal) + Send + 'static) {
        std::thread::Builder::new()
            .name("single-instance".into())
            .spawn(move || {
                while let Some(signal) = self.wait() {
                    on_signal(signal);
                }
            })
            .expect("single-instance thread");
    }

    fn wait(&self) -> Option<Signal> {
        let result =
            unsafe { WaitForMultipleObjects(&[self.show.0, self.quit.0], false, INFINITE) };
        match result.0 {
            0 => Some(Signal::Show),
            1 => Some(Signal::Quit),
            _ => None,
        }
    }
}

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
// Registered as `"<exe>" --minimized` under this value name.
const RUN_VALUE: &str = "HeartWire";

/// Enabled when registered and not switched off in Task Manager → Startup.
pub fn autostart_enabled() -> bool {
    let registered = CURRENT_USER
        .open(RUN_KEY)
        .and_then(|key| key.get_string(RUN_VALUE))
        .is_ok();
    let approved = CURRENT_USER
        .open(APPROVED_KEY)
        .and_then(|key| key.get_bytes(RUN_VALUE))
        .map(|bytes| bytes.first().is_none_or(|flag| flag % 2 == 0))
        .unwrap_or(true);
    registered && approved
}

pub fn set_autostart(enabled: bool) -> Result<(), String> {
    if enabled {
        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        CURRENT_USER
            .create(RUN_KEY)
            .and_then(|key| key.set_string(RUN_VALUE, format!("\"{}\" --minimized", exe.display())))
            .map_err(|error| error.message())?;
        // 02 marks the entry as enabled for Task Manager.
        if let Ok(key) = CURRENT_USER.create(APPROVED_KEY) {
            let _ = key.set_bytes(
                RUN_VALUE,
                Type::Bytes,
                &[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            );
        }
    } else {
        if let Ok(key) = CURRENT_USER.open(RUN_KEY) {
            let _ = key.remove_value(RUN_VALUE);
        }
        if let Ok(key) = CURRENT_USER.open(APPROVED_KEY) {
            let _ = key.remove_value(RUN_VALUE);
        }
        if autostart_enabled() {
            return Err("a entrada continua registrada".into());
        }
    }
    Ok(())
}

pub fn open_url(url: &str) -> windows::core::Result<()> {
    let url = HSTRING::from(url);
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            &url,
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // ShellExecute reports success with a value greater than 32.
    if result.0 as usize > 32 {
        Ok(())
    } else {
        Err(windows::core::Error::from_thread())
    }
}

/// Windows 11 draws undecorated windows with square corners unless asked.
pub fn round_corners(hwnd: isize) {
    let preference = DWMWCP_ROUND;
    let _ = unsafe {
        DwmSetWindowAttribute(
            HWND(hwnd as *mut _),
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&raw const preference).cast(),
            size_of_val(&preference) as u32,
        )
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// A monitor and its work area (the monitor minus the taskbar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Screen {
    pub monitor: Bounds,
    pub work: Bounds,
}

/// The screen nearest to a point.
pub fn screen_at(x: i32, y: i32) -> Option<Screen> {
    let bounds = |RECT {
                      left,
                      top,
                      right,
                      bottom,
                  }: RECT| Bounds {
        left,
        top,
        right,
        bottom,
    };
    unsafe {
        let monitor = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(monitor, &mut info)
            .as_bool()
            .then(|| Screen {
                monitor: bounds(info.rcMonitor),
                work: bounds(info.rcWork),
            })
    }
}

const GAP: i32 = 8;

/// Place the panel against the taskbar edge, centred on the tray icon along
/// that edge. The icon can sit in the hidden-icons flyout above the taskbar,
/// so only its position along the edge is used, never its distance from it.
pub fn popup_position(
    anchor: Option<Bounds>,
    width: i32,
    height: i32,
    screen: Screen,
) -> (i32, i32) {
    let Screen { monitor, work } = screen;
    let (cx, cy) = anchor.map_or((work.right, work.bottom), |icon| {
        ((icon.left + icon.right) / 2, (icon.top + icon.bottom) / 2)
    });
    // The taskbar is on the side where the work area is smaller than the
    // monitor; an auto-hidden taskbar falls back to the bottom.
    let (x, y) = if work.top > monitor.top {
        (cx - width / 2, work.top + GAP)
    } else if work.left > monitor.left {
        (work.left + GAP, cy - height / 2)
    } else if work.right < monitor.right {
        (work.right - width - GAP, cy - height / 2)
    } else {
        (cx - width / 2, work.bottom - height - GAP)
    };
    (
        x.clamp(
            work.left + GAP,
            (work.right - width - GAP).max(work.left + GAP),
        ),
        y.clamp(
            work.top + GAP,
            (work.bottom - height - GAP).max(work.top + GAP),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONITOR: Bounds = Bounds {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };
    const BOTTOM: Screen = Screen {
        monitor: MONITOR,
        work: Bounds {
            bottom: 1032,
            ..MONITOR
        },
    };

    fn icon(left: i32, top: i32) -> Option<Bounds> {
        Some(Bounds {
            left,
            top,
            right: left + 24,
            bottom: top + 24,
        })
    }

    #[test]
    fn popup_sits_above_a_bottom_taskbar_icon() {
        assert_eq!(
            popup_position(icon(1700, 1044), 380, 500, BOTTOM),
            (1522, 524)
        );
    }

    #[test]
    fn hidden_icons_flyout_still_anchors_to_the_taskbar() {
        // The flyout floats above the taskbar, inside the work area.
        assert_eq!(
            popup_position(icon(1650, 900), 380, 500, BOTTOM),
            (1472, 524)
        );
    }

    #[test]
    fn popup_stays_inside_the_screen() {
        assert_eq!(
            popup_position(icon(1896, 1044), 380, 500, BOTTOM),
            (1532, 524)
        );
    }

    #[test]
    fn popup_follows_top_and_side_taskbars() {
        let top = Screen {
            monitor: MONITOR,
            work: Bounds { top: 48, ..MONITOR },
        };
        assert_eq!(popup_position(icon(1700, 12), 380, 500, top), (1522, 56));
        let left = Screen {
            monitor: MONITOR,
            work: Bounds {
                left: 48,
                ..MONITOR
            },
        };
        assert_eq!(popup_position(icon(12, 900), 380, 500, left), (56, 572));
        let right = Screen {
            monitor: MONITOR,
            work: Bounds {
                right: 1872,
                ..MONITOR
            },
        };
        assert_eq!(
            popup_position(icon(1884, 900), 380, 500, right),
            (1484, 572)
        );
    }

    #[test]
    fn popup_without_icon_uses_bottom_right() {
        assert_eq!(popup_position(None, 380, 500, BOTTOM), (1532, 524));
    }
}
