//! WASAPI runs entirely on dedicated COM-initialized threads.
//!
//! The hand-off to the encoder is a watch channel: exactly one current 20 ms
//! packet, never an accumulating queue. Silence scheduling belongs to the RTP
//! sender because loopback is allowed to stop delivering packets in silence.

use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tokio::sync::watch;
use wasapi::{Device, DeviceEnumerator, Direction, SampleType, StreamMode, WaveFormat};
use windows::Win32::Media::Audio::{
    AUDCLNT_BUFFERFLAGS_DATA_DISCONTINUITY, AUDCLNT_BUFFERFLAGS_SILENT, IAudioCaptureClient,
};

const CHANNELS: usize = 2;
const SAMPLE_RATE: usize = 48_000;
const FRAMES: usize = 960;
const FRAME_SAMPLES: usize = FRAMES * CHANNELS;
const FRAME_BYTES: usize = FRAME_SAMPLES * size_of::<i16>();
// At most 40 ms waiting for the audio device, plus its current partial packet.
const PLAYBACK_QUEUE_PACKETS: usize = 2;

pub type PcmFrame = Arc<Vec<i16>>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_virtual: bool,
}

/// Enumerates playback endpoints. Each endpoint can also be captured in
/// loopback mode; VB-CABLE's CABLE Input is a playback endpoint as well.
pub fn list_output_devices() -> Result<Vec<AudioDevice>> {
    // The caller may be an STA UI thread. A fresh thread avoids changing
    // its apartment and guarantees every successful CoInitialize is balanced.
    thread::spawn(|| {
        let _com = ComApartment::new()?;
        let enumerator = DeviceEnumerator::new()?;
        let default_id = enumerator
            .get_default_device(&Direction::Render)
            .and_then(|device| device.get_id())
            .ok();
        let mut devices = Vec::new();
        for device in &enumerator.get_device_collection(&Direction::Render)? {
            let device = device?;
            let id = device.get_id()?;
            let name = device.get_friendlyname()?;
            let lower = name.to_lowercase();
            devices.push(AudioDevice {
                is_default: default_id.as_ref() == Some(&id),
                is_virtual: lower.contains("vb-audio")
                    || lower.contains("cable input")
                    || lower.contains("voicemeeter"),
                id,
                name,
            });
        }
        devices.sort_by(|a, b| b.is_default.cmp(&a.is_default).then(a.name.cmp(&b.name)));
        Ok(devices)
    })
    .join()
    .map_err(|_| anyhow!("A enumeração de áudio encerrou inesperadamente"))?
}

pub struct Capture {
    sender: watch::Sender<Option<PcmFrame>>,
    worker: AudioWorker,
}

impl Capture {
    pub fn start(device_id: Option<&str>) -> Result<Self> {
        let device_id = device_id.map(str::to_owned);
        let (sender, _) = watch::channel(None);
        let producer = sender.clone();
        let worker = AudioWorker::spawn("wasapi-loopback", move |stop, ready| {
            capture_loop(device_id.as_deref(), producer, &stop, &ready)
        })?;
        Ok(Self { sender, worker })
    }

    pub fn subscribe(&self) -> watch::Receiver<Option<PcmFrame>> {
        self.sender.subscribe()
    }

    /// Device removal is observable instead of turning into endless silence.
    pub fn last_error(&self) -> Option<String> {
        self.worker.last_error()
    }
}

pub struct Playback {
    queue: Arc<Mutex<VecDeque<Vec<i16>>>>,
    worker: AudioWorker,
}

impl Playback {
    /// Opened only when an actual microphone peer needs an output device.
    pub fn start(device_id: Option<&str>) -> Result<Self> {
        let device_id = device_id.map(str::to_owned);
        let queue = Arc::new(Mutex::new(VecDeque::with_capacity(PLAYBACK_QUEUE_PACKETS)));
        let consumer = queue.clone();
        let worker = AudioWorker::spawn("wasapi-microphone", move |stop, ready| {
            playback_loop(device_id.as_deref(), consumer, &stop, &ready)
        })?;
        Ok(Self { queue, worker })
    }

    pub fn push(&self, pcm: Vec<i16>) {
        if pcm.is_empty() || !pcm.len().is_multiple_of(CHANNELS) {
            return;
        }
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        push_latest(&mut queue, pcm);
    }

    pub fn last_error(&self) -> Option<String> {
        self.worker.last_error()
    }
}

fn push_latest(queue: &mut VecDeque<Vec<i16>>, pcm: Vec<i16>) {
    while queue.len() >= PLAYBACK_QUEUE_PACKETS {
        queue.pop_front();
    }
    // An Opus packet can contain up to 120 ms. Keep only its most recent 20 ms
    // if a remote peer sends an unusually long packet.
    let keep_from = pcm.len().saturating_sub(FRAME_SAMPLES);
    queue.push_back(pcm[keep_from..].to_vec());
}

