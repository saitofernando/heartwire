use crate::audio::PcmFrame;
use std::{sync::Arc, time::Duration};
use tokio::{sync::watch, time::Instant};

pub(crate) const FRAME_DURATION: Duration = Duration::from_millis(20);
pub(crate) const SAMPLES_PER_FRAME: usize = 960;

/// Releases captured PCM immediately and supplies silence when capture stalls.
/// Each completed call reserves the next 20 ms slot on a monotonic deadline.
/// Media timestamps are owned by the RTP sender, independently of this clock.
pub(crate) struct FrameScheduler {
    capture: watch::Receiver<Option<PcmFrame>>,
    next_due: Option<Instant>,
    #[cfg(test)]
    frames: u64,
    silence: PcmFrame,
}

impl FrameScheduler {
    pub(crate) fn new(capture: watch::Receiver<Option<PcmFrame>>) -> Self {
        Self {
            capture,
            next_due: None,
            #[cfg(test)]
            frames: 0,
            silence: Arc::new(vec![0; SAMPLES_PER_FRAME * 2]),
        }
    }

    pub(crate) async fn next(&mut self) -> PcmFrame {
        let now = Instant::now();
        let due = self
            .next_due
            .filter(|due| now.saturating_duration_since(*due) <= FRAME_DURATION * 3)
            .unwrap_or(now);
        // A late call starts a fresh slot instead of repaying missed deadlines.
        // During normal capture the pending slot advances by a fixed duration,
        // so ordinary timer jitter cannot accumulate into scheduling drift.
        let deadline = due + FRAME_DURATION * 2;
        let pcm = match tokio::time::timeout_at(deadline, self.capture.changed()).await {
            Ok(Ok(())) => self
                .capture
                .borrow_and_update()
                .clone()
                .unwrap_or_else(|| self.silence.clone()),
            Ok(Err(_)) => {
                // A removed device must not turn a closed watch channel into a
                // busy loop: preserve the media clock until the user restarts.
                tokio::time::sleep_until(deadline).await;
                self.silence.clone()
            }
            Err(_) => self.silence.clone(),
        };
        self.next_due = Some(due + FRAME_DURATION);
        #[cfg(test)]
        {
            self.frames += 1;
        }
        pcm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn silence_keeps_a_twenty_millisecond_media_clock() {
        let (_tx, rx) = watch::channel(None);
        let mut scheduler = FrameScheduler::new(rx);
        let start = Instant::now();
        for _ in 0..50 {
            assert_eq!(scheduler.next().await.len(), 1920);
        }
        assert_eq!(Instant::now() - start, Duration::from_millis(1020));
        assert_eq!(scheduler.frames, 50);
    }

    #[tokio::test(start_paused = true)]
    async fn late_real_audio_wins_over_silence_and_old_audio_is_replaced() {
        let (tx, rx) = watch::channel(None);
        let mut scheduler = FrameScheduler::new(rx);
        tx.send_replace(Some(Arc::new(vec![1; 1920])));
        tx.send_replace(Some(Arc::new(vec![2; 1920])));
        let start = Instant::now();
        assert_eq!(scheduler.next().await[0], 2);
        assert_eq!(Instant::now(), start);
        let next = scheduler.next();
        tokio::pin!(next);
        tokio::select! {
            biased;
            _ = &mut next => panic!("silence displaced real audio within the grace window"),
            _ = tokio::time::sleep(Duration::from_millis(45)) => {}
        }
        tx.send_replace(Some(Arc::new(vec![3; 1920])));
        assert_eq!(next.await[0], 3);
    }

    #[tokio::test(start_paused = true)]
    async fn stalls_slide_schedule_without_catchup_or_media_clock_jumps() {
        let (_tx, rx) = watch::channel(None);
        let mut scheduler = FrameScheduler::new(rx);
        scheduler.next().await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let start = Instant::now();
        scheduler.next().await;
        assert_eq!(Instant::now() - start, Duration::from_millis(40));
        assert_eq!(scheduler.frames, 2);
        let start = Instant::now();
        scheduler.next().await;
        assert_eq!(Instant::now() - start, Duration::from_millis(20));
    }

    #[tokio::test(start_paused = true)]
    async fn removed_device_still_sends_silence_at_constant_rate() {
        let (tx, rx) = watch::channel(None);
        drop(tx);
        let mut scheduler = FrameScheduler::new(rx);
        let start = Instant::now();
        for _ in 0..50 {
            scheduler.next().await;
        }
        assert_eq!(Instant::now() - start, Duration::from_millis(1020));
    }
}
