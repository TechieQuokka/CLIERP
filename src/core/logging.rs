use crate::core::{config::CLIERPConfig, result::CLIERPResult};
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub fn init_logging(config: &CLIERPConfig) -> CLIERPResult<()> {
    let level = match config.logging.level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let env_filter = EnvFilter::from_default_env()
        .add_directive(format!("clierp={}", level).parse().unwrap())
        .add_directive("diesel=warn".parse().unwrap());

    let registry = Registry::default().with(env_filter);

    match config.logging.format.as_str() {
        "compact" => {
            registry.with(fmt::layer().compact()).init();
        }
        _ => {
            registry.with(fmt::layer().pretty()).init();
        }
    }

    tracing::info!("Logging initialized with level: {}", level);
    Ok(())
}
