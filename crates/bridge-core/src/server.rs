use crate::{
    audio::{Capture, PcmFrame, Playback},
    cert::ensure_certificate,
    scheduler::{FRAME_DURATION, FrameScheduler},
    timing::RealtimeGuard,
    webclip::{self, InstallQuery},
};
use anyhow::{Context, Result, anyhow};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Query, State},
    http::{HeaderMap, StatusCode, Uri, header, uri::Authority},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, UdpSocket},
    path::PathBuf,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::{sync::watch, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use webrtc::{
    api::{
        API, APIBuilder,
        interceptor_registry::register_default_interceptors,
        media_engine::{MIME_TYPE_OPUS, MediaEngine},
    },
    interceptor::registry::Registry,
    media::Sample,
    peer_connection::{
        RTCPeerConnection,
        configuration::RTCConfiguration,
        peer_connection_state::RTCPeerConnectionState,
        sdp::{sdp_type::RTCSdpType, session_description::RTCSessionDescription},
    },
    rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
    track::{
        track_local::{TrackLocal, track_local_static_sample::TrackLocalStaticSample},
        track_remote::TrackRemote,
    },
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BridgeMode {
    #[default]
    Listen,
    Mic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BridgeConfig {
    pub capture_device_id: Option<String>,
    pub mic_device_id: Option<String>,
    pub port: u16,
    pub mode: BridgeMode,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            capture_device_id: None,
            mic_device_id: None,
            port: 8081,
            mode: BridgeMode::Listen,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStatus {
    pub running: bool,
    pub urls: Vec<String>,
    pub connected_peers: usize,
    pub mode: BridgeMode,
    pub port: u16,
    pub last_error: Option<String>,
}

struct Peer {
    pc: Arc<RTCPeerConnection>,
    cancel: CancellationToken,
}

struct Shared {
    config: BridgeConfig,
    urls: Vec<String>,
    running: AtomicBool,
    last_error: Mutex<Option<String>>,
    peers: Mutex<HashMap<u64, Peer>>,
    next_peer: AtomicU64,
    negotiation: tokio::sync::Mutex<()>,
    capture: Option<watch::Receiver<Option<PcmFrame>>>,
    api: API,
    cancel: CancellationToken,
}

impl Shared {
    fn status(&self) -> ServiceStatus {
        ServiceStatus {
            running: self.running.load(Ordering::Acquire),
            urls: self.urls.clone(),
            connected_peers: self
                .peers
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.pc.connection_state() == RTCPeerConnectionState::Connected)
                .count(),
            mode: self.config.mode,
            port: self.config.port,
            last_error: self.last_error.lock().unwrap().clone(),
        }
    }

    fn record_error(&self, error: impl ToString) {
        let error = error.to_string();
        tracing::error!("{error}");
        *self.last_error.lock().unwrap() = Some(error);
    }
}

/// Owns the HTTPS listener, all WebRTC peers, and the device capture thread.
/// Call stop before replacing it so the port and WASAPI device are released.
pub struct BridgeService {
    state: Arc<Shared>,
    server: Option<JoinHandle<()>>,
    handle: axum_server::Handle<SocketAddr>,
    capture: Option<Capture>,
    _timing: RealtimeGuard,
}

impl BridgeService {
    pub async fn start(config: BridgeConfig, data_dir: PathBuf) -> Result<Self> {
        if config.port == 0 {
            return Err(anyhow!("Escolha uma porta entre 1 e 65535."));
        }
        // Reserve the TCP port BEFORE opening any audio device.
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, config.port))
            .with_context(|| format!("A porta {} está em uso ou bloqueada. Pare a outra instância ou escolha outra porta.", config.port))?;
        listener.set_nonblocking(true)?;
        let addresses = local_addresses();
        let tls = ensure_certificate(&data_dir, &addresses).await?;
        let capture = if config.mode == BridgeMode::Listen {
            let device = config.capture_device_id.clone();
            Some(tokio::task::spawn_blocking(move || Capture::start(device.as_deref())).await??)
        } else {
            None
        };
        Self::from_parts(config, listener, tls, addresses, capture, None).await
    }

    async fn from_parts(
        config: BridgeConfig,
        listener: TcpListener,
        tls: axum_server::tls_rustls::RustlsConfig,
        addresses: Vec<IpAddr>,
        capture: Option<Capture>,
        synthetic: Option<watch::Receiver<Option<PcmFrame>>>,
    ) -> Result<Self> {
        let mut media = MediaEngine::default();
        // Only Opus is registered, so microphone RTP cannot accidentally be
        // negotiated as PCMU and then fed into the Opus decoder.
        media.register_codec(
            RTCRtpCodecParameters {
                capability: opus_capability(),
                payload_type: 111,
                ..Default::default()
            },
            RTPCodecType::Audio,
        )?;
        let interceptors = register_default_interceptors(Registry::new(), &mut media)?;
        let api = APIBuilder::new()
            .with_media_engine(media)
            .with_interceptor_registry(interceptors)
            .build();
        let urls = addresses
            .iter()
            .filter(|ip| !ip.is_loopback())
            .map(|ip| format!("https://{ip}:{}", config.port))
            .collect::<Vec<_>>();
        let urls = if urls.is_empty() {
            vec![format!("https://127.0.0.1:{}", config.port)]
        } else {
            urls
        };
        let state = Arc::new(Shared {
            capture: synthetic.or_else(|| capture.as_ref().map(Capture::subscribe)),
            config,
            urls,
            running: AtomicBool::new(true),
            last_error: Mutex::new(None),
            peers: Mutex::new(HashMap::new()),
            next_peer: AtomicU64::new(1),
            negotiation: tokio::sync::Mutex::new(()),
            api,
            cancel: CancellationToken::new(),
        });
        let app = Router::new()
            .route("/", get(index))
            .route("/app.js", get(static_asset))
            .route("/media-session.js", get(static_asset))
            .route("/styles.css", get(static_asset))
            .route("/manifest.webmanifest", get(static_asset))
            .route("/sw.js", get(static_asset))
            .route("/offline.html", get(static_asset))
            .route("/offline.js", get(static_asset))
            .route("/icons/{file}", get(static_asset))
            .route("/apple-touch-icon.png", get(static_asset))
            .route("/install/heartwire.mobileconfig", get(install_webclip))
            .route("/status", get(status))
            .route("/offer", post(listen_offer))
            .route("/mic-offer", post(mic_offer))
            .layer(DefaultBodyLimit::max(128 * 1024))
            .layer(middleware::map_response(security_headers))
            .with_state(state.clone());
        let handle = axum_server::Handle::new();
        let server = axum_server::from_tcp_rustls(listener, tls)?.handle(handle.clone());
        let weak = Arc::downgrade(&state);
        let task = tokio::spawn(async move {
            if let Err(error) = server.serve(app.into_make_service()).await
                && let Some(state) = weak.upgrade()
            {
                state.record_error(error);
            }
            if let Some(state) = weak.upgrade() {
                state.running.store(false, Ordering::Release);
                state.cancel.cancel();
            }
        });
        if tokio::time::timeout(Duration::from_secs(5), handle.listening())
            .await?
            .is_none()
        {
            task.abort();
            return Err(anyhow!("O servidor HTTPS não conseguiu iniciar."));
        }
        Ok(Self {
            state,
            server: Some(task),
            handle,
            capture,
            _timing: RealtimeGuard::new(),
        })
    }

    pub fn status(&self) -> ServiceStatus {
        let mut status = self.state.status();
        if status.last_error.is_none() {
            status.last_error = self.capture.as_ref().and_then(Capture::last_error);
        }
        status
    }

    pub async fn stop(&mut self) {
        self.state.running.store(false, Ordering::Release);
        self.state.cancel.cancel();
        self.handle.shutdown();
        let peers: Vec<Peer> = self
            .state
            .peers
            .lock()
            .unwrap()
            .drain()
            .map(|(_, peer)| peer)
            .collect();
        for peer in peers {
            peer.cancel.cancel();
            let _ = tokio::time::timeout(Duration::from_secs(2), peer.pc.close()).await;
        }
        if let Some(mut server) = self.server.take()
            && tokio::time::timeout(Duration::from_secs(3), &mut server)
                .await
                .is_err()
        {
            server.abort();
        }
        // Join WASAPI on a blocking thread, never inside a Tokio worker.
        if let Some(capture) = self.capture.take() {
            let _ = tokio::task::spawn_blocking(move || drop(capture)).await;
        }
    }
}

impl Drop for BridgeService {
    fn drop(&mut self) {
        self.state.running.store(false, Ordering::Release);
        self.state.cancel.cancel();
        self.handle.shutdown();
        if let Some(server) = self.server.take() {
            server.abort();
        }
        // Normal app shutdown uses stop(). This fallback still releases every
        // transport if an owner drops the service unexpectedly.
        let peers: Vec<Peer> = self
            .state
            .peers
            .lock()
            .unwrap()
            .drain()
            .map(|(_, peer)| peer)
            .collect();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            for peer in peers {
                peer.cancel.cancel();
                runtime.spawn(async move {
                    let _ = peer.pc.close().await;
                });
            }
        }
    }
}

