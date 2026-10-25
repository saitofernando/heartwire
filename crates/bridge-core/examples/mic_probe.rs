//! Tests the complete reverse path: Opus -> WebRTC -> WASAPI render -> loopback.
//! A short 400 Hz test tone is played through the default Windows output.
use anyhow::{Context, Result, ensure};
use bridge_core::{BridgeConfig, BridgeMode, BridgeService, audio::Capture};
use std::{sync::Arc, time::Duration};
use webrtc::{
    api::{
        APIBuilder,
        interceptor_registry::register_default_interceptors,
        media_engine::{MIME_TYPE_OPUS, MediaEngine},
    },
    interceptor::registry::Registry,
    media::Sample,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::{TrackLocal, track_local_static_sample::TrackLocalStaticSample},
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = BridgeConfig {
        mode: BridgeMode::Mic,
        port: 18081,
        ..Default::default()
    };
    if let Some(port) = std::env::args().nth(1) {
        config.port = port.parse()?;
    }
    let port = config.port;
    let data_dir = tempfile::tempdir()?;
    let capture = tokio::task::spawn_blocking(|| Capture::start(None)).await??;
    let mut frames = capture.subscribe();
    let mut service = BridgeService::start(config, data_dir.path().to_owned()).await?;
    let mut media = MediaEngine::default();
    media.register_default_codecs()?;
    let registry = register_default_interceptors(Registry::new(), &mut media)?;
    let api = APIBuilder::new()
        .with_media_engine(media)
        .with_interceptor_registry(registry)
        .build();
    let pc = Arc::new(api.new_peer_connection(RTCConfiguration::default()).await?);
    let track = Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_OPUS.into(),
            clock_rate: 48000,
            channels: 2,
            ..Default::default()
        },
        "mic".into(),
        "probe".into(),
    ));
    let sender = pc
        .add_track(track.clone() as Arc<dyn TrackLocal + Send + Sync>)
        .await?;
    let drain = tokio::spawn(async move {
        let mut buffer = [0u8; 1500];
        while sender.read(&mut buffer).await.is_ok() {}
    });
    let offer = pc.create_offer(None).await?;
    let mut gathering = pc.gathering_complete_promise().await;
    pc.set_local_description(offer).await?;
    tokio::time::timeout(Duration::from_secs(10), gathering.recv()).await?;
    let http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    let response = http
        .post(format!("https://127.0.0.1:{port}/mic-offer"))
        .json(&pc.local_description().await.unwrap())
        .send()
        .await?
        .error_for_status()?;
    pc.set_remote_description(response.json::<RTCSessionDescription>().await?)
        .await?;
    tokio::time::timeout(Duration::from_secs(15), async {
        while pc.connection_state() != RTCPeerConnectionState::Connected {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .context("WebRTC não conectou")?;
    ensure!(
        service.status().connected_peers == 1,
        "Contagem de peers incorreta"
    );
    let producer = tokio::spawn(async move {
        let mut encoder =
            opus::Encoder::new(48000, opus::Channels::Stereo, opus::Application::Audio)?;
        let mut pcm = vec![0i16; 1920];
        for frame in 0..960 {
            let value =
                (6000.0 * (std::f64::consts::TAU * 400.0 * frame as f64 / 48000.0).sin()) as i16;
            pcm[frame * 2] = value;
            pcm[frame * 2 + 1] = value;
        }
        let start = tokio::time::Instant::now();
        for frame in 0..150 {
            tokio::time::sleep_until(start + Duration::from_millis(20) * frame).await;
            let packet = encoder.encode_vec(&pcm, 4000)?;
            track
                .write_sample(&Sample {
                    data: packet.into(),
                    duration: Duration::from_millis(20),
                    ..Default::default()
                })
                .await?;
        }
        anyhow::Ok(())
    });
    let until = tokio::time::Instant::now() + Duration::from_secs(3);
    let mut tone_frames = 0;
    let mut max_amplitude = 0f64;
    while let Ok(Ok(())) = tokio::time::timeout_at(until, frames.changed()).await {
        let Some(pcm) = frames.borrow_and_update().clone() else {
            continue;
        };
        let mut re = 0.0;
        let mut im = 0.0;
        for frame in 0..960 {
            let phase = std::f64::consts::TAU * 400.0 * frame as f64 / 48000.0;
            re += pcm[frame * 2] as f64 * phase.cos();
            im += pcm[frame * 2] as f64 * phase.sin();
        }
        let amplitude = 2.0 * re.hypot(im) / 960.0;
        max_amplitude = max_amplitude.max(amplitude);
        if amplitude > 100.0 {
            tone_frames += 1;
        }
    }
    producer.await??;
    let status = service.status();
    pc.close().await?;
    drain.abort();
    service.stop().await;
    drop(capture);
    ensure!(
        status.last_error.is_none(),
        "Erro no áudio: {:?}",
        status.last_error
    );
    ensure!(
        tone_frames >= 30,
        "Som do microfone não apareceu no loopback: {tone_frames} blocos; amplitude máxima {max_amplitude:.1}. Verifique volume/mute do Windows."
    );
    println!(
        "MIC OK: {tone_frames} blocos de áudio com tom 400 Hz; amplitude máxima {max_amplitude:.1}; Opus/WebRTC/WASAPI completos."
    );
    Ok(())
}
