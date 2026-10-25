use bridge_core::{BridgeConfig, BridgeMode};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "pt-BR")]
    Portuguese,
    #[serde(rename = "en")]
    English,
}

impl Language {
    pub fn text(self, portuguese: &'static str, english: &'static str) -> &'static str {
        match self {
            Self::Portuguese => portuguese,
            Self::English => english,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub capture_device_id: Option<String>,
    pub mic_device_id: Option<String>,
    pub port: u16,
    pub mode: BridgeMode,
    pub autostart: bool,
    pub language: Language,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            capture_device_id: None,
            mic_device_id: None,
            port: 8081,
            mode: BridgeMode::Listen,
            autostart: false,
            language: Language::default(),
        }
    }
}

impl Preferences {
    pub fn config(&self) -> BridgeConfig {
        BridgeConfig {
            capture_device_id: self.capture_device_id.clone(),
            mic_device_id: self.mic_device_id.clone(),
            port: self.port,
            mode: self.mode,
        }
    }

    pub fn apply_config(&mut self, config: &BridgeConfig) {
        self.capture_device_id = config.capture_device_id.clone();
        self.mic_device_id = config.mic_device_id.clone();
        self.port = config.port;
        self.mode = config.mode;
    }
}

pub fn write_preferences(data_dir: &Path, preferences: &Preferences) -> Result<(), String> {
    if preferences.port == 0 {
        return Err("Escolha uma porta entre 1 e 65535.".into());
    }
    fs::create_dir_all(data_dir).map_err(|error| error.to_string())?;
    let temporary = data_dir.join("preferences.json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(preferences).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("Não foi possível salvar as preferências: {error}"))?;
    fs::rename(temporary, data_dir.join("preferences.json"))
        .map_err(|error| format!("Não foi possível salvar as preferências: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_is_validated_and_does_not_change_audio_configuration() {
        let mut preferences = Preferences::default();
        let config = preferences.config();
        preferences.language = Language::English;
        preferences.apply_config(&config);
        let data = serde_json::to_value(&preferences).unwrap();
        assert_eq!(data["language"], "en");
        assert_eq!(data["mode"], "listen");
        assert_eq!(data["port"], config.port);
        assert!(serde_json::from_str::<Language>("\"invalid\"").is_err());
        let defaults: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(defaults.language, Language::Portuguese);
    }
}
