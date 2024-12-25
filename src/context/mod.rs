use config::{Config, ConfigError};
use diesel::{Connection, PgConnection};
use serde::Deserialize;
use slog::{o, Drain};
use eyre::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
  pub rpc_url: String,
  pub ws_url: String,
  pub order_contract_address: String,
  pub database_url: String,
  pub from_block: Option<u64>,
}

pub struct AppContext {
  pub connection: PgConnection,
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
  let config = config()?;

  let connection = PgConnection::establish(&config.database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", config.database_url));

  return Ok(AppContext{
    connection: connection,
    config: config,
    logger: slog::Logger::root(drain, o!())
  });
}