struct ComApartment;

impl ComApartment {
    fn new() -> Result<Self> {
        wasapi::initialize_mta()
            .ok()
            .context("Não foi possível inicializar o áudio do Windows")?;
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        wasapi::deinitialize();
    }
}

struct AudioWorker {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    error: Arc<Mutex<Option<String>>>,
}

impl AudioWorker {
    fn spawn(
        name: &str,
        run: impl FnOnce(Arc<AtomicBool>, mpsc::Sender<Result<(), String>>) -> Result<()>
        + Send
        + 'static,
    ) -> Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let error = Arc::new(Mutex::new(None));
        let (ready_tx, ready_rx) = mpsc::channel();
        let thread_stop = stop.clone();
        let thread_error = error.clone();
        let thread = thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || {
                if let Err(error) = run(thread_stop, ready_tx.clone()) {
                    let message = format!("{error:#}");
                    let _ = ready_tx.send(Err(message.clone()));
                    *thread_error
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner()) = Some(message);
                }
            })?;
        let worker = Self {
            stop,
            thread: Some(thread),
            error,
        };
        ready_rx
            .recv()
            .context("O dispositivo de áudio não respondeu")?
            .map_err(anyhow::Error::msg)?;
        Ok(worker)
    }

    fn last_error(&self) -> Option<String> {
        self.error
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
            .or_else(|| {
                if self.thread.as_ref().is_some_and(JoinHandle::is_finished)
                    && !self.stop.load(Ordering::Acquire)
                {
                    Some(
                        "A thread de áudio encerrou inesperadamente. Pare e inicie a transmissão"
                            .to_owned(),
                    )
                } else {
                    None
                }
            })
    }
}

impl Drop for AudioWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Event waits are bounded to 20 ms, including loopback starvation.
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn output_device(device_id: Option<&str>) -> Result<Device> {
    let enumerator = DeviceEnumerator::new()?;
    let device = match device_id.filter(|id| !id.is_empty()) {
        Some(id) => enumerator
            .get_device(id)
            .context("O dispositivo selecionado não está mais disponível. Atualize a lista")?,
        None => enumerator
            .get_default_device(&Direction::Render)
            .context("Nenhum dispositivo de saída padrão está disponível no Windows")?,
    };
    if device.get_direction() != Direction::Render {
        bail!("Selecione um dispositivo de reprodução, como alto-falantes ou CABLE Input");
    }
    Ok(device)
}

fn stream_format() -> WaveFormat {
    WaveFormat::new(16, 16, &SampleType::Int, SAMPLE_RATE, CHANNELS, None)
}

fn capture_loop(
    device_id: Option<&str>,
    sender: watch::Sender<Option<PcmFrame>>,
    stop: &AtomicBool,
    ready: &mpsc::Sender<Result<(), String>>,
) -> Result<()> {
    let _com = ComApartment::new()?;
    let _timers = crate::timing::RealtimeGuard::new();
    crate::timing::boost_audio_thread();
    let device = output_device(device_id)?;
    let mut client = device.get_iaudioclient()?;
    client
        .initialize_client(
            &stream_format(),
            &Direction::Capture,
            &StreamMode::EventsShared {
                autoconvert: true,
                buffer_duration_hns: 0,
            },
        )
        .context("Não foi possível capturar este dispositivo em 48 kHz estéreo")?;
    let event = client.set_get_eventhandle()?;
    // wasapi 0.26's read helper copies a raw slice even for SILENT packets.
    // WASAPI explicitly permits a null buffer for those packets, so handle the
    // flag before touching the pointer and always balance Get/ReleaseBuffer.
    let capture: IAudioCaptureClient = unsafe { client.as_iaudioclient().GetService()? };
    client.start_stream()?;
    let _ = ready.send(Ok(()));
    let mut pending = VecDeque::with_capacity(FRAME_BYTES * 2);
    let mut last_data = Instant::now();
    let result = (|| -> Result<()> {
        while !stop.load(Ordering::Acquire) {
            // Timeout means a silent renderer, not a failed capture stream.
            let _ = event.wait_for_event(20);
            let mut latest = None;
            while unsafe { capture.GetNextPacketSize()? } > 0 {
                if stop.load(Ordering::Acquire) {
                    break;
                }
                let packet = read_capture_packet(&capture)?;
                if packet.discontinuity || last_data.elapsed() > Duration::from_millis(60) {
                    pending.clear();
                    latest = None;
                }
                last_data = Instant::now();
                pending.extend(packet.bytes);
                // Discard complete old packets; preserve sample alignment and
                // the incomplete tail for the next hardware callback.
                while pending.len() >= FRAME_BYTES {
                    let mut samples = Vec::with_capacity(FRAME_SAMPLES);
                    for _ in 0..FRAME_SAMPLES {
                        let lo = pending.pop_front().unwrap();
                        let hi = pending.pop_front().unwrap();
                        samples.push(i16::from_le_bytes([lo, hi]));
                    }
                    latest = Some(Arc::new(samples));
                }
            }
            // Publish only the newest packet after draining WASAPI, so a late
            // worker cannot rapid-fire stale audio into the RTP scheduler.
            if let Some(pcm) = latest {
                sender.send_replace(Some(pcm));
            }
        }
        Ok(())
    })();
    let _ = client.stop_stream();
    result
}

