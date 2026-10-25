use std::path::PathBuf;

use bridge_core::{BridgeConfig, BridgeService, ServiceStatus};
use tokio::sync::Mutex;

use crate::preferences::{Language, Preferences, write_preferences};

struct Controller {
    service: Option<BridgeService>,
    preferences: Preferences,
    last_error: Option<String>,
}

impl Controller {
    fn snapshot(&self) -> ServiceStatus {
        match &self.service {
            Some(service) => service.status(),
            None => ServiceStatus {
                running: false,
                urls: Vec::new(),
                connected_peers: 0,
                mode: self.preferences.mode,
                port: self.preferences.port,
                last_error: self.last_error.clone(),
            },
        }
    }

    fn running(&self) -> bool {
        self.service
            .as_ref()
            .is_some_and(|service| service.status().running)
    }
}

/// Owns the audio service and persisted preferences. Every method is safe to
/// call from the tray, the panel or startup; the lock serializes them.
pub struct Core {
    controller: Mutex<Controller>,
    pub data_dir: PathBuf,
}

impl Core {
    pub fn new(preferences: Preferences, data_dir: PathBuf) -> Self {
        Self {
            controller: Mutex::new(Controller {
                service: None,
                preferences,
                last_error: None,
            }),
            data_dir,
        }
    }

    /// Never waits: the UI keeps its previous snapshot while an operation runs.
    pub fn try_snapshot(&self) -> Option<ServiceStatus> {
        self.controller.try_lock().ok().map(|c| c.snapshot())
    }

    pub async fn snapshot(&self) -> ServiceStatus {
        self.controller.lock().await.snapshot()
    }

    pub async fn save_preferences(&self, mut preferences: Preferences) -> Result<(), String> {
        let mut controller = self.controller.lock().await;
        if controller.running() {
            return Err("Pare a transmissão antes de alterar os dispositivos ou a porta.".into());
        }
        // Language and autostart have their own operations.
        preferences.language = controller.preferences.language;
        preferences.autostart = controller.preferences.autostart;
        write_preferences(&self.data_dir, &preferences)?;
        controller.preferences = preferences;
        Ok(())
    }

    pub async fn start(&self, config: BridgeConfig) -> Result<ServiceStatus, String> {
        let mut controller = self.controller.lock().await;
        if controller.running() {
            return Err("A ponte já está iniciada. Pare antes de trocar a configuração.".into());
        }
        if let Some(mut stopped) = controller.service.take() {
            stopped.stop().await;
        }
        let mut preferences = controller.preferences.clone();
        preferences.apply_config(&config);
        write_preferences(&self.data_dir, &preferences)?;
        controller.preferences = preferences;
        controller.last_error = None;
        match BridgeService::start(config, self.data_dir.clone()).await {
            Ok(service) => {
                let snapshot = service.status();
                controller.service = Some(service);
                tracing::info!("Ponte iniciada");
                Ok(snapshot)
            }
            Err(error) => {
                let message = format!("Não foi possível iniciar a ponte: {error:#}");
                tracing::error!(%message);
                controller.last_error = Some(message.clone());
                Err(message)
            }
        }
    }

    pub async fn start_saved(&self) -> Result<ServiceStatus, String> {
        let config = self.controller.lock().await.preferences.config();
        self.start(config).await
    }

    pub async fn stop(&self) -> ServiceStatus {
        let mut controller = self.controller.lock().await;
        if let Some(mut service) = controller.service.take() {
            service.stop().await;
            tracing::info!("Ponte parada");
        }
        controller.last_error = None;
        controller.snapshot()
    }

    pub async fn set_language(&self, language: Language) -> Result<(), String> {
        let mut controller = self.controller.lock().await;
        let mut preferences = controller.preferences.clone();
        preferences.language = language;
        write_preferences(&self.data_dir, &preferences)?;
        controller.preferences = preferences;
        Ok(())
    }

    pub async fn set_autostart(&self, enabled: bool) -> Result<(), String> {
        let mut controller = self.controller.lock().await;
        let previous = crate::platform::autostart_enabled();
        crate::platform::set_autostart(enabled).map_err(|error| {
            format!("Não foi possível alterar a inicialização com o Windows: {error}")
        })?;
        let mut preferences = controller.preferences.clone();
        preferences.autostart = enabled;
        if let Err(error) = write_preferences(&self.data_dir, &preferences) {
            let _ = crate::platform::set_autostart(previous);
            return Err(error);
        }
        controller.preferences = preferences;
        Ok(())
    }

    /// Only the root URL of a running bridge may be handed to the browser.
    pub async fn open_link(&self, url: &str) -> Result<(), String> {
        let snapshot = self.snapshot().await;
        validate_link(&snapshot, url)?;
        crate::platform::open_url(url).map_err(|error| error.to_string())
    }
}

fn validate_link(snapshot: &ServiceStatus, url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|_| "Endereço inválido.".to_owned())?;
    let allowed = snapshot
        .urls
        .iter()
        .filter_map(|candidate| url::Url::parse(candidate).ok())
        .any(|candidate| candidate.origin() == parsed.origin());
    if !snapshot.running
        || parsed.path() != "/"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || !allowed
    {
        return Err("O endereço precisa ser um dos links da ponte iniciada.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridge_core::BridgeMode;

    fn running() -> ServiceStatus {
        ServiceStatus {
            running: true,
            urls: vec!["https://192.168.0.10:8081".into()],
            connected_peers: 0,
            mode: BridgeMode::Listen,
            port: 8081,
            last_error: None,
        }
    }

    #[test]
    fn only_running_bridge_roots_open() {
        let status = running();
        assert!(validate_link(&status, "https://192.168.0.10:8081/?mode=listen&lang=en").is_ok());
        assert!(validate_link(&status, "https://192.168.0.10:8081/sw.js").is_err());
        assert!(validate_link(&status, "https://192.168.0.11:8081/").is_err());
        assert!(validate_link(&status, "https://user@192.168.0.10:8081/").is_err());
        assert!(validate_link(&status, "file:///C:/Windows").is_err());
        let stopped = ServiceStatus {
            running: false,
            ..status
        };
        assert!(validate_link(&stopped, "https://192.168.0.10:8081/").is_err());
    }
}
