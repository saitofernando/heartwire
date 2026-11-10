//! The native audio engine. No Python process or external web service is used.
#[cfg(windows)]
pub mod audio;
#[cfg(any(target_os = "linux", test))]
pub mod audio_linux;
#[cfg(target_os = "linux")]
pub use audio_linux as audio;
mod cert;
mod scheduler;
mod server;
#[cfg(windows)]
pub mod timing;
#[cfg(target_os = "linux")]
#[path = "timing_linux.rs"]
pub mod timing;
mod webclip;

pub use audio::{AudioDevice, list_output_devices};
pub use server::{BridgeConfig, BridgeMode, BridgeService, ServiceStatus};
