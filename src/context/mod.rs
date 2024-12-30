use config::{Config, ConfigError};
use serde::Deserialize;
use slog::{o, Drain};
use eyre::Result;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AppConfig {
  pub rpc_url: String,
  pub ws_url: String,
  pub order_contract_address: String,
  pub redis_url: String,
  pub redis_poll_interval: u64,
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
  let config = config()?;

  return Ok(AppContext{
    config: config,
    logger: slog::Logger::root(drain, o!())
  });
}
#[cfg(test)]
mod tests {
  use std::str::FromStr;

use alloy::{primitives::Address, signers::local::PrivateKeySigner};

#[test]
fn test_priv_key_to_address_conversion() {
  // This is a placeholder for the actual test implementation.
  // You would need to implement the logic to convert a private key to an address
  // and then verify the conversion is correct.
  let priv_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";
  let expected_address_str = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";

  // Implement the conversion logic here
  let signer = PrivateKeySigner::from_str(priv_key).unwrap();
  let actual_address_str = signer.address().to_string();

  assert_eq!(actual_address_str, expected_address_str);
}

#[test]
fn test_address_from_vec8() {
  let filler = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 35, 97, 142, 129, 227, 245, 205, 247, 245, 76, 61, 101, 247, 251, 192, 171, 245, 178, 30, 143, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  let expected_address = Address::from_slice(&filler[12..32]);

  let priv_key = "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97";
  let signer = PrivateKeySigner::from_str(priv_key).unwrap();
  let actual_address = signer.address();

  assert_eq!(actual_address, expected_address);
}

}
