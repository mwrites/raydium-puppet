#![allow(unused)]
#![allow(dead_code)]

use ansi_term::Colour;
use log::{debug, error, info};
use std::error::Error;
use std::str::FromStr;

use raydium_amm::state::AmmInfo;
use raydium_library::amm::{self, AmmCommands};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;

use crate::add_liquidity::build_add_liquidity_ix;
use crate::cache::{Market, Pool};
use crate::client_helper::ClientHelper;
use crate::fetch_pool;
use crate::remove_liquidity::build_remove_liquidity_ix;
use crate::*;

pub fn add_remove_liquidity(
    client: &ClientHelper,
    pool_id: &Pubkey,
    pool_info: &AmmInfo,
    add_amount_no_dec: u64,
    remove_amount_no_dec: u64,
    slippage: f64,
    dryrun: bool,
) -> Result<(), Box<dyn Error>> {
    let add_ix =
        build_add_liquidity_ix(client, pool_id, pool_info, add_amount_no_dec, slippage)?;
    let remove_ix = build_remove_liquidity_ix(
        client,
        pool_id,
        pool_info,
        remove_amount_no_dec,
        slippage,
    )?;
    let ixs: Vec<Instruction> = add_ix.into_iter().chain(remove_ix).collect();
    client.process_transaction(&ixs, dryrun);
    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn test_add_remove_liquidity() {
        // SETUP
        // obviously it will be more useful if we add from one market and remove from another, it's possible you just need:
        // 1. create the market
        // 2. create the pool
        // 3. give yourself some of the mintA' mintB'
        // Refer to tests/raydium.ts which setup the whole thing for one market
        let client = ClientHelper::default();
        let market = Market::read_from_cache().unwrap();
        let pool = Pool::read_from_cache(&market.market_id).unwrap();
        let pool_pubkey = Pubkey::from_str(&pool.amm_id).unwrap();

        // BEFORE
        let pool_info = client
            .fetch_extended_amm_info(&pool_pubkey)
            .expect("Failed to fetch initial pool balances");
        let pre_user_lp_amount = client
            .derive_ata_and_fetch_balance(
                &client.user_keypair.pubkey(),
                &pool_info.amm_info.lp_mint,
            )
            .unwrap();

        // Inputs
        let add_amount_no_dec = 5; // w/o decimals
        let remove_amount_no_dec = 1; // w/o decimals
        let slippage = 0.01;

        // EXECUTE
        let dryrun = false;
        add_remove_liquidity(
            &client,
            &pool_pubkey,
            &pool_info.amm_info,
            add_amount_no_dec,
            remove_amount_no_dec,
            slippage,
            dryrun,
        )
        .unwrap();

        // AFTER
        client.tests_wait_for_confirmation();
        let after_pool_info = client
            .fetch_extended_amm_info(&pool_pubkey)
            .expect("Failed to fetch final pool balances");
        let after_user_lp_amount = client
            .derive_ata_and_fetch_balance(
                &client.user_keypair.pubkey(),
                &pool_info.amm_info.lp_mint,
            )
            .unwrap();

        // EXPECT
        let min_expected_amount =
            ((add_amount_no_dec - remove_amount_no_dec) as f64 * (1.0 - slippage)) as u64;
        assert!(
            after_user_lp_amount - pre_user_lp_amount >= min_expected_amount,
            "User LP amount did not increase by at least the expected amount considering slippage. Expected at least: {}, but got: {}",
            min_expected_amount,
            after_user_lp_amount - pre_user_lp_amount
        );
        assert!(
            after_pool_info.lp_amount - pool_info.lp_amount >= min_expected_amount,
            "Pool LP amount did not increase by at least the expected amount considering slippage. Expected at least: {}, but got: {}",
            min_expected_amount,
            after_pool_info.lp_amount - pool_info.lp_amount
        );
        assert!(
            after_pool_info.coin_vault_balance - pool_info.coin_vault_balance >= min_expected_amount,
            "Coin vault balance did not increase by at least the expected amount considering slippage. Expected at least: {}, but got: {}",
            min_expected_amount,
            after_pool_info.coin_vault_balance - pool_info.coin_vault_balance
        );
        assert!(
            after_pool_info.pc_vault_balance - pool_info.pc_vault_balance >= min_expected_amount,
            "PC vault balance did not increase by at least the expected amount considering slippage. Expected at least: {}, but got: {}",
            min_expected_amount,
            after_pool_info.pc_vault_balance - pool_info.pc_vault_balance
        );
        info!(
            "User LP amount {} -> {}",
            Colour::Red.paint(pre_user_lp_amount.to_string()),
            Colour::Green.paint(after_user_lp_amount.to_string())
        );
        info!(
            "Pool LP amount {} -> {}",
            Colour::Red.paint(pool_info.lp_amount.to_string()),
            Colour::Green.paint(after_pool_info.lp_amount.to_string())
        );
        info!(
            "Coin vault balance {} -> {}",
            Colour::Red.paint(pool_info.coin_vault_balance.to_string()),
            Colour::Green.paint(after_pool_info.coin_vault_balance.to_string())
        );
        info!(
            "PC vault balance {} -> {}",
            Colour::Red.paint(pool_info.pc_vault_balance.to_string()),
            Colour::Green.paint(after_pool_info.pc_vault_balance.to_string())
        );
    }
}