fn opus_capability() -> RTCRtpCodecCapability {
    RTCRtpCodecCapability {
        mime_type: MIME_TYPE_OPUS.to_owned(),
        clock_rate: 48000,
        channels: 2,
        sdp_fmtp_line:
            "minptime=10;useinbandfec=1;stereo=1;sprop-stereo=1;maxaveragebitrate=128000".to_owned(),
        ..Default::default()
    }
}

async fn index() -> impl IntoResponse {
    Html(include_str!("../../../web/index.html"))
}

// Deliberately not a filesystem server. Neither URL segments nor query strings
// can select certificates, preferences, source files or paths outside this list.
fn embedded_asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    Some(match path {
        "/app.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("../../../web/app.js"),
        ),
        "/media-session.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("../../../web/media-session.js"),
        ),
        "/styles.css" => (
            "text/css; charset=utf-8",
            include_bytes!("../../../web/styles.css"),
        ),
        "/manifest.webmanifest" => (
            "application/manifest+json; charset=utf-8",
            include_bytes!("../../../web/manifest.webmanifest"),
        ),
        "/sw.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("../../../web/sw.js"),
        ),
        "/offline.html" => (
            "text/html; charset=utf-8",
            include_bytes!("../../../web/offline.html"),
        ),
        "/offline.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("../../../web/offline.js"),
        ),
        "/icons/heartwire.svg" => (
            "image/svg+xml",
            include_bytes!("../../../web/icons/heartwire.svg"),
        ),
        "/icons/icon-192.png" => (
            "image/png",
            include_bytes!("../../../web/icons/icon-192.png"),
        ),
        "/icons/icon-512.png" => (
            "image/png",
            include_bytes!("../../../web/icons/icon-512.png"),
        ),
        "/icons/apple-touch-icon.png" | "/apple-touch-icon.png" => (
            "image/png",
            include_bytes!("../../../web/icons/apple-touch-icon.png"),
        ),
        "/icons/maskable-512.png" => (
            "image/png",
            include_bytes!("../../../web/icons/maskable-512.png"),
        ),
        _ => return None,
    })
}

