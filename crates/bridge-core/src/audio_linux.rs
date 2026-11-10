//! PipeWire/PulseAudio PCM transport through the system's parec/pacat clients.
//! Capture retains one 20 ms frame; playback retains at most two frames.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    io::{Read, Write},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tokio::sync::watch;

const CHANNELS: usize = 2;
const FRAME_SAMPLES: usize = 960 * CHANNELS;
const FRAME_BYTES: usize = FRAME_SAMPLES * 2;
const QUEUE_PACKETS: usize = 2;
pub type PcmFrame = Arc<Vec<i16>>;
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_virtual: bool,
}

#[derive(Deserialize)]
struct Sink {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default, rename = "monitor_source")]
    monitor_source_name: String,
    #[serde(default)]
    properties: HashMap<String, serde_json::Value>,
}
fn pactl(args: &[&str]) -> Result<String> {
    let result = Command::new("pactl")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .context("Instale libpulse e inicie pipewire-pulse na sessão")?;
    ensure!(
        result.status.success(),
        "O servidor PipeWire/PulseAudio não respondeu ao pactl"
    );
    Ok(String::from_utf8(result.stdout)?)
}
fn sinks() -> Result<(Vec<Sink>, String)> {
    let sinks = serde_json::from_str(&pactl(&["--format=json", "list", "sinks"])?)?;
    let default = pactl(&["get-default-sink"])?;
    Ok((sinks, default.trim().to_owned()))
}
fn devices(sinks: &[Sink], default: &str) -> Vec<AudioDevice> {
    let mut result: Vec<_> = sinks
        .iter()
        .map(|sink| AudioDevice {
            id: sink.name.clone(),
            name: if sink.description.is_empty() {
                sink.name.clone()
            } else {
                sink.description.clone()
            },
            is_default: sink.name == default,
            is_virtual: sink.properties.get("device.class").and_then(|v| v.as_str())
                == Some("abstract")
                || sink.name.starts_with("heartwire_")
                || sink.name.contains("null"),
        })
        .collect();
    result.sort_by(|a, b| b.is_default.cmp(&a.is_default).then(a.name.cmp(&b.name)));
    result
}
pub fn list_output_devices() -> Result<Vec<AudioDevice>> {
    let (sinks, default) = sinks()?;
    Ok(devices(&sinks, &default))
}
fn select_sink(sinks: Vec<Sink>, default: &str, id: Option<&str>) -> Result<Sink> {
    let selected = id.filter(|id| !id.is_empty()).unwrap_or(default);
    sinks
        .into_iter()
        .find(|s| s.name == selected)
        .context("Saída de áudio indisponível; atualize a lista e selecione outra")
}
fn client(program: &str, device: &str) -> Command {
    let mut command = Command::new(program);
    command
        .args([
            "--raw",
            "--format=s16le",
            "--rate=48000",
            "--channels=2",
            "--latency-msec=20",
            "--process-time-msec=20",
        ])
        .arg(format!("--device={device}"))
        .stderr(Stdio::null());
    command
}

struct Worker {
    stop: Arc<AtomicBool>,
    child: Mutex<Child>,
    thread: Option<JoinHandle<()>>,
    error: Arc<Mutex<Option<String>>>,
}
impl Worker {
    fn start(
        mut child: Child,
        run: impl FnOnce(Arc<AtomicBool>) -> Result<()> + Send + 'static,
    ) -> Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let error = Arc::new(Mutex::new(None));
        let (thread_stop, thread_error) = (stop.clone(), error.clone());
        let thread = match thread::Builder::new()
            .name("pulse-pcm".into())
            .spawn(move || {
                if let Err(e) = run(thread_stop.clone())
                    && !thread_stop.load(Ordering::Acquire)
                {
                    *thread_error.lock().unwrap() = Some(format!("Áudio Linux: {e:#}"));
                }
            }) {
            Ok(thread) => thread,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.into());
            }
        };
        Ok(Self {
            stop,
            child: Mutex::new(child),
            thread: Some(thread),
            error,
        })
    }
    fn last_error(&self) -> Option<String> {
        self.error.lock().unwrap().clone().or_else(|| {
            if self.thread.as_ref().is_some_and(JoinHandle::is_finished)
                || self
                    .child
                    .lock()
                    .unwrap()
                    .try_wait()
                    .ok()
                    .flatten()
                    .is_some()
            {
                Some(
                    "O processo de áudio encerrou. Verifique PipeWire e reinicie a transmissão."
                        .into(),
                )
            } else {
                None
            }
        })
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Killing our client unblocks read/write even after server/device loss.
        let child = self.child.get_mut().unwrap();
        let _ = child.kill();
        let _ = child.wait();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub struct Capture {
    sender: watch::Sender<Option<PcmFrame>>,
    worker: Worker,
}
impl Capture {
    pub fn start(device_id: Option<&str>) -> Result<Self> {
        let (sinks, default) = sinks()?;
        let sink = select_sink(sinks, &default, device_id)?;
        ensure!(
            !sink.monitor_source_name.is_empty(),
            "A saída selecionada não oferece monitor de captura"
        );
        let mut child = client("parec", &sink.monitor_source_name)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .context("Não foi possível iniciar parec (pacote libpulse)")?;
        let mut output = child.stdout.take().expect("piped stdout");
        let (sender, _) = watch::channel(None);
        let producer = sender.clone();
        let worker = Worker::start(child, move |stop| {
            let mut bytes = [0; FRAME_BYTES];
            while !stop.load(Ordering::Acquire) {
                output
                    .read_exact(&mut bytes)
                    .context("Captura interrompida")?;
                producer.send_replace(Some(Arc::new(decode_pcm(&bytes))));
            }
            Ok(())
        })?;
        Ok(Self { sender, worker })
    }
    pub fn subscribe(&self) -> watch::Receiver<Option<PcmFrame>> {
        self.sender.subscribe()
    }
    pub fn last_error(&self) -> Option<String> {
        self.worker.last_error()
    }
}
fn decode_pcm(bytes: &[u8]) -> Vec<i16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| i16::from_le_bytes(*b))
        .collect()
}

