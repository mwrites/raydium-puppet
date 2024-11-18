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

pub fn build_remove_liquidity_ix(
    client: &ClientHelper,
    pool_id: &Pubkey,
    pool_info: &AmmInfo,
    input_lp_amount: u64,
    slippage_limit: f64,
) -> Result<Vec<Instruction>, LiquidityError> {
    // Basic sanity checks
    if input_lp_amount == 0 {
        return Err(LiquidityError::AmountZero);
    }
    // Check that slippage limit is between 0 and 1
    if !(0.0..=1.0).contains(&slippage_limit) {
        return Err(LiquidityError::SlippageOutOfRange);
    }

    // Build Tx
    let decimals = pool_info.sys_decimal_value;
    let amount_with_decimals = input_lp_amount
        .checked_mul(decimals)
        .ok_or_else(|| LiquidityError::MultiplicationOverflow)?;
    let cmd = AmmCommands::Withdraw {
        pool_id: *pool_id,
        /// The specified lp token of the user withdraw.
        /// If none is given, the account will be ATA account.
        withdraw_token_lp: None,
        /// The specified token coin of the user will receive.
        /// If none is given, the account will be ATA account.
        recipient_token_coin: None,
        /// The specified token pc of the user will receive.
        /// If none is given, the account will be ATA account.
        recipient_token_pc: None,
        /// The amount of liquidity to burn.
        input_lp_amount: amount_with_decimals,
        /// The amount of both tokens to be calculated though `input_lp_amount` may be less than expected due to price fluctuations.
        /// It's necessary to add an optional parameter to limit the minimum amount of the tokens.
        slippage_limit: slippage_limit > 0.0,
    };
    let result =
        amm::process_amm_commands(cmd, &client.config).map_err(LiquidityError::from)?;
    result.ok_or(LiquidityError::NoInstructions)
}

// Function to remove liquidity
pub fn remove_liquidity(
    client: &ClientHelper,
    pool_id: &Pubkey,
    pool_info: &AmmInfo,
    input_lp_amount: u64,
    slippage_limit: f64,
    dryrun: bool,
) -> Result<(), Box<dyn Error>> {
    let instructions =
        build_remove_liquidity_ix(client, pool_id, pool_info, input_lp_amount, slippage_limit)?;
    client.process_transaction(&instructions, dryrun);
    info!("{}", Colour::Green.paint("Liquidity successfully removed"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_liquidity() {
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
        let result = remove_liquidity(
            &client,
            &pool_pubkey,
            &pool_info.amm_info,
            amount,
            slippage,
            false,
        );
        assert!(
            result.is_ok(),
            "remove_liquidity failed with error: {:?}",
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
        println!("min_expected_amount: {}", min_expected_amount);
        assert!(
            after_user_lp_amount.abs_diff(pre_user_lp_amount) >= min_expected_amount,
            "User LP amount did not decrease by at least the expected amount considering slippage. Expected at least: {}, but got: {}",
            min_expected_amount,
            after_user_lp_amount.abs_diff(pre_user_lp_amount)
        );
        assert!(
            after_pool_info.lp_amount.abs_diff(pool_info.lp_amount) >= min_expected_amount,
            "Pool LP amount did not decrease by at least the expected amount considering slippage. Expected at least: {}, but got: {}",
            min_expected_amount,
            after_pool_info.lp_amount.abs_diff(pool_info.lp_amount)
        );
        assert!(
            after_pool_info.coin_vault_balance.abs_diff(pool_info.coin_vault_balance) >= min_expected_amount,
            "Coin vault balance did not decrease by at least the expected amount considering slippage. Expected at least: {}, but got: {}",
            min_expected_amount,
            after_pool_info.coin_vault_balance.abs_diff(pool_info.coin_vault_balance)
        );
        assert!(
            after_pool_info.pc_vault_balance.abs_diff(pool_info.pc_vault_balance) >= min_expected_amount,
            "PC vault balance did not decrease by at least the expected amount considering slippage. Expected at least: {}, but got: {}",
            min_expected_amount,
            after_pool_info.pc_vault_balance.abs_diff(pool_info.pc_vault_balance)
        );
        info!(
            "User LP amount {} -> {}",
            Colour::Green.paint(pre_user_lp_amount.to_string()),
            Colour::Red.paint(after_user_lp_amount.to_string())
        );
        info!(
            "Pool LP amount {} -> {}",
            Colour::Green.paint(pool_info.lp_amount.to_string()),
            Colour::Red.paint(after_pool_info.lp_amount.to_string())
        );
        info!(
            "Coin vault balance {} -> {}",
            Colour::Green.paint(pool_info.coin_vault_balance.to_string()),
            Colour::Red.paint(after_pool_info.coin_vault_balance.to_string())
        );
        info!(
            "PC vault balance {} -> {}",
            Colour::Green.paint(pool_info.pc_vault_balance.to_string()),
            Colour::Red.paint(after_pool_info.pc_vault_balance.to_string())
        );
    }
}