async fn static_asset(uri: Uri) -> Response {
    let Some((mime, body)) = embedded_asset(uri.path()) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // File names are stable across upgrades, so always revalidate. The worker
    // must also update from the network rather than the HTTP cache.
    let mut response = (
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
        .into_response();
    if uri.path() == "/sw.js" {
        response
            .headers_mut()
            .insert("service-worker-allowed", "/".parse().unwrap());
    }
    response
}

async fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers
        .entry(header::CACHE_CONTROL)
        .or_insert("no-store".parse().unwrap());
    headers.insert(header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; media-src 'self' blob:; connect-src 'self'; worker-src 'self'; manifest-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'".parse().unwrap());
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    headers.insert(header::X_FRAME_OPTIONS, "DENY".parse().unwrap());
    headers.insert(header::REFERRER_POLICY, "no-referrer".parse().unwrap());
    headers.insert(
        "permissions-policy",
        "camera=(), geolocation=(), microphone=(self)"
            .parse()
            .unwrap(),
    );
    response
}

async fn status(State(state): State<Arc<Shared>>) -> impl IntoResponse {
    ([(header::CACHE_CONTROL, "no-store")], Json(state.status()))
}

// A downloaded profile permanently stores this address on the phone. Never
// derive it from an arbitrary Host or any proxy header supplied by a client.
fn webclip_origin(
    headers: &HeaderMap,
    uri: &Uri,
    urls: &[String],
    port: u16,
) -> Result<url::Url, HttpError> {
    let rejected = || {
        HttpError(
            StatusCode::FORBIDDEN,
            "Endereço de instalação não permitido.".into(),
        )
    };
    if headers.get_all(header::HOST).iter().count() > 1
        || uri.scheme_str().is_some_and(|scheme| scheme != "https")
    {
        return Err(rejected());
    }
    let host = headers
        .get(header::HOST)
        .map(|value| value.to_str().map_err(|_| rejected()))
        .transpose()?;
    let authority = uri
        .authority()
        .map(|value| value.as_str())
        .or(host)
        .ok_or_else(rejected)?;
    let target = url::Url::parse(&format!("https://{authority}")).map_err(|_| rejected())?;
    if !target.username().is_empty()
        || target.password().is_some()
        || target.path() != "/"
        || target.query().is_some()
        || target.fragment().is_some()
        || target.port_or_known_default() != Some(port)
    {
        return Err(rejected());
    }
    if let Some(host) = host {
        let host = url::Url::parse(&format!("https://{host}")).map_err(|_| rejected())?;
        if host.origin() != target.origin() {
            return Err(rejected());
        }
    }
    let local_alias = matches!(target.host_str(), Some("127.0.0.1" | "localhost"));
    if !local_alias
        && !urls
            .iter()
            .filter_map(|value| url::Url::parse(value).ok())
            .any(|candidate| candidate.origin() == target.origin())
    {
        return Err(rejected());
    }
    Ok(target)
}

async fn install_webclip(
    State(state): State<Arc<Shared>>,
    uri: Uri,
    headers: HeaderMap,
    Query(query): Query<InstallQuery>,
) -> Result<Response, HttpError> {
    validate_origin(&headers, &uri)?;
    let origin = webclip_origin(&headers, &uri, &state.urls, state.config.port)?;
    let body = webclip::profile(&origin, query.lang).map_err(|error| {
        HttpError(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Não foi possível gerar o atalho: {error}"),
        )
    })?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/x-apple-aspen-config"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"HeartWire.mobileconfig\"",
            ),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response())
}

