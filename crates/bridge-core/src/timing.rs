//! Balanced Windows timer resolution and priority changes.

use std::sync::{Mutex, OnceLock};
use windows::Win32::{
    Media::{timeBeginPeriod, timeEndPeriod},
    System::Threading::{
        GetCurrentProcess, GetCurrentThread, GetPriorityClass, GetProcessInformation,
        HIGH_PRIORITY_CLASS, PROCESS_CREATION_FLAGS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION, PROCESS_POWER_THROTTLING_STATE,
        ProcessPowerThrottling, SetPriorityClass, SetProcessInformation, SetThreadPriority,
        THREAD_PRIORITY_HIGHEST,
    },
};

#[derive(Default)]
struct PriorityState {
    users: usize,
    previous: u32,
    previous_power: Option<PROCESS_POWER_THROTTLING_STATE>,
}

fn process_state() -> &'static Mutex<PriorityState> {
    static STATE: OnceLock<Mutex<PriorityState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(PriorityState::default()))
}

/// Keep this guard alive while streaming. Multiple streams safely share the
/// process priority change, and the final drop restores the original class.
pub struct RealtimeGuard {
    timer_enabled: bool,
}

impl RealtimeGuard {
    pub fn new() -> Self {
        let mut state = process_state().lock().unwrap_or_else(|p| p.into_inner());
        if state.users == 0 {
            // SAFETY: GetCurrentProcess returns a borrowed pseudo handle.
            unsafe {
                state.previous = GetPriorityClass(GetCurrentProcess());
                let _ = SetPriorityClass(GetCurrentProcess(), HIGH_PRIORITY_CLASS);
                let mut power = PROCESS_POWER_THROTTLING_STATE {
                    Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
                    ..Default::default()
                };
                if GetProcessInformation(
                    GetCurrentProcess(),
                    ProcessPowerThrottling,
                    (&mut power as *mut PROCESS_POWER_THROTTLING_STATE).cast(),
                    size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
                )
                .is_ok()
                {
                    state.previous_power = Some(power);
                    // Windows 11 may otherwise ignore timeBeginPeriod when
                    // the app window is hidden in the system tray.
                    power.ControlMask |= PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION;
                    power.StateMask &= !PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION;
                    let _ = SetProcessInformation(
                        GetCurrentProcess(),
                        ProcessPowerThrottling,
                        (&power as *const PROCESS_POWER_THROTTLING_STATE).cast(),
                        size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
                    );
                }
            }
        }
        state.users += 1;
        Self {
            timer_enabled: unsafe { timeBeginPeriod(1) } == 0,
        }
    }
}

impl Default for RealtimeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RealtimeGuard {
    fn drop(&mut self) {
        if self.timer_enabled {
            unsafe { timeEndPeriod(1) };
        }
        let mut state = process_state().lock().unwrap_or_else(|p| p.into_inner());
        state.users = state.users.saturating_sub(1);
        if state.users == 0 && state.previous != 0 {
            unsafe {
                let _ =
                    SetPriorityClass(GetCurrentProcess(), PROCESS_CREATION_FLAGS(state.previous));
            }
        }
        if state.users == 0
            && let Some(power) = state.previous_power.take()
        {
            unsafe {
                let _ = SetProcessInformation(
                    GetCurrentProcess(),
                    ProcessPowerThrottling,
                    (&power as *const PROCESS_POWER_THROTTLING_STATE).cast(),
                    size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
                );
            }
        }
    }
}

pub fn boost_audio_thread() {
    // HIGHEST keeps the capture/render workers ahead of ordinary UI work
    // without assigning the starvation-prone TIME_CRITICAL priority.
    unsafe {
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
    }
}
