use anyhow::{Context, Result};
use log::{debug, error, info};
use std::error::Error;
use std::str::FromStr;

use raydium_library::common;
use solana_sdk::pubkey::Pubkey;

use crate::cache::Pool;
use crate::client_helper::ClientHelper;

use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::program_pack::Pack;
use spl_token::state::Account as TokenAccount;

#[derive(Debug)]
pub struct ExtendedAmmInfo {
    pub amm_info: raydium_amm::state::AmmInfo,
    pub lp_amount: u64,
    pub coin_vault_balance: u64,
    pub pc_vault_balance: u64,
}

impl ExtendedAmmInfo {
    // Constructor to create an ExtendedAmmInfo from AmmInfo and additional data
    pub fn new(
        amm_info: raydium_amm::state::AmmInfo,
        lp_amount: u64,
        coin_vault_balance: u64,
        pc_vault_balance: u64,
    ) -> Self {
        ExtendedAmmInfo {
            amm_info,
            lp_amount,
            coin_vault_balance,
            pc_vault_balance,
        }
    }
}

impl ClientHelper {
    pub fn fetch_amm_info(&self, pool_id: &Pubkey) -> Result<raydium_amm::state::AmmInfo> {
        common::rpc::get_account::<raydium_amm::state::AmmInfo>(&self.client, pool_id)?
            .ok_or_else(|| anyhow::anyhow!("Pool state not found"))
    }

    pub fn fetch_extended_amm_info(&self, pool_id: &Pubkey) -> Result<ExtendedAmmInfo> {
        let amm_info = self.fetch_amm_info(pool_id)?;

        let coin_vault_address = amm_info.coin_vault;
        let pc_vault_address = amm_info.pc_vault;

        // Query the balance of the coin vault
        let coin_vault_balance = self.fetch_token_balance(&coin_vault_address)?;

        // Query the balance of the pc vault
        let lp_balance = amm_info.lp_amount;
        let pc_vault_balance = self.fetch_token_balance(&pc_vault_address)?;

        debug!("Coin Vault Balance: {:?}", coin_vault_balance);
        debug!("PC Vault Balance: {:?}", pc_vault_balance);
        debug!("LP Amount: {:?}", lp_balance);

        Ok(ExtendedAmmInfo::new(
            amm_info,
            amm_info.lp_amount,
            coin_vault_balance,
            pc_vault_balance,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::Market;

    use super::*;
    use ctor::ctor;

    #[ctor]
    fn init() {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    #[test]
    fn test_fetch_amm_info() {
        let client = ClientHelper::default();
        let market = Market::read_from_cache().unwrap();
        let pool = Pool::read_from_cache(&market.market_id).unwrap();
        let pool_pubkey = Pubkey::from_str(&pool.amm_id).unwrap();

        let r = client.fetch_amm_info(&pool_pubkey);
        assert!(
            r.is_ok(),
            "fetch_amm_info failed with error: {:?}",
            r.unwrap_err()
        );
    }

    #[test]
    fn test_fetch_extended_amm_info() {
        let client = ClientHelper::default();
        let market = Market::read_from_cache().unwrap();
        let pool = Pool::read_from_cache(&market.market_id).unwrap();
        let pool_pubkey = Pubkey::from_str(&pool.amm_id).unwrap();

        let r = client.fetch_extended_amm_info(&pool_pubkey);
        assert!(
            r.is_ok(),
            "fetch_extended_amm_info failed with error: {:?}",
            r.unwrap_err()
        );

        let info = r.unwrap();
        assert!(info.lp_amount > 0, "LP amount should be greater than 0");
        assert!(
            info.coin_vault_balance > 0,
            "Coin vault balance should be greater than 0"
        );
        assert!(
            info.pc_vault_balance > 0,
            "PC vault balance should be greater than 0"
        );
    }
}