type Queue = Arc<(Mutex<VecDeque<Vec<i16>>>, Condvar)>;
pub struct Playback {
    queue: Queue,
    worker: Worker,
}
impl Playback {
    pub fn start(device_id: Option<&str>) -> Result<Self> {
        let (sinks, default) = sinks()?;
        let sink = select_sink(sinks, &default, device_id)?;
        let mut child = client("pacat", &sink.name)
            .arg("--playback")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .context("Não foi possível iniciar pacat (pacote libpulse)")?;
        let mut input = child.stdin.take().expect("piped stdin");
        let queue: Queue = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));
        let pending = queue.clone();
        let worker = Worker::start(child, move |stop| {
            let mut bytes = Vec::with_capacity(FRAME_BYTES);
            while !stop.load(Ordering::Acquire) {
                let (mutex, ready) = &*pending;
                let mut packets = mutex.lock().unwrap();
                if packets.is_empty() {
                    packets = ready
                        .wait_timeout(packets, Duration::from_millis(20))
                        .unwrap()
                        .0;
                }
                let packet = packets.pop_front();
                drop(packets);
                if let Some(packet) = packet {
                    bytes.clear();
                    bytes.extend(packet.into_iter().flat_map(i16::to_le_bytes));
                    input.write_all(&bytes).context("Reprodução interrompida")?;
                }
            }
            Ok(())
        })?;
        Ok(Self { queue, worker })
    }
    pub fn push(&self, pcm: Vec<i16>) {
        if pcm.is_empty() || !pcm.len().is_multiple_of(CHANNELS) {
            return;
        }
        push_latest(&mut self.queue.0.lock().unwrap(), pcm);
        self.queue.1.notify_one();
    }
    pub fn last_error(&self) -> Option<String> {
        self.worker.last_error()
    }
}
fn push_latest(queue: &mut VecDeque<Vec<i16>>, pcm: Vec<i16>) {
    while queue.len() >= QUEUE_PACKETS {
        queue.pop_front();
    }
    let start = pcm.len().saturating_sub(FRAME_SAMPLES);
    queue.push_back(pcm[start..].to_vec());
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<Sink> {
        serde_json::from_str(r#"[{"name":"speakers","description":"Speakers","monitor_source":"speakers.monitor"},{"name":"heartwire_mic","description":"HeartWire","monitor_source":"heartwire_mic.monitor","properties":{"device.class":"abstract"}}]"#).unwrap()
    }
    #[test]
    fn output_selection_preserves_monitor_and_rejects_missing_device() {
        assert_eq!(
            select_sink(fixture(), "speakers", None)
                .unwrap()
                .monitor_source_name,
            "speakers.monitor"
        );
        assert_eq!(
            select_sink(fixture(), "speakers", Some("heartwire_mic"))
                .unwrap()
                .name,
            "heartwire_mic"
        );
        assert!(select_sink(fixture(), "speakers", Some("removed")).is_err());
        assert!(select_sink(vec![], "speakers", None).is_err());
    }
    #[test]
    fn devices_are_named_sorted_and_virtual_is_marked() {
        let result = devices(&fixture(), "speakers");
        assert!(result[0].is_default);
        assert_eq!(result[0].name, "Speakers");
        assert!(result[1].is_virtual);
    }
    #[test]
    fn pcm_is_little_endian_and_queue_discards_old_audio() {
        assert_eq!(decode_pcm(&[0, 128, 255, 127, 1]), vec![i16::MIN, i16::MAX]);
        let mut queue = VecDeque::new();
        for n in 0..10 {
            push_latest(&mut queue, vec![n; FRAME_SAMPLES * 6]);
        }
        assert_eq!(queue.len(), 2);
        assert_eq!(queue[0], vec![8; FRAME_SAMPLES]);
        assert_eq!(queue[1], vec![9; FRAME_SAMPLES]);
    }
}