type HttpResult = std::result::Result<Json<RTCSessionDescription>, HttpError>;
#[derive(Debug)]
struct HttpError(StatusCode, String);
impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({"error": self.1}))).into_response()
    }
}
impl From<anyhow::Error> for HttpError {
    fn from(error: anyhow::Error) -> Self {
        Self(StatusCode::BAD_REQUEST, format!("{error:#}"))
    }
}

async fn listen_offer(
    State(state): State<Arc<Shared>>,
    uri: Uri,
    headers: HeaderMap,
    Json(offer): Json<RTCSessionDescription>,
) -> HttpResult {
    negotiate(state, uri, headers, offer, BridgeMode::Listen).await
}
async fn mic_offer(
    State(state): State<Arc<Shared>>,
    uri: Uri,
    headers: HeaderMap,
    Json(offer): Json<RTCSessionDescription>,
) -> HttpResult {
    negotiate(state, uri, headers, offer, BridgeMode::Mic).await
}

fn validate_origin(headers: &HeaderMap, uri: &Uri) -> std::result::Result<(), HttpError> {
    let rejected = || {
        HttpError(
            StatusCode::FORBIDDEN,
            "Origem da conexão não permitida.".into(),
        )
    };
    let Some(origin) = headers.get(header::ORIGIN) else {
        // Native WebRTC clients may omit Origin.
        return Ok(());
    };
    if headers.get_all(header::ORIGIN).iter().count() != 1 {
        return Err(rejected());
    }
    // HTTP/2 carries :authority in the request URI; it need not send Host.
    // Forwarded host headers are deliberately not used for this comparison.
    let authority = uri
        .authority()
        .cloned()
        .or_else(|| {
            headers
                .get(header::HOST)?
                .to_str()
                .ok()?
                .parse::<Authority>()
                .ok()
        })
        .ok_or_else(rejected)?;
    let target = url::Url::parse(&format!("https://{authority}")).map_err(|_| rejected())?;
    let origin = origin
        .to_str()
        .ok()
        .and_then(|value| url::Url::parse(value).ok())
        .ok_or_else(rejected)?;
    let is_bare_origin = |url: &url::Url| {
        url.scheme() == "https"
            && url.host().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.path() == "/"
            && url.query().is_none()
            && url.fragment().is_none()
    };
    if !is_bare_origin(&target) || !is_bare_origin(&origin) || target.origin() != origin.origin() {
        return Err(rejected());
    }
    Ok(())
}