struct CapturePacket {
    bytes: Vec<u8>,
    discontinuity: bool,
}

fn read_capture_packet(capture: &IAudioCaptureClient) -> Result<CapturePacket> {
    let mut buffer = std::ptr::null_mut();
    let mut frames = 0u32;
    let mut flags = 0u32;
    unsafe { capture.GetBuffer(&mut buffer, &mut frames, &mut flags, None, None)? };
    let len = frames as usize * CHANNELS * size_of::<i16>();
    let silent = flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0;
    let bytes = if len == 0 || silent {
        Ok(vec![0; len])
    } else if buffer.is_null() {
        Err(anyhow!("WASAPI retornou áudio sem um buffer válido"))
    } else {
        // The packet remains borrowed from WASAPI until ReleaseBuffer below.
        Ok(unsafe { std::slice::from_raw_parts(buffer, len) }.to_vec())
    };
    unsafe { capture.ReleaseBuffer(frames)? };
    Ok(CapturePacket {
        bytes: bytes?,
        discontinuity: flags & AUDCLNT_BUFFERFLAGS_DATA_DISCONTINUITY.0 as u32 != 0,
    })
}

fn playback_loop(
    device_id: Option<&str>,
    queue: Arc<Mutex<VecDeque<Vec<i16>>>>,
    stop: &AtomicBool,
    ready: &mpsc::Sender<Result<(), String>>,
) -> Result<()> {
    let _com = ComApartment::new()?;
    let _timers = crate::timing::RealtimeGuard::new();
    crate::timing::boost_audio_thread();
    let device = output_device(device_id)?;
    let mut client = device.get_iaudioclient()?;
    client
        .initialize_client(
            &stream_format(),
            &Direction::Render,
            &StreamMode::EventsShared {
                autoconvert: true,
                buffer_duration_hns: 0,
            },
        )
        .context("Não foi possível abrir a saída do microfone em 48 kHz estéreo")?;
    let event = client.set_get_eventhandle()?;
    let render = client.get_audiorenderclient()?;
    client.start_stream()?;
    let _ = ready.send(Ok(()));
    let mut pending = VecDeque::<i16>::new();
    let result = (|| -> Result<()> {
        while !stop.load(Ordering::Acquire) {
            let _ = event.wait_for_event(20);
            let available = client.get_available_space_in_frames()? as usize;
            if available == 0 {
                continue;
            }
            let wanted = available * CHANNELS;
            // Do not preload future silence: write only this device period.
            let mut output = vec![0u8; wanted * size_of::<i16>()];
            let mut position = 0usize;
            while position < wanted {
                if pending.is_empty() {
                    match queue.lock().unwrap_or_else(|p| p.into_inner()).pop_front() {
                        Some(pcm) => pending.extend(pcm),
                        None => break, // The remaining output is already silent.
                    }
                }
                let count = pending.len().min(wanted - position);
                for sample_bytes in output[position * 2..(position + count) * 2]
                    .as_chunks_mut::<2>()
                    .0
                {
                    sample_bytes.copy_from_slice(&pending.pop_front().unwrap().to_le_bytes());
                }
                position += count;
            }
            render.write_to_device(available, &output, None)?;
        }
        Ok(())
    })();
    let _ = client.stop_stream();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stalled_device_retains_only_recent_microphone_audio() {
        let mut queue = VecDeque::new();
        for marker in 0..100 {
            push_latest(&mut queue, vec![marker; FRAME_SAMPLES]);
        }
        assert_eq!(queue.len(), PLAYBACK_QUEUE_PACKETS);
        assert_eq!(queue[0][0], 98);
        assert_eq!(queue[1][0], 99);
    }

    #[test]
    fn long_remote_packets_cannot_create_a_large_playback_backlog() {
        let mut queue = VecDeque::new();
        let mut pcm = vec![1; FRAME_SAMPLES * 5];
        pcm.extend(vec![2; FRAME_SAMPLES]);
        push_latest(&mut queue, pcm);
        assert_eq!(queue[0], vec![2; FRAME_SAMPLES]);
    }
}
