// FILE: math.rs

use anchor_lang::prelude::*;
use crate::{
    constants::{BASIS_POINT_MAX, PRECISION},
    errors::MyError,
    state::{Bin, Pool, Position},
};

pub fn get_price_at_bin(bin_id: i32, bin_step: u16) -> Result<u128> {
    if bin_step == 0 {
        return err!(MyError::InvalidBinStep);
    }
    let basis_point_step = bin_step as u128;
    let power = bin_id.unsigned_abs() as u128;
    let base = BASIS_POINT_MAX
        .checked_add(basis_point_step)
        .ok_or(MyError::MathOverflow)?;

    let price_ratio = power_fp(base, power)?;

    if bin_id >= 0 {
        Ok(price_ratio)
    } else {
        PRECISION
            .checked_mul(PRECISION)
            .ok_or(MyError::MathOverflow)?
            .checked_div(price_ratio)
            .ok_or(MyError::MathOverflow.into())
    }
}

fn power_fp(base: u128, exp: u128) -> Result<u128> {
    let mut res = PRECISION;
    let mut base_fp = base;
    let mut exp_rem = exp;

    if exp == 0 {
        return Ok(PRECISION);
    }

    while exp_rem > 0 {
        if exp_rem % 2 == 1 {
            res = res
                .checked_mul(base_fp)
                .ok_or(MyError::MathOverflow)?
                .checked_div(BASIS_POINT_MAX)
                .ok_or(MyError::MathOverflow)?;
        }
        base_fp = base_fp
            .checked_mul(base_fp)
            .ok_or(MyError::MathOverflow)?
            .checked_div(BASIS_POINT_MAX)
            .ok_or(MyError::MathOverflow)?;
        exp_rem /= 2;
    }
    Ok(res)
}

pub fn calculate_required_for_bin(
    active_bin_id: i32,
    bin_id: i32,
    bin_step: u16,
    liquidity_amount: u128,
) -> Result<(u128, u128)> {
    let mut required_a: u128 = 0;
    let mut required_b: u128 = 0;

    if bin_id > active_bin_id {
        // Bins above the active price are denominated in Token A
        required_a = liquidity_amount;
    } else if bin_id < active_bin_id {
        // Bins below the active price are denominated in Token B
        let price = get_price_at_bin(bin_id, bin_step)?;
        required_b = liquidity_amount
            .checked_mul(price)
            .ok_or(MyError::MathOverflow)?
            .checked_div(PRECISION)
            .ok_or(MyError::MathOverflow)?;
    } else {
        // The active bin can contain both tokens
        let price = get_price_at_bin(bin_id, bin_step)?;
        required_a = liquidity_amount;
        required_b = liquidity_amount
            .checked_mul(price)
            .ok_or(MyError::MathOverflow)?
            .checked_div(PRECISION)
            .ok_or(MyError::MathOverflow)?;
    }

    Ok((required_a, required_b))
}