async fn negotiate(
    state: Arc<Shared>,
    uri: Uri,
    headers: HeaderMap,
    offer: RTCSessionDescription,
    mode: BridgeMode,
) -> HttpResult {
    // Local pages have no API for starting/stopping or changing host settings.
    // Reject cross-origin signaling so unrelated pages cannot use this LAN app.
    if let Err(error) = validate_origin(&headers, &uri) {
        tracing::warn!(origin = ?headers.get(header::ORIGIN), host = ?headers.get(header::HOST), authority = ?uri.authority(), "Origem de sinalização recusada");
        return Err(error);
    }
    if !state.running.load(Ordering::Acquire) {
        return Err(HttpError(
            StatusCode::SERVICE_UNAVAILABLE,
            "O servidor está parando.".into(),
        ));
    }
    if state.config.mode != mode {
        return Err(HttpError(
            StatusCode::CONFLICT,
            "Mude o modo no aplicativo do PC e reinicie a transmissão.".into(),
        ));
    }
    if offer.sdp_type != RTCSdpType::Offer {
        return Err(HttpError(
            StatusCode::BAD_REQUEST,
            "Era esperada uma oferta WebRTC.".into(),
        ));
    }
    let _negotiation = state.negotiation.lock().await;
    let count = state.peers.lock().unwrap().len();
    if count >= if mode == BridgeMode::Mic { 1 } else { 8 } {
        return Err(HttpError(
            StatusCode::CONFLICT,
            if mode == BridgeMode::Mic {
                "Um microfone já está conectado. Desconecte-o antes de usar outro."
            } else {
                "Limite de oito aparelhos atingido."
            }
            .into(),
        ));
    }
    let pc = Arc::new(
        state
            .api
            .new_peer_connection(RTCConfiguration {
                ice_servers: vec![],
                ..Default::default()
            })
            .await
            .map_err(|e| HttpError::from(anyhow::Error::from(e)))?,
    );
    let id = state.next_peer.fetch_add(1, Ordering::Relaxed);
    let cancel = state.cancel.child_token();
    state.peers.lock().unwrap().insert(
        id,
        Peer {
            pc: pc.clone(),
            cancel: cancel.clone(),
        },
    );
    install_state_handler(&state, &pc, id);

    let result: Result<RTCSessionDescription> = async {
        if mode == BridgeMode::Listen {
            let track = Arc::new(TrackLocalStaticSample::new(opus_capability(), "audio".to_owned(), "pc-audio".to_owned()));
            let sender = pc.add_track(track.clone() as Arc<dyn TrackLocal + Send + Sync>).await?;
            let rtcp_cancel = cancel.clone();
            tokio::spawn(async move {
                let mut buffer = [0u8; 1500];
                loop { tokio::select! { _ = rtcp_cancel.cancelled() => break, result = sender.read(&mut buffer) => if result.is_err() { break; } } }
            });
            let source = state.capture.as_ref().context("Captura de áudio indisponível")?.clone();
            let weak_pc = Arc::downgrade(&pc);
            let weak_state = Arc::downgrade(&state);
            let send_cancel = cancel.clone();
            tokio::spawn(async move {
                let result = send_audio(track, weak_pc, source, send_cancel.clone()).await;
                if let Err(error) = result && let Some(state) = weak_state.upgrade() { state.record_error(error); cleanup_peer(state, id).await; }
            });
        } else {
            let weak_state = Arc::downgrade(&state);
            let mic_cancel = cancel.clone();
            pc.on_track(Box::new(move |track, _, _| {
                let weak_state = weak_state.clone();
                let cancel = mic_cancel.clone();
                Box::pin(async move {
                    if track.kind() != RTPCodecType::Audio { return; }
                    tokio::spawn(async move {
                        let Some(state) = weak_state.upgrade() else { return; };
                        let device = state.config.mic_device_id.clone();
                        drop(state);
                        let result = receive_microphone(track, device, cancel).await;
                        if let Some(state) = weak_state.upgrade() {
                            if let Err(error) = result { state.record_error(error); }
                            cleanup_peer(state, id).await;
                        }
                    });
                })
            }));
        }
        pc.set_remote_description(offer).await?;
        let answer = pc.create_answer(None).await?;
        let mut gathering = pc.gathering_complete_promise().await;
        pc.set_local_description(answer).await?;
        tokio::select! {
            _ = cancel.cancelled() => return Err(anyhow!("A transmissão foi interrompida.")),
            result = tokio::time::timeout(Duration::from_secs(10), gathering.recv()) => { result.context("Tempo de descoberta de rede esgotado")?; }
        }
        pc.local_description().await.context("WebRTC não gerou uma resposta")
    }.await;
    match result {
        Ok(answer) => {
            // Peers that never complete ICE must not leak a device or count
            // against connection limits forever.
            let weak_state = Arc::downgrade(&state);
            tokio::spawn(async move {
                tokio::select! { _ = cancel.cancelled() => return, _ = tokio::time::sleep(Duration::from_secs(30)) => {} }
                if let Some(state) = weak_state.upgrade() {
                    let connected = state.peers.lock().unwrap().get(&id).is_some_and(|p| {
                        p.pc.connection_state() == RTCPeerConnectionState::Connected
                    });
                    if !connected {
                        cleanup_peer(state, id).await;
                    }
                }
            });
            Ok(Json(answer))
        }
        Err(error) => {
            cleanup_peer(state.clone(), id).await;
            Err(HttpError::from(error))
        }
    }
}

