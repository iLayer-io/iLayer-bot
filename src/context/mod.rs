use config::{Config, ConfigError};
use serde::Deserialize;
use slog::{o, Drain};
use eyre::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
  pub rpc_url: String,
  pub ws_url: String,
  pub order_contract_address: String,
  pub from_block: Option<u64>,
}

pub struct AppContext {
  pub config: AppConfig,
  pub logger: slog::Logger,
}

pub fn config() -> Result<AppConfig, ConfigError>{
  let settings = Config::builder()
      .add_source(config::File::with_name("./config.toml").required(false))
      .add_source(config::Environment::with_prefix("APP"))
      .build()?;

  let config = settings.try_deserialize::<AppConfig>()?;
  Ok(config)
}

pub fn context() -> Result<AppContext>{
  let decorator = slog_term::TermDecorator::new().build();
  let drain = std::sync::Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();

  return Ok(AppContext{
    config: config()?,
    logger: slog::Logger::root(drain, o!())
  });
}