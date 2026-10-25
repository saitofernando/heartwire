use super::*;
use tokio::sync::mpsc;
use webrtc::{
    api::APIBuilder,
    rtp_transceiver::{
        RTCRtpTransceiverInit, rtp_transceiver_direction::RTCRtpTransceiverDirection,
    },
};

#[test]
fn webclip_uses_local_authorities_and_rejects_forwarded_or_forged_targets() {
    let urls = vec!["https://192.168.1.20:8081".into()];
    for authority in ["192.168.1.20:8081", "127.0.0.1:8081", "LOCALHOST:8081"] {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, authority.parse().unwrap());
        let uri: Uri = "/install/heartwire.mobileconfig".parse().unwrap();
        assert!(webclip_origin(&headers, &uri, &urls, 8081).is_ok());
        let uri: Uri = format!("https://{authority}/install/heartwire.mobileconfig")
            .parse()
            .unwrap();
        headers.remove(header::HOST);
        assert!(webclip_origin(&headers, &uri, &urls, 8081).is_ok());
    }
    for authority in [
        "attacker.example:8081",
        "192.168.1.20:8082",
        "user@192.168.1.20:8081",
    ] {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, authority.parse().unwrap());
        headers.insert("x-forwarded-host", "192.168.1.20:8081".parse().unwrap());
        assert!(
            webclip_origin(
                &headers,
                &"/install/heartwire.mobileconfig".parse().unwrap(),
                &urls,
                8081
            )
            .is_err(),
            "accepted {authority}"
        );
    }
    let mut headers = HeaderMap::new();
    headers.append(header::HOST, "192.168.1.20:8081".parse().unwrap());
    headers.append(header::HOST, "attacker.example:8081".parse().unwrap());
    assert!(
        webclip_origin(
            &headers,
            &"/install/heartwire.mobileconfig".parse().unwrap(),
            &urls,
            8081
        )
        .is_err()
    );
    headers.remove(header::HOST);
    headers.insert(header::HOST, "attacker.example:8081".parse().unwrap());
    assert!(
        webclip_origin(
            &headers,
            &"https://192.168.1.20:8081/install/heartwire.mobileconfig"
                .parse()
                .unwrap(),
            &urls,
            8081
        )
        .is_err()
    );
    headers.remove(header::HOST);
    assert!(
        webclip_origin(
            &headers,
            &"http://192.168.1.20:8081/install/heartwire.mobileconfig"
                .parse()
                .unwrap(),
            &urls,
            8081
        )
        .is_err()
    );
}

#[test]
fn origin_uses_http2_authority_and_keeps_http1_host_support() {
    let mut headers = HeaderMap::new();
    headers.insert(header::ORIGIN, "https://192.168.1.20:8081".parse().unwrap());
    let uri: Uri = "https://192.168.1.20:8081/offer".parse().unwrap();
    assert!(validate_origin(&headers, &uri).is_ok());
    headers.insert(header::HOST, "192.168.1.20:8081".parse().unwrap());
    assert!(validate_origin(&headers, &"/offer".parse().unwrap()).is_ok());
}

#[test]
fn origins_compare_normalized_https_ports_and_ipv6_hosts() {
    for (origin, target) in [
        ("https://LOCALHOST:443", "https://localhost/offer"),
        ("https://[::1]:8081", "https://[::1]:8081/mic-offer"),
    ] {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, origin.parse().unwrap());
        assert!(validate_origin(&headers, &target.parse().unwrap()).is_ok());
    }
}

#[test]
fn origin_rejects_other_sites_ports_schemes_and_opaque_origins() {
    let uri: Uri = "https://192.168.1.20:8081/offer".parse().unwrap();
    for origin in [
        "https://unrelated.example",
        "https://192.168.1.20:8082",
        "http://192.168.1.20:8081",
        "null",
        "https://user@192.168.1.20:8081",
        "https://192.168.1.20:8081/path",
    ] {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, origin.parse().unwrap());
        assert!(
            validate_origin(&headers, &uri).is_err(),
            "accepted {origin}"
        );
    }
}