fn install_state_handler(state: &Arc<Shared>, pc: &Arc<RTCPeerConnection>, id: u64) {
    let weak_state = Arc::downgrade(state);
    pc.on_peer_connection_state_change(Box::new(move |connection| {
        let weak_state = weak_state.clone();
        Box::pin(async move {
            tracing::info!(peer = id, ?connection, "WebRTC");
            if matches!(
                connection,
                RTCPeerConnectionState::Failed | RTCPeerConnectionState::Closed
            ) {
                if let Some(state) = weak_state.upgrade() {
                    // Close outside the state-change callback to avoid awaiting
                    // the same transport event loop from inside itself.
                    tokio::spawn(cleanup_peer(state, id));
                }
            } else if connection == RTCPeerConnectionState::Disconnected {
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    if let Some(state) = weak_state.upgrade() {
                        let disconnected = state.peers.lock().unwrap().get(&id).is_some_and(|p| {
                            p.pc.connection_state() == RTCPeerConnectionState::Disconnected
                        });
                        if disconnected {
                            cleanup_peer(state, id).await;
                        }
                    }
                });
            }
        })
    }));
}

async fn cleanup_peer(state: Arc<Shared>, id: u64) {
    let peer = state.peers.lock().unwrap().remove(&id);
    if let Some(peer) = peer {
        peer.cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(2), peer.pc.close()).await;
    }
}

