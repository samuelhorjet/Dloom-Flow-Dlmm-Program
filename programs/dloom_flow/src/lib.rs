// FILE: lib.rs

use anchor_lang::prelude::*;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;

declare_id!("6fG9BGsHZjsV9Rie5fm2r9J9cfsqBG8kgTAicbHQtCwH"); // Replace with your actual Program ID

#[program]
pub mod dloom_flow {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        bin_step: u16,
        fee_rate: u16,
        initial_bin_id: i32,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, bin_step, fee_rate, initial_bin_id)
    }

    pub fn get_price(ctx: Context<GetPrice>, bin_id: i32) -> Result<u128> {
        instructions::get_price::handler(ctx, bin_id)
    }

    pub fn initialize_bin(ctx: Context<InitializeBin>, bin_id: i32) -> Result<()> {
        instructions::initialize_bin::handler(ctx, bin_id)
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        lower_bin_id: i32,
        upper_bin_id: i32,
    ) -> Result<()> {
        instructions::open_position::handler(ctx, lower_bin_id, upper_bin_id)
    }

    // UPDATED `add_liquidity` function signature
    pub fn add_liquidity<'info>(
        ctx: Context<'_, '_, 'info, 'info, AddLiquidity<'info>>,
        start_bin_id: i32,
        liquidity_per_bin: u128,
    ) -> Result<()> {
        instructions::add_liquidity::handler(ctx, start_bin_id, liquidity_per_bin)
    }

    pub fn swap<'info>(
        ctx: Context<'_, '_, 'info, 'info, Swap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::swap::handler(ctx, amount_in, min_amount_out)
    }

    pub fn remove_liquidity<'info>(
        ctx: Context<'_, '_, 'info, 'info, RemoveLiquidity<'info>>,
        liquidity_to_remove: u128,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<()> {
        instructions::remove_liquidity::handler(ctx, liquidity_to_remove, min_amount_a, min_amount_b)
    }

    pub fn burn_empty_position(ctx: Context<BurnEmptyPosition>) -> Result<()> {
        instructions::burn_empty_position::handler(ctx)
    }

    // REPLACED `rebalance_liquidity`
    pub fn modify_liquidity<'info>(
        ctx: Context<'_, '_, 'info, 'info, ModifyLiquidity<'info>>,
        min_surplus_a_out: u64,
        min_surplus_b_out: u64,
    ) -> Result<()> {
        instructions::modify_liquidity::handler(
            ctx,
            min_surplus_a_out,
            min_surplus_b_out,
        )
    }
}