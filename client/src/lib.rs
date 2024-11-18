#![allow(unused)]
#![allow(dead_code)]
pub mod add_liquidity;
pub mod add_remove_liquidity;
pub mod cache;
pub mod client_helper;
pub mod config;
pub mod fetch_pool;
pub mod remove_liquidity;

use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LiquidityError {
    #[error("Amount must be greater than zero")]
    AmountZero,

    #[error("Slippage must be between 0 and 1")]
    SlippageOutOfRange,

    #[error("Multiplication overflow occurred")]
    MultiplicationOverflow,

    #[error("Failed to generate instructions")]
    InstructionGenerationFailed,

    #[error(transparent)]
    InternalError(#[from] anyhow::Error),

    #[error("No instructions generated")]
    NoInstructions,
}
