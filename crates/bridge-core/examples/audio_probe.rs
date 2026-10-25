//! Hardware smoke test: cargo run -p bridge-core --example audio_probe
//! Add --tone to render a short audible tone and verify real loopback samples.
use anyhow::Result;
use bridge_core::audio::{Capture, Playback, list_output_devices};
use std::{f32::consts::TAU, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let devices = list_output_devices()?;
    for device in &devices {
        println!(
            "{}{}: {}",
            if device.is_default { "[default] " } else { "" },
            if device.is_virtual { "[virtual] " } else { "" },
            device.name
        );
    }
    let capture = Capture::start(None)?;
    let mut samples = capture.subscribe();
    let tone = std::env::args().any(|arg| arg == "--tone");
    let playback = if tone {
        Some(Playback::start(None)?)
    } else {
        None
    };
    let start = tokio::time::Instant::now();
    let mut count = 0usize;
    let mut peak = 0i16;
    let mut phase = 0usize;
    let mut tick = tokio::time::interval(Duration::from_millis(20));
    while start.elapsed() < Duration::from_secs(2) {
        tokio::select! {
            changed = samples.changed() => {
                changed?;
                if let Some(pcm) = samples.borrow_and_update().as_ref() {
                    assert_eq!(pcm.len(), 1920);
                    peak = peak.max(pcm.iter().copied().map(i16::saturating_abs).max().unwrap_or(0));
                    count += 1;
                }
            }
            _ = tick.tick() => {
                if let Some(playback) = &playback {
                    let mut pcm = Vec::with_capacity(1920);
                    for _ in 0..960 {
                        let sample = ((phase as f32 * TAU * 440.0 / 48000.0).sin() * 4000.0) as i16;
                        pcm.extend_from_slice(&[sample, sample]);
                        phase += 1;
                    }
                    playback.push(pcm);
                    anyhow::ensure!(playback.last_error().is_none(), "{:?}", playback.last_error());
                }
                anyhow::ensure!(capture.last_error().is_none(), "{:?}", capture.last_error());
            }
        }
    }
    println!(
        "Capture: {count} blocks in 2 seconds, peak {peak}, error {:?}",
        capture.last_error()
    );
    if tone {
        anyhow::ensure!(
            count > 60 && peak > 100,
            "Loopback did not receive the rendered tone"
        );
    }
    drop(playback);
    drop(capture);
    println!("Audio threads stopped and joined cleanly.");
    Ok(())
}
