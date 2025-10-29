// FILE: errors.rs

use anchor_lang::prelude::*;

#[error_code]
pub enum MyError {
    #[msg("The provided fee and bin_step parameters are not on the whitelist.")]
    InvalidParameters,
    #[msg("The mint addresses are not in the correct canonical order. Token A must be less than Token B.")]
    InvalidMintOrder,
    #[msg("The provided mint does not match the pool's mint.")]
    InvalidMint,
    #[msg("The lower bin ID must be less than the upper bin ID.")]
    InvalidBinRange,
    #[msg("Liquidity to deposit must be greater than zero.")]
    ZeroLiquidity,
    #[msg("The market price moved unfavorably, exceeding your slippage tolerance.")]
    SlippageExceeded,
    #[msg("The signer is not the authorized owner of this position.")]
    Unauthorized,
    #[msg("The amount of liquidity to remove exceeds the amount in the position.")]
    InsufficientLiquidity,
    #[msg("Cannot operate on a position that has no liquidity.")]
    PositionNotEmpty,
    #[msg("Input amount for a swap must be greater than zero.")]
    ZeroAmount,
    #[msg("The provided vault account does not match the pool's vault.")]
    InvalidVault,
    #[msg("The provided bin IDs must be a multiple of the pool's bin_step.")]
    InvalidBinId,
    #[msg("The specified bin range is wider than the allowed maximum.")]
    RangeTooWide,
    #[msg("Math operation overflowed or underflowed.")]
    MathOverflow,
    #[msg("The provided bin step value is invalid (e.g., zero).")]
    InvalidBinStep,
    #[msg("Not enough liquidity in the pool to complete the swap.")]
    InsufficientLiquidityForSwap,
    #[msg("The number of bins provided does not match the position's range.")]
    InvalidBinCount,
    #[msg("A provided bin account does not have the expected address.")]
    InvalidBinAccount,
    #[msg("The provided position account does not belong to the specified pool.")]
    InvalidPool,
}