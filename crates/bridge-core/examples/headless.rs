//! Run the HeartWire engine without the desktop interface for diagnostics.
use bridge_core::{BridgeConfig, BridgeMode, BridgeService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = BridgeConfig::default();
    let mut seconds = None::<u64>;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seconds" => {
                seconds = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("Faltou a duração"))?
                        .parse()?,
                )
            }
            "--mic" => config.mode = BridgeMode::Mic,
            "--port" => {
                config.port = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Faltou a porta"))?
                    .parse()?
            }
            "--device" => config.capture_device_id = args.next(),
            "--mic-device" => config.mic_device_id = args.next(),
            "--list-devices" => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&bridge_core::list_output_devices()?)?
                );
                return Ok(());
            }
            _ => anyhow::bail!("Opção desconhecida: {arg}"),
        }
    }
    let data_dir = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("HeartWire-headless");
    let mut service = BridgeService::start(config, data_dir).await?;
    println!("{}", serde_json::to_string_pretty(&service.status())?);
    println!("Ctrl+C para parar");
    if let Some(seconds) = seconds {
        tokio::select! {
            result = tokio::signal::ctrl_c() => result?,
            _ = tokio::time::sleep(std::time::Duration::from_secs(seconds)) => {}
        }
    } else {
        tokio::signal::ctrl_c().await?;
    }
    service.stop().await;
    Ok(())
}
