use config::{Config, ConfigError};
use eyre::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub struct ChainConfig {
    pub name: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub ws_url: String,

    pub start_block: Option<u64>,
    pub block_batch_size: Option<u64>,
    pub max_tx_retry: Option<u8>,
    pub min_order_val: u32,
    pub max_order_val: u32,
    pub profitability_threshold: f32,

    pub order_contract_address: String,
    pub filler_poll_interval: u64,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
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
    pub redis_url: String,
}

pub struct AppContext {
    pub config: AppConfig,
}

pub fn config() -> Result<AppConfig, ConfigError> {
    let settings = Config::builder()
        .add_source(config::File::with_name("./config.toml").required(false))
        .add_source(config::Environment::with_prefix("ILR"))
        .build()?;

    settings.try_deserialize::<AppConfig>()
}

pub fn context() -> Result<AppContext> {
    Ok(AppContext { config: config()? })
}
