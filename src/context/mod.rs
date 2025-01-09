use config::{Config, ConfigError};
use eyre::Result;
use serde::Deserialize;
use slog::{o, Drain};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ChainConfig {
    pub name: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub ws_url: String,

    pub start_block: Option<u64>,
    pub block_batch_size: Option<u16>,
    pub max_tx_retry: Option<u8>,
    pub min_order_val: u32,
    pub max_order_val: u32,
    pub profitability_threshold: f32,

    pub order_contract_address: String,
    pub filler_poll_interval: u64,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TokenConfig {
    pub name: String,
    pub symbol: String,
    #[serde(rename = "type")]
    pub token_type: String,
    pub address: String,
    pub decimals: Option<u8>,
    pub price_feed: String,
    pub display_decimals: Option<u8>,
    pub image: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub chain: Vec<ChainConfig>,
    pub postgres_url: String,
}

pub struct AppContext {
    pub config: AppConfig,
    pub logger: slog::Logger,
}

pub fn config() -> Result<AppConfig, ConfigError> {
    let settings = Config::builder()
        .add_source(config::File::with_name("./config.toml").required(false))
        .add_source(config::Environment::with_prefix("ILR"))
        .build()?;

    let config = settings.try_deserialize::<AppConfig>()?;
    Ok(config)
}

pub fn context() -> Result<AppContext> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = std::sync::Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();
    let config = config()?;

    return Ok(AppContext {
        config,
        logger: slog::Logger::root(drain, o!()),
    });
}
