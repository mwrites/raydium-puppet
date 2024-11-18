use log::{debug, error};
use serde::de::Error as SerdeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[cfg(feature = "devnet")]
const PREFIX: &str = "devnet_";

#[cfg(not(feature = "devnet"))]
const PREFIX: &str = "";
const CACHE_DIR: &str = "../cache/";

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("File not found at path: {0}")]
    FileNotFound(String),
    #[error("Unable to read file: {0}")]
    ReadError(String),
    #[error("JSON was not well-formatted")]
    JsonError(#[from] serde_json::Error),
    #[error("Market ID does not match")]
    CacheIdMismatch,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    pub market_id: String,
    pub request_queue: String,
    pub event_queue: String,
    pub bids: String,
    pub asks: String,
    pub base_vault: String,
    pub quote_vault: String,
    pub base_mint: String,
    #[serde(rename = "quoteMin")] // yes the original data from raydium is missing a t
    pub quote_mint: String,
}

impl Market {
    pub fn read_from_cache() -> Result<Self, CacheError> {
        let path = std::env::current_dir()
            .expect("Unable to get current directory")
            .join(format!("{}{}market.json", CACHE_DIR, PREFIX));
        if !Path::new(&path).exists() {
            error!("File not found at path: {}", path.to_string_lossy());
            return Err(CacheError::FileNotFound(
                path.to_string_lossy().into_owned(),
            ));
        }
        let data = fs::read_to_string(&path).map_err(|_| {
            error!("Unable to read file: {}", path.to_string_lossy());
            CacheError::ReadError(path.to_string_lossy().into_owned())
        })?;
        let v: Value = serde_json::from_str(&data)?;
        let address_value = v
            .get("address")
            .ok_or_else(|| CacheError::JsonError(SerdeError::custom("Address field not found")))?;
        serde_json::from_value(address_value.clone()).map_err(CacheError::JsonError)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    pub program_id: String,
    pub amm_id: String,
    pub amm_authority: String,
    pub amm_open_orders: String,
    pub lp_mint: String,
    pub coin_mint: String,
    pub pc_mint: String,
    pub coin_vault: String,
    pub pc_vault: String,
    pub withdraw_queue: String,
    pub amm_target_orders: String,
    pub pool_temp_lp: String,
    pub market_program_id: String,
    pub market_id: String,
    pub amm_config_id: String,
    pub fee_destination_id: String,
}

impl Pool {
    pub fn read_from_cache(expected_market_id: &str) -> Result<Self, CacheError> {
        let path = std::env::current_dir()
            .expect("Unable to get current directory")
            .join(format!("{}{}pool.json", CACHE_DIR, PREFIX));
        if !path.exists() {
            error!("File not found at path: {}", path.to_string_lossy());
            return Err(CacheError::FileNotFound(
                path.to_string_lossy().into_owned(),
            ));
        }
        let data = fs::read_to_string(&path).map_err(|_| {
            error!("Unable to read file: {}", path.to_string_lossy());
            CacheError::ReadError(path.to_string_lossy().into_owned())
        })?;
        let v: Value = serde_json::from_str(&data)?;
        let address_value = v
            .get("address")
            .ok_or_else(|| CacheError::JsonError(SerdeError::custom("Address field not found")))?;
        let pool: Pool = serde_json::from_value(address_value.clone())?;
        if pool.market_id != expected_market_id {
            return Err(CacheError::CacheIdMismatch);
        }
        Ok(pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_market_from_cache() {
        let cache = Market::read_from_cache();
        assert!(
            cache.is_ok(),
            "Market should be read successfully: {:?}",
            cache.unwrap_err()
        );
        assert!(
            !cache.unwrap().market_id.is_empty(),
            "AMM ID should not be empty"
        );
    }

    #[test]
    fn test_read_pool_from_cache_with_matching_market_id() {
        let market = Market::read_from_cache().unwrap();
        let cache = Pool::read_from_cache(&market.market_id);
        assert!(
            cache.is_ok(),
            "Pool should be read successfully: {:?}",
            cache.unwrap_err()
        );
        let pool = cache.unwrap();
        assert_eq!(pool.market_id, market.market_id, "Market ID should match");
        assert!(!pool.amm_id.is_empty(), "AMM ID should not be empty");
    }

    #[test]
    fn test_read_pool_from_cache_with_mismatching_market_id() {
        let cache = Pool::read_from_cache("foo");
        match cache {
            Err(CacheError::CacheIdMismatch) => {
                // Successfully caught the expected CacheIdMismatch error
            }
            Err(e) => panic!(
                "Expected CacheIdMismatch, but got a different error: {:?}",
                e
            ),
            Ok(_) => panic!("Expected an error, but got a successful result"),
        }
    }
}