pub fn calculate_required_token_amounts(
    pool: &Account<Pool>,
    lower_bin_id: i32,
    upper_bin_id: i32,
    amount_to_deposit: u64,
) -> Result<(u64, u64)> {
    let mut amount_a: u128 = 0;
    let mut amount_b: u128 = 0;

    let num_bins = ((upper_bin_id - lower_bin_id) as u128 / pool.bin_step as u128)
        .checked_add(1)
        .ok_or(MyError::MathOverflow)?;
    if num_bins == 0 {
        return Ok((0, 0));
    }

    let liquidity_per_bin = (amount_to_deposit as u128)
        .checked_div(num_bins)
        .ok_or(MyError::MathOverflow)?;

    for bin_id in (lower_bin_id..=upper_bin_id).step_by(pool.bin_step as usize) {
        if bin_id > pool.active_bin_id {
            // Above active price: only token A is required
            amount_a = amount_a
                .checked_add(liquidity_per_bin)
                .ok_or(MyError::MathOverflow)?;
        } else if bin_id < pool.active_bin_id {
            // Below active price: only token B is required
            let price = get_price_at_bin(bin_id, pool.bin_step)?;
            let amount_b_in_bin = liquidity_per_bin
                .checked_mul(price)
                .ok_or(MyError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(MyError::MathOverflow)?;
            amount_b = amount_b
                .checked_add(amount_b_in_bin)
                .ok_or(MyError::MathOverflow)?;
        } else {
            // At active price: both tokens are required
            let price = get_price_at_bin(bin_id, pool.bin_step)?;
            let amount_b_in_bin = liquidity_per_bin
                .checked_mul(price)
                .ok_or(MyError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(MyError::MathOverflow)?;
            amount_a = amount_a
                .checked_add(liquidity_per_bin)
                .ok_or(MyError::MathOverflow)?;
            amount_b = amount_b
                .checked_add(amount_b_in_bin)
                .ok_or(MyError::MathOverflow)?;
        }
    }
    Ok((amount_a as u64, amount_b as u64))
}

pub fn calculate_claimable_amounts(
    pool: &Account<Pool>,
    position: &Account<Position>,
    liquidity_to_remove: u128,
) -> Result<(u128, u128)> {
    let total_bins_in_pos = ((position.upper_bin_id - position.lower_bin_id) as u128
        / pool.bin_step as u128)
        .checked_add(1)
        .ok_or(MyError::MathOverflow)?;
    if total_bins_in_pos == 0 {
        return Ok((0, 0));
    }

    let liquidity_per_bin = liquidity_to_remove
        .checked_div(total_bins_in_pos)
        .ok_or(MyError::MathOverflow)?;

    let mut amount_a: u128 = 0;
    let mut amount_b: u128 = 0;

    for bin_id in (position.lower_bin_id..=position.upper_bin_id).step_by(pool.bin_step as usize) {
        if bin_id > pool.active_bin_id {
            amount_a = amount_a
                .checked_add(liquidity_per_bin)
                .ok_or(MyError::MathOverflow)?;
        } else if bin_id < pool.active_bin_id {
            let price = get_price_at_bin(bin_id, pool.bin_step)?;
            let amount_b_in_bin = liquidity_per_bin
                .checked_mul(price)
                .ok_or(MyError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(MyError::MathOverflow)?;
            amount_b = amount_b
                .checked_add(amount_b_in_bin)
                .ok_or(MyError::MathOverflow)?;
        } else {
            let price = get_price_at_bin(bin_id, pool.bin_step)?;
            let amount_b_in_bin = liquidity_per_bin
                .checked_mul(price)
                .ok_or(MyError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(MyError::MathOverflow)?;
            amount_a = amount_a
                .checked_add(liquidity_per_bin)
                .ok_or(MyError::MathOverflow)?;
            amount_b = amount_b
                .checked_add(amount_b_in_bin)
                .ok_or(MyError::MathOverflow)?;
        }
    }

    Ok((amount_a, amount_b))
}

pub fn calculate_accrued_fees(position: &Account<Position>, bin: &Bin) -> (u64, u64) {
    let fee_growth_a = bin
        .fee_growth_per_unit_a
        .checked_sub(position.fee_growth_snapshot_a)
        .unwrap_or(0);
    let fee_growth_b = bin
        .fee_growth_per_unit_b
        .checked_sub(position.fee_growth_snapshot_b)
        .unwrap_or(0);

    let fees_a = fee_growth_a
        .checked_mul(position.liquidity)
        .unwrap_or(0)
        .checked_div(PRECISION)
        .unwrap_or(0) as u64;
    let fees_b = fee_growth_b
        .checked_mul(position.liquidity)
        .unwrap_or(0)
        .checked_div(PRECISION)
        .unwrap_or(0) as u64;

    (fees_a, fees_b)
}

pub fn swap_a_to_b<'info>(
    pool: &Account<'info, Pool>,
    amount_in: u64,
    bin_accounts: &'info [AccountInfo<'info>],
    program_id: &Pubkey,
) -> Result<(u64, i32)> {
    let mut amount_remaining_in = amount_in as u128;
    let mut total_amount_out: u128 = 0;
    let mut current_bin_id = pool.active_bin_id;
    let mut bin_accounts_iter = bin_accounts.iter();

    while amount_remaining_in > 0 {
        let bin_info = bin_accounts_iter
            .next()
            .ok_or(MyError::InsufficientLiquidityForSwap)?;

        let (expected_pda, _) = Pubkey::find_program_address(
            &[
                b"bin",
                pool.key().as_ref(),
                &current_bin_id.to_le_bytes(),
            ],
            program_id,
        );
        require_keys_eq!(bin_info.key(), expected_pda, MyError::InvalidBinAccount);

        let bin_loader = AccountLoader::<'_, Bin>::try_from(bin_info)?;
        let mut bin = bin_loader.load_mut()?;

        let price = get_price_at_bin(current_bin_id, pool.bin_step)?;
        let available_amount_b = bin
            .liquidity
            .checked_mul(price)
            .ok_or(MyError::MathOverflow)?
            .checked_div(PRECISION)
            .ok_or(MyError::MathOverflow)?;

        if available_amount_b > 0 {
            let fee = amount_remaining_in
                .checked_mul(pool.fee_rate as u128)
                .ok_or(MyError::MathOverflow)?
                .checked_div(BASIS_POINT_MAX)
                .ok_or(MyError::MathOverflow)?;
            let amount_in_after_fee = amount_remaining_in
                .checked_sub(fee)
                .ok_or(MyError::MathOverflow)?;

            let amount_out_from_bin = std::cmp::min(
                amount_in_after_fee
                    .checked_mul(price)
                    .ok_or(MyError::MathOverflow)?
                    .checked_div(PRECISION)
                    .ok_or(MyError::MathOverflow)?,
                available_amount_b,
            );

            let amount_in_consumed = if amount_out_from_bin == available_amount_b {
                // This logic seems incorrect, let's stick to the math
                amount_out_from_bin
                    .checked_mul(PRECISION)
                    .ok_or(MyError::MathOverflow)?
                    .checked_div(price)
                    .ok_or(MyError::MathOverflow)?
            } else {
                amount_out_from_bin
                    .checked_mul(PRECISION)
                    .ok_or(MyError::MathOverflow)?
                    .checked_div(price)
                    .ok_or(MyError::MathOverflow)?
            };

             let actual_amount_in_with_fee = amount_in_consumed
                .checked_mul(BASIS_POINT_MAX)
                .ok_or(MyError::MathOverflow)?
                .checked_div(BASIS_POINT_MAX.checked_sub(pool.fee_rate as u128).ok_or(MyError::MathOverflow)?)
                .ok_or(MyError::MathOverflow)?;


            if bin.liquidity > 0 {
                let fee_to_add = actual_amount_in_with_fee.checked_sub(amount_in_consumed).ok_or(MyError::MathOverflow)?;
                let fee_growth_update = fee_to_add
                    .checked_mul(PRECISION)
                    .ok_or(MyError::MathOverflow)?
                    .checked_div(bin.liquidity)
                    .ok_or(MyError::MathOverflow)?;
                bin.fee_growth_per_unit_b = bin
                    .fee_growth_per_unit_b
                    .checked_add(fee_growth_update)
                    .ok_or(MyError::MathOverflow)?;
            }

            bin.liquidity = bin
                .liquidity
                .checked_add(amount_in_consumed)
                .ok_or(MyError::MathOverflow)?;
            total_amount_out = total_amount_out
                .checked_add(amount_out_from_bin)
                .ok_or(MyError::MathOverflow)?;
            amount_remaining_in = amount_remaining_in
                .checked_sub(actual_amount_in_with_fee)
                .ok_or(MyError::MathOverflow)?;
        }

        current_bin_id = current_bin_id
            .checked_sub(pool.bin_step as i32)
            .ok_or(MyError::MathOverflow)?;
    }
    
    Ok((total_amount_out as u64, current_bin_id))
}

pub fn swap_b_to_a<'info>(
    pool: &Account<'info, Pool>,
    amount_in: u64,
    bin_accounts: &'info [AccountInfo<'info>],
    program_id: &Pubkey,
) -> Result<(u64, i32)> {
    let mut amount_remaining_in = amount_in as u128;
    let mut total_amount_out: u128 = 0;
    let mut current_bin_id = pool.active_bin_id;
    let mut bin_accounts_iter = bin_accounts.iter();

    while amount_remaining_in > 0 {
        let bin_info = bin_accounts_iter
            .next()
            .ok_or(MyError::InsufficientLiquidityForSwap)?;

        let (expected_pda, _) = Pubkey::find_program_address(
            &[
                b"bin",
                pool.key().as_ref(),
                &current_bin_id.to_le_bytes(),
            ],
            program_id,
        );
        require_keys_eq!(bin_info.key(), expected_pda, MyError::InvalidBinAccount);

        let bin_loader = AccountLoader::<'_, Bin>::try_from(bin_info)?;
        let mut bin = bin_loader.load_mut()?;
        
        let available_amount_a = bin.liquidity;

        if available_amount_a > 0 {
            let price = get_price_at_bin(current_bin_id, pool.bin_step)?;
            let fee = amount_remaining_in
                .checked_mul(pool.fee_rate as u128)
                .ok_or(MyError::MathOverflow)?
                .checked_div(BASIS_POINT_MAX)
                .ok_or(MyError::MathOverflow)?;
            let amount_in_after_fee = amount_remaining_in
                .checked_sub(fee)
                .ok_or(MyError::MathOverflow)?;

            let amount_out_from_bin = std::cmp::min(
                amount_in_after_fee
                    .checked_mul(PRECISION)
                    .ok_or(MyError::MathOverflow)?
                    .checked_div(price)
                    .ok_or(MyError::MathOverflow)?,
                available_amount_a,
            );

            let amount_in_consumed = amount_out_from_bin
                .checked_mul(price)
                .ok_or(MyError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(MyError::MathOverflow)?;
            
            let actual_amount_in_with_fee = amount_in_consumed
                .checked_mul(BASIS_POINT_MAX)
                .ok_or(MyError::MathOverflow)?
                .checked_div(BASIS_POINT_MAX.checked_sub(pool.fee_rate as u128).ok_or(MyError::MathOverflow)?)
                .ok_or(MyError::MathOverflow)?;

            if bin.liquidity > 0 {
                 let fee_to_add = actual_amount_in_with_fee.checked_sub(amount_in_consumed).ok_or(MyError::MathOverflow)?;
                let fee_growth_update = fee_to_add
                    .checked_mul(PRECISION)
                    .ok_or(MyError::MathOverflow)?
                    .checked_div(bin.liquidity)
                    .ok_or(MyError::MathOverflow)?;
                bin.fee_growth_per_unit_a = bin
                    .fee_growth_per_unit_a
                    .checked_add(fee_growth_update)
                    .ok_or(MyError::MathOverflow)?;
            }

            bin.liquidity = bin
                .liquidity
                .checked_sub(amount_out_from_bin) // <--- CORRECTED HERE
                .ok_or(MyError::MathOverflow)?;
            total_amount_out = total_amount_out
                .checked_add(amount_out_from_bin)
                .ok_or(MyError::MathOverflow)?;
            amount_remaining_in = amount_remaining_in
                .checked_sub(actual_amount_in_with_fee)
                .ok_or(MyError::MathOverflow)?;
        }

        current_bin_id = current_bin_id
            .checked_add(pool.bin_step as i32)
            .ok_or(MyError::MathOverflow)?;
    }

    Ok((total_amount_out as u64, current_bin_id))
}