//! PipeWire manages device scheduling. RTP keeps its existing monotonic clock.
//! Do not assign real-time process priority or require elevated privileges.
#[derive(Default)]
pub struct RealtimeGuard;
impl RealtimeGuard {
    pub fn new() -> Self {
        Self
    }
}
pub fn boost_audio_thread() {}
