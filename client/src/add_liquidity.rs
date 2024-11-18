use ansi_term::Colour;
use log::{debug, error, info};
use std::error::Error;
use std::str::FromStr;

use raydium_amm::state::AmmInfo;
use raydium_library::amm::{self, AmmCommands};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;

use crate::cache::{Market, Pool};
use crate::client_helper::ClientHelper;
use crate::fetch_pool;
use crate::*;

// Function to build the transaction instructions for adding liquidity
pub fn build_add_liquidity_ix(
    client: &ClientHelper,
    pool_id: &Pubkey,
    pool_info: &AmmInfo,
    amount: u64,
    slippage: f64,
) -> Result<Vec<Instruction>, LiquidityError> {
    // Basic sanity checks, AmmCommands::Deposit do more checks for us already
    if amount == 0 {
        return Err(LiquidityError::AmountZero);
    }
    // Check that slippage is between 0 and 1
    if !(0.0..=1.0).contains(&slippage) {
        return Err(LiquidityError::SlippageOutOfRange);
    }

    // Build Tx
    let decimals = pool_info.sys_decimal_value;
    let amount_with_decimals = amount
        .checked_mul(decimals)
        .ok_or_else(|| LiquidityError::MultiplicationOverflow)?;

    let cmd = AmmCommands::Deposit {
        pool_id: *pool_id,
        deposit_token_coin: Option::None,
        deposit_token_pc: Option::None,
        recipient_token_lp: Option::None,
        amount_specified: amount_with_decimals,
        another_min_limit: false,
        base_coin: false,
    };
    let result =
        amm::process_amm_commands(cmd, &client.config).map_err(LiquidityError::from)?;
    result.ok_or(LiquidityError::NoInstructions)
}

// Function to add liquidity
pub fn add_liquidity(
    client: &ClientHelper,
    pool_id: &Pubkey,
    pool_info: &AmmInfo,
    amount: u64,
    slippage: f64,
    dryrun: bool,
) -> Result<(), Box<dyn Error>> {
    let instructions = build_add_liquidity_ix(client, pool_id, pool_info, amount, slippage)?;
    client.process_transaction(&instructions, dryrun);
    info!("{}", Colour::Green.paint("Liquidity successfully added"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_liquidity() {
        // SETUP
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

        // ******************
        // INPUTS
        // ******************
        // for the version where you can deposit exact amount of mintA and mintB please refer to tests/add_liquidity.ts
        let amount = 1; // w/o decimals
        let slippage = 0.01;

        // Add liquidity
        let result = add_liquidity(
            &client,
            &pool_pubkey,
            &pool_info.amm_info,
            amount,
            slippage,
            false,
        );
        assert!(
            result.is_ok(),
            "add_liquidity failed with error: {:?}",
            result.unwrap_err()
        );

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
        let min_expected_amount = (amount as f64 * (1.0 - slippage)) as u64;
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
