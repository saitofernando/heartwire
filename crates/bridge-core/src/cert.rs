use ::time::{Duration, OffsetDateTime};
use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use rcgen::{CertificateParams, DnType, ExtendedKeyUsagePurpose, KeyPair};
use std::{net::IpAddr, path::Path};
use x509_parser::{extensions::GeneralName, prelude::*};

pub(crate) async fn ensure_certificate(
    data_dir: &Path,
    addresses: &[IpAddr],
) -> Result<RustlsConfig> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    std::fs::create_dir_all(data_dir).context("Não foi possível criar a pasta dos certificados")?;
    let cert_path = data_dir.join("bridge-cert.pem");
    let key_path = data_dir.join("bridge-key.pem");
    if let (Ok(cert), Ok(key)) = (std::fs::read(&cert_path), std::fs::read(&key_path))
        && certificate_covers(&cert, addresses)
        && let Ok(config) = RustlsConfig::from_pem(cert, key).await
    {
        return Ok(config);
    }
    let mut names = vec!["localhost".to_owned()];
    names.extend(addresses.iter().map(ToString::to_string));
    let mut params = CertificateParams::new(names)?;
    params
        .distinguished_name
        .push(DnType::CommonName, "HeartWire");
    let now = OffsetDateTime::now_utc();
    params.not_before = now - Duration::days(1);
    params.not_after = now + Duration::days(365);
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let key = KeyPair::generate()?;
    let cert = params.self_signed(&key)?;
    let cert_pem = cert.pem().into_bytes();
    let key_pem = key.serialize_pem().into_bytes();
    // Validate before replacing cached files; a damaged key never downgrades
    // the server to insecure HTTP and silently disables the phone microphone.
    let config = RustlsConfig::from_pem(cert_pem.clone(), key_pem.clone()).await?;
    std::fs::write(&key_path, &key_pem).context("Não foi possível salvar a chave HTTPS")?;
    std::fs::write(&cert_path, &cert_pem).context("Não foi possível salvar o certificado HTTPS")?;
    Ok(config)
}

fn certificate_covers(pem_bytes: &[u8], addresses: &[IpAddr]) -> bool {
    let Ok((_, pem)) = parse_x509_pem(pem_bytes) else {
        return false;
    };
    let Ok(cert) = pem.parse_x509() else {
        return false;
    };
    let validity = cert.validity();
    if !validity.is_valid()
        || validity.not_after.timestamp() - validity.not_before.timestamp() > 398 * 86400
    {
        return false;
    }
    if !matches!(cert.extended_key_usage(), Ok(Some(usage)) if usage.value.server_auth) {
        return false;
    }
    let Ok(Some(san)) = cert.subject_alternative_name() else {
        return false;
    };
    san.value
        .general_names
        .iter()
        .any(|name| matches!(name, GeneralName::DNSName("localhost")))
        && addresses.iter().all(|address| {
            san.value
                .general_names
                .iter()
                .any(|name| match (address, name) {
                    (IpAddr::V4(ip), GeneralName::IPAddress(bytes)) => *bytes == ip.octets(),
                    (IpAddr::V6(ip), GeneralName::IPAddress(bytes)) => *bytes == ip.octets(),
                    _ => false,
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reuses_certificate_and_renews_for_ip_changes_and_damaged_keys() {
        let dir = tempfile::tempdir().unwrap();
        let addresses = ["127.0.0.1".parse().unwrap()];
        ensure_certificate(dir.path(), &addresses).await.unwrap();
        let original = std::fs::read(dir.path().join("bridge-cert.pem")).unwrap();
        ensure_certificate(dir.path(), &addresses).await.unwrap();
        assert_eq!(
            original,
            std::fs::read(dir.path().join("bridge-cert.pem")).unwrap()
        );
        let changed = [addresses[0], "192.168.1.25".parse().unwrap()];
        ensure_certificate(dir.path(), &changed).await.unwrap();
        let renewed = std::fs::read(dir.path().join("bridge-cert.pem")).unwrap();
        assert_ne!(original, renewed);
        assert!(certificate_covers(&renewed, &changed));
        std::fs::write(dir.path().join("bridge-key.pem"), "broken").unwrap();
        ensure_certificate(dir.path(), &changed).await.unwrap();
        assert_ne!(
            renewed,
            std::fs::read(dir.path().join("bridge-cert.pem")).unwrap()
        );
    }

    #[tokio::test]
    async fn rejects_expired_and_legacy_certificates() {
        let mut params =
            CertificateParams::new(vec!["localhost".into(), "127.0.0.1".into()]).unwrap();
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        let key = KeyPair::generate().unwrap();
        assert!(!certificate_covers(
            params.self_signed(&key).unwrap().pem().as_bytes(),
            &["127.0.0.1".parse().unwrap()]
        ));
        params.not_before = OffsetDateTime::now_utc() - Duration::days(3);
        params.not_after = OffsetDateTime::now_utc() - Duration::days(1);
        assert!(!certificate_covers(
            params.self_signed(&key).unwrap().pem().as_bytes(),
            &["127.0.0.1".parse().unwrap()]
        ));
    }
}