#[test]
fn missing_origin_is_allowed_but_invalid_or_duplicate_origins_are_rejected() {
    let uri: Uri = "https://127.0.0.1:8081/offer".parse().unwrap();
    let mut headers = HeaderMap::new();
    assert!(validate_origin(&headers, &uri).is_ok());
    headers.insert(
        header::ORIGIN,
        axum::http::HeaderValue::from_bytes(&[255]).unwrap(),
    );
    assert!(validate_origin(&headers, &uri).is_err());
    headers.insert(header::ORIGIN, "https://127.0.0.1:8081".parse().unwrap());
    headers.append(header::ORIGIN, "https://unrelated.example".parse().unwrap());
    assert!(validate_origin(&headers, &uri).is_err());
}

/// Real HTTPS static routes and privacy headers; no physical sound card required.
#[tokio::test]
async fn pwa_assets_are_embedded_scoped_and_served_with_security_headers() {
    let dir = tempfile::tempdir().unwrap();
    let addresses = vec![IpAddr::V4(Ipv4Addr::LOCALHOST)];
    let tls = ensure_certificate(dir.path(), &addresses).await.unwrap();
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    // Mic playback opens lazily, so this test needs no physical audio device.
    let config = BridgeConfig {
        port,
        mode: BridgeMode::Mic,
        ..Default::default()
    };
    let mut service = BridgeService::from_parts(config, listener, tls, addresses, None, None)
        .await
        .unwrap();
    let http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .http2_prior_knowledge()
        .build()
        .unwrap();
    let base = format!("https://127.0.0.1:{port}");

    let page = http.get(format!("{base}/")).send().await.unwrap();
    assert_eq!(page.status(), reqwest::StatusCode::OK);
    assert_eq!(page.headers()["cache-control"], "no-store");
    assert_eq!(page.headers()["x-content-type-options"], "nosniff");
    assert_eq!(page.headers()["x-frame-options"], "DENY");
    let csp = page.headers()["content-security-policy"].to_str().unwrap();
    assert!(csp.contains("worker-src 'self'"));
    assert!(!csp.contains("unsafe-inline"));
    let html = page.text().await.unwrap();
    assert!(html.contains("apple-touch-icon"));
    assert!(html.contains("/manifest.webmanifest"));
    assert!(html.contains("<title>HeartWire</title>"));
    assert!(html.contains("/media-session.js"));

    let controls = http
        .get(format!("{base}/media-session.js"))
        .send()
        .await
        .unwrap();
    assert_eq!(controls.status(), reqwest::StatusCode::OK);
    assert_eq!(controls.headers()["cache-control"], "no-cache");
    assert!(
        controls.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/javascript")
    );
    assert!(
        controls
            .text()
            .await
            .unwrap()
            .contains("HeartWireMediaSession")
    );

    let manifest = http
        .get(format!("{base}/manifest.webmanifest"))
        .send()
        .await
        .unwrap();
    assert!(
        manifest.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("application/manifest+json")
    );
    let manifest: serde_json::Value = manifest.json().await.unwrap();
    assert_eq!(manifest["name"], "HeartWire");
    assert_eq!(manifest["start_url"], "/");
    assert_eq!(manifest["scope"], "/");
    assert_eq!(manifest["display"], "standalone");
    for icon in manifest["icons"].as_array().unwrap() {
        let path = icon["src"].as_str().unwrap();
        let response = http.get(format!("{base}{path}")).send().await.unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        assert_eq!(response.headers()["content-type"], "image/png");
        let bytes = response.bytes().await.unwrap();
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }
    let worker = http.get(format!("{base}/sw.js")).send().await.unwrap();
    assert_eq!(worker.headers()["service-worker-allowed"], "/");
    assert_eq!(worker.headers()["cache-control"], "no-cache");
    assert!(
        worker.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/javascript")
    );
    let status = http.get(format!("{base}/status")).send().await.unwrap();
    assert_eq!(status.headers()["cache-control"], "no-store");
    let http1 = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .http1_only()
        .build()
        .unwrap();
    for (client, version) in [
        (&http1, reqwest::Version::HTTP_11),
        (&http, reqwest::Version::HTTP_2),
    ] {
        let response = client
            .get(format!("{base}/install/heartwire.mobileconfig?lang=en"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        assert_eq!(response.version(), version);
        assert_eq!(
            response.headers()["content-type"],
            "application/x-apple-aspen-config"
        );
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(
            response.headers()["content-disposition"],
            "attachment; filename=\"HeartWire.mobileconfig\""
        );
        let body = response.bytes().await.unwrap();
        let profile = plist::Value::from_reader_xml(body.as_ref()).unwrap();
        let config = profile.as_dictionary().unwrap();
        let contents = config["PayloadContent"].as_array().unwrap();
        assert_eq!(contents.len(), 1);
        let clip = contents[0].as_dictionary().unwrap();
        assert_eq!(
            clip["URL"].as_string(),
            Some(format!("{base}/?lang=en").as_str())
        );
        assert_eq!(
            clip["Icon"].as_data().unwrap(),
            include_bytes!("../../../web/icons/apple-touch-icon.png")
        );
    }
    for query in [
        "lang=invalid",
        "url=https%3A%2F%2Fattacker.example",
        "lang=en&token=private",
    ] {
        let response = http
            .get(format!("{base}/install/heartwire.mobileconfig?{query}"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    }
    let forged = http1
        .get(format!("{base}/install/heartwire.mobileconfig"))
        .header(header::HOST, format!("attacker.example:{port}"))
        .header("x-forwarded-host", format!("127.0.0.1:{port}"))
        .send()
        .await
        .unwrap();
    assert_eq!(forged.status(), reqwest::StatusCode::FORBIDDEN);
    let foreign = http
        .get(format!("{base}/install/heartwire.mobileconfig"))
        .header(header::ORIGIN, "https://attacker.example")
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), reqwest::StatusCode::FORBIDDEN);
    for path in [
        "/icons/not-an-icon.png",
        "/preferences.json",
        "/certificate.pem",
        "/Cargo.toml",
    ] {
        let response = http.get(format!("{base}{path}")).send().await.unwrap();
        assert_eq!(
            response.status(),
            reqwest::StatusCode::NOT_FOUND,
            "exposed {path}"
        );
        assert_eq!(response.headers()["cache-control"], "no-store");
    }
    for path in [
        "/icons/../certificate.pem",
        "/icons/%2e%2e%2fpreferences.json",
        "/icons/heartwire.svg/extra",
    ] {
        assert!(embedded_asset(path).is_none(), "accepted {path}");
    }
    service.stop().await;
}

/// Real HTTPS, ICE, DTLS/SRTP and Opus; no physical sound card required.
#[tokio::test]
async fn webrtc_tone_and_silence_have_gapless_rtp_and_stable_cadence() {
    let dir = tempfile::tempdir().unwrap();
    let addresses = vec![IpAddr::V4(Ipv4Addr::LOCALHOST)];
    let tls = ensure_certificate(dir.path(), &addresses).await.unwrap();
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let (source, capture) = watch::channel(None);
    let config = BridgeConfig {
        port,
        ..Default::default()
    };
    let mut service =
        BridgeService::from_parts(config, listener, tls, addresses, None, Some(capture))
            .await
            .unwrap();
    let mut media = MediaEngine::default();
    media.register_default_codecs().unwrap();
    let registry = register_default_interceptors(Registry::new(), &mut media).unwrap();
    let api = APIBuilder::new()
        .with_media_engine(media)
        .with_interceptor_registry(registry)
        .build();
    let client = Arc::new(
        api.new_peer_connection(RTCConfiguration::default())
            .await
            .unwrap(),
    );
    let (packets, mut received) = mpsc::unbounded_channel();
    client.on_track(Box::new(move |track, _, _| {
        let packets = packets.clone();
        Box::pin(async move {
            tokio::spawn(async move {
                while let Ok((packet, _)) = track.read_rtp().await {
                    if packets.send((tokio::time::Instant::now(), packet)).is_err() {
                        break;
                    }
                }
            });
        })
    }));
    client
        .add_transceiver_from_kind(
            RTPCodecType::Audio,
            Some(RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Recvonly,
                send_encodings: vec![],
            }),
        )
        .await
        .unwrap();
    let offer = client.create_offer(None).await.unwrap();
    let mut gathering = client.gathering_complete_promise().await;
    client.set_local_description(offer).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), gathering.recv())
        .await
        .unwrap();
    let http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .http2_prior_knowledge()
        .build()
        .unwrap();
    let offer = client.local_description().await.unwrap();
    let http1 = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .http1_only()
        .build()
        .unwrap();
    let response = http1
        .post(format!("https://127.0.0.1:{port}/mic-offer"))
        .header("Origin", format!("https://127.0.0.1:{port}"))
        .json(&offer)
        .send()
        .await
        .unwrap();
    assert_eq!(response.version(), reqwest::Version::HTTP_11);
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
    let rejected = http
        .post(format!("https://127.0.0.1:{port}/mic-offer"))
        .header("Origin", format!("https://127.0.0.1:{port}"))
        .json(&offer)
        .send()
        .await
        .unwrap();
    assert_eq!(rejected.status(), reqwest::StatusCode::CONFLICT);
    let rejected = http
        .post(format!("https://127.0.0.1:{port}/offer"))
        .header("Origin", "https://unrelated.example")
        .json(&offer)
        .send()
        .await
        .unwrap();
    assert_eq!(rejected.status(), reqwest::StatusCode::FORBIDDEN);
    let response = http
        .post(format!("https://127.0.0.1:{port}/offer"))
        .header("Origin", format!("https://127.0.0.1:{port}"))
        .json(&offer)
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "same-origin HTTP/2 offer: {}",
        response.status()
    );
    assert_eq!(response.version(), reqwest::Version::HTTP_2);
    let answer: RTCSessionDescription = response.json().await.unwrap();
    client.set_remote_description(answer).await.unwrap();
    tokio::time::timeout(Duration::from_secs(15), received.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(service.status().connected_peers, 1);
    let mut tone = vec![0i16; 1920];
    for frame in 0..960 {
        let value =
            (12000.0 * (std::f64::consts::TAU * 400.0 * frame as f64 / 48000.0).sin()) as i16;
        tone[frame * 2] = value;
        tone[frame * 2 + 1] = value;
    }
    let tone = Arc::new(tone);
    let started = tokio::time::Instant::now();
    let producer = tokio::spawn(async move {
        for frame in 0..100 {
            tokio::time::sleep_until(started + FRAME_DURATION * frame).await;
            source.send_replace(Some(tone.clone()));
        }
        // WASAPI publishes nothing at all while idle.
        tokio::time::sleep(Duration::from_secs(3)).await;
        drop(source);
    });
    let mut rows = Vec::new();
    let until = started + Duration::from_secs(4);
    while let Ok(Some(row)) = tokio::time::timeout_at(until, received.recv()).await {
        rows.push(row);
    }
    for (name, begin, end) in [("tone", 0.3, 1.9), ("silence", 2.3, 3.9)] {
        let phase = rows
            .iter()
            .filter(|(time, _)| {
                let elapsed = time.duration_since(started).as_secs_f64();
                elapsed >= begin && elapsed < end
            })
            .collect::<Vec<_>>();
        assert!(phase.len() > 65, "{name}: only {} packets", phase.len());
        let wall = phase
            .last()
            .unwrap()
            .0
            .duration_since(phase[0].0)
            .as_secs_f64();
        let media = phase
            .last()
            .unwrap()
            .1
            .header
            .timestamp
            .wrapping_sub(phase[0].1.header.timestamp) as f64
            / 48000.0;
        let rate = (phase.len() - 1) as f64 / wall;
        assert!((45.0..55.0).contains(&rate), "{name}: {rate:.1} pkt/s");
        assert!(
            (wall - media).abs() < 0.15,
            "{name}: wall={wall:.3}, media={media:.3}"
        );
        assert!(
            phase.windows(2).all(|pair| pair[1]
                .1
                .header
                .timestamp
                .wrapping_sub(pair[0].1.header.timestamp)
                == 960),
            "{name}: timestamp gaps"
        );
        println!("{name}: {rate:.1} pkt/s; wall={wall:.3}s media={media:.3}s; zero RTP gaps");
    }
    producer.abort();
    client.close().await.unwrap();
    service.stop().await;
    assert!(!service.status().running);
    assert_eq!(service.status().connected_peers, 0);
    let rebound = TcpListener::bind((Ipv4Addr::LOCALHOST, port)).unwrap();
    drop(rebound);
}
