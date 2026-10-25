//! iOS shortcut profiles contain only public artwork and a verified local URL.
use plist::{Dictionary, Value};
use serde::Deserialize;
use url::Url;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(crate) enum InstallLanguage {
    #[default]
    #[serde(rename = "pt-BR")]
    Portuguese,
    #[serde(rename = "en")]
    English,
}

impl InstallLanguage {
    fn code(self) -> &'static str {
        match self {
            Self::Portuguese => "pt-BR",
            Self::English => "en",
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstallQuery {
    #[serde(default)]
    pub lang: InstallLanguage,
}

fn payload(identifier: String, uuid: Uuid, kind: &str) -> Dictionary {
    let mut content = Dictionary::new();
    content.insert("PayloadIdentifier".into(), identifier.into());
    content.insert("PayloadUUID".into(), uuid.to_string().into());
    content.insert("PayloadType".into(), kind.into());
    content.insert("PayloadVersion".into(), 1.into());
    content.insert("PayloadDisplayName".into(), "HeartWire".into());
    content
}

pub(crate) fn profile(origin: &Url, language: InstallLanguage) -> Result<Vec<u8>, plist::Error> {
    // Stable across languages, downloads and service restarts, unique per
    // origin. IDs identify a shortcut; they never authorize a connection.
    let identity = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        origin.origin().ascii_serialization().as_bytes(),
    );
    let profile_uuid = Uuid::new_v5(&identity, b"profile");
    let clip_uuid = Uuid::new_v5(&identity, b"webclip");
    let mut url = origin.clone();
    url.set_path("/");
    url.set_query(None);
    url.set_fragment(None);
    url.query_pairs_mut().append_pair("lang", language.code());

    let mut clip = payload(
        format!("com.heartwire.webclip.{}", identity.simple()),
        clip_uuid,
        "com.apple.webClip.managed",
    );
    clip.insert("Label".into(), "HeartWire".into());
    clip.insert("URL".into(), url.as_str().into());
    clip.insert(
        "Icon".into(),
        Value::Data(include_bytes!("../../../web/icons/apple-touch-icon.png").to_vec()),
    );
    clip.insert("FullScreen".into(), true.into());
    clip.insert("IsRemovable".into(), true.into());
    clip.insert("Precomposed".into(), true.into());
    clip.insert("IgnoreManifestScope".into(), false.into());

    let mut config = payload(
        format!("com.heartwire.profile.{}", identity.simple()),
        profile_uuid,
        "Configuration",
    );
    config.insert("PayloadOrganization".into(), "saitofernando".into());
    config.insert("PayloadRemovalDisallowed".into(), false.into());
    config.insert("PayloadDescription".into(), match language {
        InstallLanguage::Portuguese => "Adiciona o atalho HeartWire com ícone à Tela de Início. Não instala certificados nem gerenciamento do aparelho.",
        InstallLanguage::English => "Adds the HeartWire shortcut and icon to the Home Screen. Does not install certificates or device management.",
    }.into());
    config.insert(
        "PayloadContent".into(),
        Value::Array(vec![Value::Dictionary(clip)]),
    );
    let mut xml = Vec::new();
    Value::Dictionary(config).to_writer_xml(&mut xml)?;
    Ok(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(bytes: &[u8]) -> Value {
        Value::from_reader_xml(bytes).unwrap()
    }

    #[test]
    fn profile_is_only_a_removable_web_clip_with_the_exact_approved_icon() {
        let url = Url::parse("https://192.168.1.20:8081").unwrap();
        let bytes = profile(&url, InstallLanguage::Portuguese).unwrap();
        let value = read(&bytes);
        let config = value.as_dictionary().unwrap();
        assert_eq!(config["PayloadType"].as_string(), Some("Configuration"));
        assert_eq!(config["PayloadRemovalDisallowed"].as_boolean(), Some(false));
        let clips = config["PayloadContent"].as_array().unwrap();
        assert_eq!(
            clips.len(),
            1,
            "A shortcut profile must never gain unrelated payloads"
        );
        let clip = clips[0].as_dictionary().unwrap();
        assert_eq!(
            clip["PayloadType"].as_string(),
            Some("com.apple.webClip.managed")
        );
        assert_eq!(
            clip["URL"].as_string(),
            Some("https://192.168.1.20:8081/?lang=pt-BR")
        );
        for key in ["FullScreen", "IsRemovable", "Precomposed"] {
            assert_eq!(clip[key].as_boolean(), Some(true));
        }
        assert_eq!(clip["IgnoreManifestScope"].as_boolean(), Some(false));
        assert_eq!(
            clip["Icon"].as_data().unwrap(),
            include_bytes!("../../../web/icons/apple-touch-icon.png")
        );
        for key in [
            "TargetApplicationBundleIdentifier",
            "PayloadCertificateFileName",
            "ServerURL",
            "CheckInURL",
        ] {
            assert!(!clip.contains_key(key));
        }
        assert!(!String::from_utf8(bytes).unwrap().contains("PRIVATE KEY"));
    }

    #[test]
    fn identities_are_stable_for_language_changes_and_distinct_for_other_servers() {
        let url = Url::parse("https://192.168.1.20:8081").unwrap();
        let portuguese = read(&profile(&url, InstallLanguage::Portuguese).unwrap());
        let english = read(&profile(&url, InstallLanguage::English).unwrap());
        let another = read(
            &profile(
                &Url::parse("https://192.168.1.21:8081").unwrap(),
                InstallLanguage::Portuguese,
            )
            .unwrap(),
        );
        let first = portuguese.as_dictionary().unwrap();
        let second = english.as_dictionary().unwrap();
        for key in ["PayloadIdentifier", "PayloadUUID"] {
            assert_eq!(first[key], second[key]);
            assert_ne!(first[key], another.as_dictionary().unwrap()[key]);
        }
        let first_clip = first["PayloadContent"].as_array().unwrap()[0]
            .as_dictionary()
            .unwrap();
        let second_clip = second["PayloadContent"].as_array().unwrap()[0]
            .as_dictionary()
            .unwrap();
        assert_eq!(first_clip["PayloadUUID"], second_clip["PayloadUUID"]);
        assert_ne!(first["PayloadUUID"], first_clip["PayloadUUID"]);
        assert_eq!(
            second_clip["URL"].as_string(),
            Some("https://192.168.1.20:8081/?lang=en")
        );
    }
}