async fn send_audio(
    track: Arc<TrackLocalStaticSample>,
    pc: Weak<RTCPeerConnection>,
    capture: watch::Receiver<Option<PcmFrame>>,
    cancel: CancellationToken,
) -> Result<()> {
    let mut encoder = opus::Encoder::new(48000, opus::Channels::Stereo, opus::Application::Audio)?;
    encoder.set_bitrate(opus::Bitrate::Bits(128000))?;
    encoder.set_vbr(true)?;
    // Opus DTX is disabled by default. Even digital silence is encoded and
    // sent: Safari must observe ~50 packets/s throughout silent passages.
    loop {
        let connected = pc
            .upgrade()
            .is_some_and(|p| p.connection_state() == RTCPeerConnectionState::Connected);
        if connected {
            break;
        }
        tokio::select! { _ = cancel.cancelled() => return Ok(()), _ = tokio::time::sleep(FRAME_DURATION) => {} }
    }
    let mut scheduler = FrameScheduler::new(capture);
    loop {
        let pcm = tokio::select! { _ = cancel.cancelled() => return Ok(()), pcm = scheduler.next() => pcm };
        let packet = encoder.encode_vec(&pcm, 4000)?;
        let sample = Sample {
            data: Bytes::from(packet),
            duration: FRAME_DURATION,
            ..Default::default()
        };
        // No prev_dropped_packets: each successful packet advances RTP by
        // exactly 960 samples, even after dropped capture data or scheduler lag.
        tokio::select! { _ = cancel.cancelled() => return Ok(()), result = track.write_sample(&sample) => result? }
    }
}

async fn receive_microphone(
    track: Arc<TrackRemote>,
    device: Option<String>,
    cancel: CancellationToken,
) -> Result<()> {
    let playback =
        tokio::task::spawn_blocking(move || Playback::start(device.as_deref())).await??;
    let mut decoder = opus::Decoder::new(48000, opus::Channels::Stereo)?;
    let mut buffer = vec![0i16; 5760 * 2]; // Opus allows up to 120ms in a packet.
    let mut last_sequence = None::<u16>;
    let result: Result<()> = async {
        loop {
            let (packet, _) = tokio::select! {
                _ = cancel.cancelled() => return Ok(()),
                packet = track.read_rtp() => match packet {
                    Ok(packet) => packet,
                    // An ended remote track is an ordinary disconnect.
                    Err(_) => return Ok(()),
                }
            };
            if let Some(previous) = last_sequence {
                let distance = packet.header.sequence_number.wrapping_sub(previous);
                if distance == 0 || distance > 32768 {
                    continue;
                } // duplicate or stale packet
            }
            last_sequence = Some(packet.header.sequence_number);
            let frames = decoder.decode(&packet.payload, &mut buffer, false)?;
            if let Some(error) = playback.last_error() {
                return Err(anyhow!(error));
            }
            playback.push(buffer[..frames * 2].to_vec());
        }
    }
    .await;
    let _ = tokio::task::spawn_blocking(move || drop(playback)).await;
    result
}

fn local_addresses() -> Vec<IpAddr> {
    let primary = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
        .ok()
        .and_then(|socket| {
            // connect selects the outbound interface but sends no network packet.
            socket.connect((Ipv4Addr::new(192, 0, 2, 1), 80)).ok()?;
            Some(socket.local_addr().ok()?.ip())
        });
    let mut addresses = if_addrs::get_if_addrs().unwrap_or_default().into_iter().map(|adapter| adapter.ip()).filter(|ip| matches!(ip, IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_link_local() && !v4.is_unspecified())).collect::<Vec<_>>();
    addresses.sort();
    addresses.dedup();
    if let Some(primary) = primary
        && let Some(index) = addresses.iter().position(|ip| *ip == primary)
    {
        addresses.swap(0, index);
    }
    addresses.push(IpAddr::V4(Ipv4Addr::LOCALHOST));
    addresses
}

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
