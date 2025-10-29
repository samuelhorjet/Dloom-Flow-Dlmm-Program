use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Pool {
    pub bump: u8,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub active_bin_id: i32,
    pub bin_step: u16,
    pub fee_rate: u16,
    pub reserves_a: u64,
    pub reserves_b: u64,
}

#[account(zero_copy)]
#[repr(C)]
pub struct Bin {
    pub liquidity: u128,
    pub fee_growth_per_unit_a: u128,
    pub fee_growth_per_unit_b: u128,
}

#[account]
#[derive(Default)]
pub struct Position {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub lower_bin_id: i32,
    pub upper_bin_id: i32,
    pub liquidity: u128,
    pub position_mint: Pubkey,
    pub fee_growth_snapshot_a: u128,
    pub fee_growth_snapshot_b: u128,
}