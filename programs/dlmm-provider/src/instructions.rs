// FILE: instructions.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        self, Burn, CloseAccount, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};
use mpl_token_metadata::{
    accounts::{MasterEdition, Metadata},
    instructions::{
        CreateMasterEditionV3Cpi, CreateMasterEditionV3CpiAccounts,
        CreateMasterEditionV3InstructionArgs, CreateMetadataAccountV3Cpi,
        CreateMetadataAccountV3CpiAccounts, CreateMetadataAccountV3InstructionArgs,
    },
    types::DataV2,
    ID as TOKEN_METADATA_ID,
};

use crate::{constants::*, errors::MyError, math, state::*};

//
// Instruction logic
//

// Unchanged modules
pub mod initialize_pool {
    use super::*;
    pub fn handler(
        ctx: Context<InitializePool>,
        bin_step: u16,
        fee_rate: u16,
        initial_bin_id: i32,
    ) -> Result<()> {
        require!(
            is_allowed_parameter(bin_step, fee_rate),
            MyError::InvalidParameters
        );
        let pool = &mut ctx.accounts.pool;
        pool.bump = ctx.bumps.pool;
        pool.bin_step = bin_step;
        pool.fee_rate = fee_rate;
        pool.token_a_mint = ctx.accounts.token_a_mint.key();
        pool.token_b_mint = ctx.accounts.token_b_mint.key();
        pool.token_a_vault = ctx.accounts.token_a_vault.key();
        pool.token_b_vault = ctx.accounts.token_b_vault.key();
        pool.active_bin_id = initial_bin_id;
        pool.reserves_a = 0;
        pool.reserves_b = 0;
        Ok(())
    }
    fn is_allowed_parameter(bin_step: u16, fee_rate: u16) -> bool {
        ALLOWED_PARAMETERS
            .iter()
            .any(|&(bs, fr)| bs == bin_step && fr == fee_rate)
    }
}
pub mod get_price {
    use super::*;
    pub fn handler(ctx: Context<GetPrice>, bin_id: i32) -> Result<u128> {
        let pool = &ctx.accounts.pool;
        math::get_price_at_bin(bin_id, pool.bin_step)
    }
}
pub mod initialize_bin {
    use super::*;
    pub fn handler(ctx: Context<InitializeBin>, _bin_id: i32) -> Result<()> {
        let bin_loader = &mut ctx.accounts.bin;
        let mut bin = bin_loader.load_init()?;
        bin.liquidity = 0;
        bin.fee_growth_per_unit_a = 0;
        bin.fee_growth_per_unit_b = 0;
        Ok(())
    }
}

// NEW `open_position` module
pub mod open_position {
    use super::*;
    pub fn handler(
        ctx: Context<OpenPosition>,
        lower_bin_id: i32,
        upper_bin_id: i32,
    ) -> Result<()> {
        require!(lower_bin_id < upper_bin_id, MyError::InvalidBinRange);
        let bin_step = ctx.accounts.pool.bin_step as i32;
        require!(
            lower_bin_id % bin_step == 0 && upper_bin_id % bin_step == 0,
            MyError::InvalidBinId
        );

        let position = &mut ctx.accounts.position;
        position.pool = ctx.accounts.pool.key();
        position.owner = ctx.accounts.owner.key();
        position.lower_bin_id = lower_bin_id;
        position.upper_bin_id = upper_bin_id;
        position.liquidity = 0;
        position.position_mint = ctx.accounts.position_mint.key();
        position.fee_growth_snapshot_a = 0;
        position.fee_growth_snapshot_b = 0;

        token_interface::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.position_mint.to_account_info(),
                    to: ctx.accounts.user_position_nft_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            1,
        )?;

        CreateMetadataAccountV3Cpi::new(
            &ctx.accounts.token_metadata_program,
            CreateMetadataAccountV3CpiAccounts {
                metadata: &ctx.accounts.metadata_account,
                mint: &ctx.accounts.position_mint.to_account_info(),
                mint_authority: &ctx.accounts.owner.to_account_info(),
                payer: &ctx.accounts.owner.to_account_info(),
                update_authority: (&ctx.accounts.owner.to_account_info(), true),
                system_program: &ctx.accounts.system_program,
                rent: Some(&ctx.accounts.rent.to_account_info()),
            },
            CreateMetadataAccountV3InstructionArgs {
                data: DataV2 {
                    name: "DLMM Position".to_string(),
                    symbol: "DLMMLP".to_string(),
                    uri: "".to_string(),
                    seller_fee_basis_points: 0,
                    creators: None,
                    collection: None,
                    uses: None,
                },
                is_mutable: true,
                collection_details: None,
            },
        )
        .invoke()?;

        CreateMasterEditionV3Cpi::new(
            &ctx.accounts.token_metadata_program,
            CreateMasterEditionV3CpiAccounts {
                edition: &ctx.accounts.master_edition_account,
                mint: &ctx.accounts.position_mint.to_account_info(),
                update_authority: &ctx.accounts.owner.to_account_info(),
                mint_authority: &ctx.accounts.owner.to_account_info(),
                payer: &ctx.accounts.owner.to_account_info(),
                metadata: &ctx.accounts.metadata_account,
                token_program: &ctx.accounts.token_program,
                system_program: &ctx.accounts.system_program,
                rent: Some(&ctx.accounts.rent.to_account_info()),
            },
            CreateMasterEditionV3InstructionArgs { max_supply: Some(0) },
        )
        .invoke()?;

        Ok(())
    }
}

// COMPLETELY REWRITTEN `add_liquidity` module to support chunking
pub mod add_liquidity {
    use super::*;
    pub fn handler<'info>(
        ctx: Context<'_, '_, 'info, 'info, AddLiquidity<'info>>,
        start_bin_id: i32,
        liquidity_per_bin: u128,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let position = &mut ctx.accounts.position;

        require!(liquidity_per_bin > 0, MyError::ZeroLiquidity);

        let bin_step = pool.bin_step as i32;
        let mut current_bin_id = start_bin_id;

        let mut total_required_a: u128 = 0;
        let mut total_required_b: u128 = 0;

        // 1. First Pass: Calculate total required token amounts for this chunk
        for _ in ctx.remaining_accounts.iter() {
            // Security check: ensure the bin we are processing is within the position's declared bounds
            require!(
                current_bin_id >= position.lower_bin_id && current_bin_id <= position.upper_bin_id,
                MyError::InvalidBinRange
            );

            // Use our new math helper for each bin in the chunk
            let (required_a, required_b) = math::calculate_required_for_bin(
                pool.active_bin_id,
                current_bin_id,
                pool.bin_step,
                liquidity_per_bin,
            )?;

            total_required_a = total_required_a.checked_add(required_a).ok_or(MyError::MathOverflow)?;
            total_required_b = total_required_b.checked_add(required_b).ok_or(MyError::MathOverflow)?;

            current_bin_id = current_bin_id.checked_add(bin_step).ok_or(MyError::MathOverflow)?;
        }

        // 2. Transfer the calculated total tokens for this chunk
        if total_required_a > 0 {
            token_interface::transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_a_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.user_token_a_account.to_account_info(),
                        to: ctx.accounts.token_a_vault.to_account_info(),
                        authority: ctx.accounts.owner.to_account_info(),
                        mint: ctx.accounts.token_a_mint.to_account_info(),
                    },
                ),
                total_required_a as u64, // Cast to u64, requires client-side check
                ctx.accounts.token_a_mint.decimals,
            )?;
            pool.reserves_a = pool.reserves_a.checked_add(total_required_a as u64).ok_or(MyError::MathOverflow)?;
        }
        if total_required_b > 0 {
            token_interface::transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_b_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.user_token_b_account.to_account_info(),
                        to: ctx.accounts.token_b_vault.to_account_info(),
                        authority: ctx.accounts.owner.to_account_info(),
                        mint: ctx.accounts.token_b_mint.to_account_info(),
                    },
                ),
                total_required_b as u64, // Cast to u64, requires client-side check
                ctx.accounts.token_b_mint.decimals,
            )?;
            pool.reserves_b = pool.reserves_b.checked_add(total_required_b as u64).ok_or(MyError::MathOverflow)?;
        }

        // 3. Second Pass: Update the liquidity in each bin account
        current_bin_id = start_bin_id; // Reset counter for the second pass
        for bin_account_info in ctx.remaining_accounts.iter() {
            let (expected_bin_pda, _) = Pubkey::find_program_address(
                &[b"bin", pool.key().as_ref(), &current_bin_id.to_le_bytes()],
                ctx.program_id,
            );
            require_keys_eq!(bin_account_info.key(), expected_bin_pda, MyError::InvalidBinAccount);

            let bin_loader = AccountLoader::<'_, Bin>::try_from(bin_account_info)?;
            let mut bin = bin_loader.load_mut()?;
            bin.liquidity = bin.liquidity.checked_add(liquidity_per_bin).ok_or(MyError::MathOverflow)?;

            current_bin_id = current_bin_id.checked_add(bin_step).ok_or(MyError::MathOverflow)?;
        }
        
        // 4. Update the total liquidity in the position account
        let total_liquidity_added_in_chunk = liquidity_per_bin
            .checked_mul(ctx.remaining_accounts.len() as u128)
            .ok_or(MyError::MathOverflow)?;
        
        position.liquidity = position.liquidity.checked_add(total_liquidity_added_in_chunk).ok_or(MyError::MathOverflow)?;

        Ok(())
    }
}


// NEW `modify_liquidity` module that replaces `rebalance_liquidity`
pub mod modify_liquidity {
    use super::*;
    pub fn handler<'info>(
        ctx: Context<'_, '_, 'info, 'info, ModifyLiquidity<'info>>,
        min_surplus_a_out: u64,
        min_surplus_b_out: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let bin_step = pool.bin_step as i32;
        let old_position = &mut ctx.accounts.old_position;
        let new_position = &mut ctx.accounts.new_position;

        let liquidity_to_move = old_position.liquidity;
        require!(liquidity_to_move > 0, MyError::PositionNotEmpty);

        let expected_old_bins_count = ((old_position.upper_bin_id - old_position.lower_bin_id) / bin_step + 1) as usize;
        let expected_new_bins_count = ((new_position.upper_bin_id - new_position.lower_bin_id) / bin_step + 1) as usize;
        require!(ctx.remaining_accounts.len() == expected_old_bins_count + expected_new_bins_count, MyError::InvalidBinCount);

        let (old_bins_info, new_bins_info) = ctx.remaining_accounts.split_at(expected_old_bins_count);

        let (principal_a, principal_b) = math::calculate_claimable_amounts(pool, old_position, liquidity_to_move)?;
        let mut total_fees_a: u128 = 0;
        let mut total_fees_b: u128 = 0;
        let mut current_bin_id = old_position.lower_bin_id;

        let liquidity_per_old_bin = liquidity_to_move.checked_div(expected_old_bins_count as u128).ok_or(MyError::MathOverflow)?;

        for bin_info in old_bins_info.iter() {
            let (expected_bin_pda, _) = Pubkey::find_program_address(&[b"bin", pool.key().as_ref(), &current_bin_id.to_le_bytes()], ctx.program_id);
            require_keys_eq!(bin_info.key(), expected_bin_pda, MyError::InvalidBinAccount);
            let bin_loader = AccountLoader::<'_, Bin>::try_from(bin_info)?;
            let mut bin = bin_loader.load_mut()?;
            let (fees_a, fees_b) = math::calculate_accrued_fees(old_position, &bin);
            total_fees_a = total_fees_a.checked_add(fees_a as u128).ok_or(MyError::MathOverflow)?;
            total_fees_b = total_fees_b.checked_add(fees_b as u128).ok_or(MyError::MathOverflow)?;
            bin.liquidity = bin.liquidity.checked_sub(liquidity_per_old_bin).ok_or(MyError::MathOverflow)?;
            current_bin_id = current_bin_id.checked_add(bin_step).ok_or(MyError::MathOverflow)?;
        }

        let total_claimable_a = principal_a.checked_add(total_fees_a).ok_or(MyError::MathOverflow)?;
        let total_claimable_b = principal_b.checked_add(total_fees_b).ok_or(MyError::MathOverflow)?;
        old_position.liquidity = 0;

        let (required_a, required_b) = math::calculate_required_token_amounts(pool, new_position.lower_bin_id, new_position.upper_bin_id, liquidity_to_move as u64)?;
        let surplus_a = total_claimable_a.checked_sub(required_a as u128).ok_or(MyError::InsufficientLiquidity)?;
        let surplus_b = total_claimable_b.checked_sub(required_b as u128).ok_or(MyError::InsufficientLiquidity)?;
        require!(surplus_a >= min_surplus_a_out as u128 && surplus_b >= min_surplus_b_out as u128, MyError::SlippageExceeded);
        
        let seeds = &[b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref(), &pool.bin_step.to_le_bytes(), &[ctx.accounts.pool.bump]];
        let signer_seeds = &[&seeds[..]];

        if surplus_a > 0 {
            token_interface::transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_a_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.token_a_vault.to_account_info(),
                        to: ctx.accounts.user_token_a_account.to_account_info(),
                        authority: pool.to_account_info(),
                        mint: ctx.accounts.token_a_mint.to_account_info(),
                    },
                    signer_seeds
                ),
                surplus_a as u64,
                ctx.accounts.token_a_mint.decimals,
            )?;
        }
        if surplus_b > 0 {
            token_interface::transfer_checked(
                CpiContext::new_with_signer(
                    ctx.accounts.token_b_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.token_b_vault.to_account_info(),
                        to: ctx.accounts.user_token_b_account.to_account_info(),
                        authority: pool.to_account_info(),
                        mint: ctx.accounts.token_b_mint.to_account_info(),
                    },
                    signer_seeds
                ),
                surplus_b as u64,
                ctx.accounts.token_b_mint.decimals,
            )?;
        }

        let liquidity_per_new_bin = liquidity_to_move.checked_div(expected_new_bins_count as u128).ok_or(MyError::MathOverflow)?;
        let mut snapshot_a: u128 = 0;
        let mut snapshot_b: u128 = 0;
        current_bin_id = new_position.lower_bin_id;
        for bin_info in new_bins_info.iter() {
            let (expected_bin_pda, _) = Pubkey::find_program_address(&[b"bin", pool.key().as_ref(), &current_bin_id.to_le_bytes()], ctx.program_id);
            require_keys_eq!(bin_info.key(), expected_bin_pda, MyError::InvalidBinAccount);
            let bin_loader = AccountLoader::<'_, Bin>::try_from(bin_info)?;
            let mut bin = bin_loader.load_mut()?;
            bin.liquidity = bin.liquidity.checked_add(liquidity_per_new_bin).ok_or(MyError::MathOverflow)?;
            snapshot_a = snapshot_a.max(bin.fee_growth_per_unit_a);
            snapshot_b = snapshot_b.max(bin.fee_growth_per_unit_b);
            current_bin_id = current_bin_id.checked_add(bin_step).ok_or(MyError::MathOverflow)?;
        }
        
        new_position.liquidity = new_position.liquidity.checked_add(liquidity_to_move).ok_or(MyError::MathOverflow)?;
        new_position.fee_growth_snapshot_a = snapshot_a;
        new_position.fee_growth_snapshot_b = snapshot_b;
        
        emit!(LiquidityRebalanced {
            pool: pool.key(),
            owner: ctx.accounts.owner.key(),
            old_position: old_position.key(),
            new_position: new_position.key(),
            liquidity_moved: liquidity_to_move,
            new_lower_bin_id: new_position.lower_bin_id,
            new_upper_bin_id: new_position.upper_bin_id,
        });
        Ok(())
    }
}

// Unchanged instruction modules
pub mod swap {
    use super::*;
    pub fn handler<'info>(
        ctx: Context<'_, '_, 'info, 'info, Swap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        require!(amount_in > 0, MyError::ZeroAmount);
        let pool = &mut ctx.accounts.pool;
        let is_a_to_b = ctx.accounts.user_source_token_account.mint == pool.token_a_mint;
        let (amount_out, final_active_bin_id) = if is_a_to_b {
            math::swap_a_to_b(pool, amount_in, &ctx.remaining_accounts, ctx.program_id)?
        } else {
            math::swap_b_to_a(pool, amount_in, &ctx.remaining_accounts, ctx.program_id)?
        };
        require!(amount_out >= min_amount_out, MyError::SlippageExceeded);
        pool.active_bin_id = final_active_bin_id;
        let seeds = &[b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref(), &pool.bin_step.to_le_bytes(), &[pool.bump]];
        let signer_seeds = &[&seeds[..]];
        let (source_token_program, destination_token_program) = if is_a_to_b { (ctx.accounts.token_a_program.to_account_info(), ctx.accounts.token_b_program.to_account_info()) } else { (ctx.accounts.token_b_program.to_account_info(), ctx.accounts.token_a_program.to_account_info()) };
        let source_mint_info = if is_a_to_b { ctx.accounts.token_a_mint.to_account_info() } else { ctx.accounts.token_b_mint.to_account_info() };
        let destination_mint_info = if is_a_to_b { ctx.accounts.token_b_mint.to_account_info() } else { ctx.accounts.token_a_mint.to_account_info() };
        let source_decimals = if is_a_to_b { ctx.accounts.token_a_mint.decimals } else { ctx.accounts.token_b_mint.decimals };
        let destination_decimals = if is_a_to_b { ctx.accounts.token_b_mint.decimals } else { ctx.accounts.token_a_mint.decimals };
        token_interface::transfer_checked(CpiContext::new(source_token_program, TransferChecked { from: ctx.accounts.user_source_token_account.to_account_info(), to: ctx.accounts.source_vault.to_account_info(), authority: ctx.accounts.trader.to_account_info(), mint: source_mint_info }), amount_in, source_decimals)?;
        token_interface::transfer_checked(CpiContext::new_with_signer(destination_token_program, TransferChecked { from: ctx.accounts.destination_vault.to_account_info(), to: ctx.accounts.user_destination_token_account.to_account_info(), authority: pool.to_account_info(), mint: destination_mint_info }, signer_seeds), amount_out, destination_decimals)?;
        if is_a_to_b {
            pool.reserves_a = pool.reserves_a.checked_add(amount_in).ok_or(MyError::MathOverflow)?;
            pool.reserves_b = pool.reserves_b.checked_sub(amount_out).ok_or(MyError::MathOverflow)?;
        } else {
            pool.reserves_b = pool.reserves_b.checked_add(amount_in).ok_or(MyError::MathOverflow)?;
            pool.reserves_a = pool.reserves_a.checked_sub(amount_out).ok_or(MyError::MathOverflow)?;
        }
        Ok(())
    }
}
pub mod remove_liquidity {
    use super::*;
    pub fn handler<'info>(
        ctx: Context<'_, '_, 'info, 'info, RemoveLiquidity<'info>>,
        liquidity_to_remove: u128,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<()> {
        let position = &mut ctx.accounts.position;
        let pool = &mut ctx.accounts.pool;
        require!(liquidity_to_remove <= position.liquidity, MyError::InsufficientLiquidity);
        let (principal_amount_a, principal_amount_b) = math::calculate_claimable_amounts(pool, position, liquidity_to_remove)?;
        let mut total_fees_a: u64 = 0;
        let mut total_fees_b: u64 = 0;
        let mut final_fee_growth_a: u128 = position.fee_growth_snapshot_a;
        let mut final_fee_growth_b: u128 = position.fee_growth_snapshot_b;
        let bin_step = pool.bin_step as i32;
        let lower_bin_id = position.lower_bin_id;
        let upper_bin_id = position.upper_bin_id;
        let expected_bin_count = ((upper_bin_id - lower_bin_id) / bin_step + 1) as usize;
        require!(ctx.remaining_accounts.len() == expected_bin_count, MyError::InvalidBinCount);
        let mut current_bin_id = lower_bin_id;
        for bin_info in ctx.remaining_accounts.iter() {
            let (expected_pda, _) = Pubkey::find_program_address(&[b"bin", pool.key().as_ref(), &current_bin_id.to_le_bytes()], ctx.program_id);
            require_keys_eq!(bin_info.key(), expected_pda, MyError::InvalidBinAccount);
            let bin_loader = AccountLoader::<'_, Bin>::try_from(bin_info)?;
            let bin = bin_loader.load()?;
            let (fees_a, fees_b) = math::calculate_accrued_fees(position, &bin);
            total_fees_a = total_fees_a.checked_add(fees_a).ok_or(MyError::MathOverflow)?;
            total_fees_b = total_fees_b.checked_add(fees_b).ok_or(MyError::MathOverflow)?;
            final_fee_growth_a = std::cmp::max(final_fee_growth_a, bin.fee_growth_per_unit_a);
            final_fee_growth_b = std::cmp::max(final_fee_growth_b, bin.fee_growth_per_unit_b);
            current_bin_id = current_bin_id.checked_add(bin_step).ok_or(MyError::MathOverflow)?;
        }
        let total_withdrawal_a = (principal_amount_a as u64).checked_add(total_fees_a).ok_or(MyError::MathOverflow)?;
        let total_withdrawal_b = (principal_amount_b as u64).checked_add(total_fees_b).ok_or(MyError::MathOverflow)?;
        require!(total_withdrawal_a >= min_amount_a, MyError::SlippageExceeded);
        require!(total_withdrawal_b >= min_amount_b, MyError::SlippageExceeded);
        if total_withdrawal_a > 0 { pool.reserves_a = pool.reserves_a.checked_sub(total_withdrawal_a).ok_or(MyError::MathOverflow)?; }
        if total_withdrawal_b > 0 { pool.reserves_b = pool.reserves_b.checked_sub(total_withdrawal_b).ok_or(MyError::MathOverflow)?; }
        let seeds = &[b"pool", pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref(), &pool.bin_step.to_le_bytes(), &[pool.bump]];
        let signer_seeds = &[&seeds[..]];
        if total_withdrawal_a > 0 {
            token_interface::transfer_checked(CpiContext::new_with_signer(ctx.accounts.token_a_program.to_account_info(), TransferChecked { from: ctx.accounts.token_a_vault.to_account_info(), to: ctx.accounts.user_token_a_account.to_account_info(), authority: pool.to_account_info(), mint: ctx.accounts.token_a_mint.to_account_info() }, signer_seeds), total_withdrawal_a, ctx.accounts.token_a_mint.decimals)?;
        }
        if total_withdrawal_b > 0 {
            token_interface::transfer_checked(CpiContext::new_with_signer(ctx.accounts.token_b_program.to_account_info(), TransferChecked { from: ctx.accounts.token_b_vault.to_account_info(), to: ctx.accounts.user_token_b_account.to_account_info(), authority: pool.to_account_info(), mint: ctx.accounts.token_b_mint.to_account_info() }, signer_seeds), total_withdrawal_b, ctx.accounts.token_b_mint.decimals)?;
        }
        position.liquidity = position.liquidity.checked_sub(liquidity_to_remove).ok_or(MyError::MathOverflow)?;
        position.fee_growth_snapshot_a = final_fee_growth_a;
        position.fee_growth_snapshot_b = final_fee_growth_b;
        Ok(())
    }
}
pub mod burn_empty_position {
    use super::*;
    pub fn handler(ctx: Context<BurnEmptyPosition>) -> Result<()> {
        token_interface::burn(CpiContext::new(ctx.accounts.token_program.to_account_info(), Burn { mint: ctx.accounts.position_mint.to_account_info(), from: ctx.accounts.user_position_nft_account.to_account_info(), authority: ctx.accounts.owner.to_account_info() }), 1)?;
        token_interface::close_account(CpiContext::new(ctx.accounts.token_program.to_account_info(), CloseAccount { account: ctx.accounts.user_position_nft_account.to_account_info(), destination: ctx.accounts.owner.to_account_info(), authority: ctx.accounts.owner.to_account_info() }))?;
        Ok(())
    }
}

//
// All `#[derive(Accounts)]` structs follow.
//

#[derive(Accounts)]
pub struct GetPrice<'info> {
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
#[instruction(bin_step: u16)]
pub struct InitializePool<'info> {
    pub token_a_mint: InterfaceAccount<'info, Mint>,
    pub token_b_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref(), &bin_step.to_le_bytes()], bump, payer = payer, space = 8 + 153)]
    pub pool: Account<'info, Pool>,
    #[account(init, seeds = [b"vault", pool.key().as_ref(), token_a_mint.key().as_ref()], bump, payer = payer, token::mint = token_a_mint, token::authority = pool, token::token_program = token_a_program)]
    pub token_a_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(init, seeds = [b"vault", pool.key().as_ref(), token_b_mint.key().as_ref()], bump, payer = payer, token::mint = token_b_mint, token::authority = pool, token::token_program = token_b_program)]
    pub token_b_vault: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_a_program: Interface<'info, TokenInterface>,
    pub token_b_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
#[instruction(bin_id: i32)]
pub struct InitializeBin<'info> {
    #[account(init, seeds = [b"bin", pool.key().as_ref(), &bin_id.to_le_bytes()], bump, payer = payer, space = 8 + 16 + 16 + 16)]
    pub bin: AccountLoader<'info, Bin>,
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(constraint = pool.token_a_mint == token_a_mint.key() && pool.token_b_mint == token_b_mint.key() @ MyError::InvalidMint)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(init, seeds = [b"position", position_mint.key().as_ref()], bump, payer = owner, space = 8 + 32 + 32 + 4 + 4 + 16 + 32 + 16 + 16)]
    pub position: Box<Account<'info, Position>>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(init, payer = owner, mint::decimals = 0, mint::authority = owner, mint::freeze_authority = owner)]
    pub position_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(init, payer = owner, associated_token::mint = position_mint, associated_token::authority = owner)]
    pub user_position_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = Metadata::find_pda(&position_mint.key()).0)]
    /// CHECK: We are passing this to the CPI
    pub metadata_account: AccountInfo<'info>,
    #[account(mut, address = MasterEdition::find_pda(&position_mint.key()).0)]
    /// CHECK: We are passing this to the CPI
    pub master_edition_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = TOKEN_METADATA_ID)]
    /// CHECK: We are passing this to the CPI
    pub token_metadata_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, has_one = owner, constraint = position.pool == pool.key() @ MyError::InvalidPool)]
    pub position: Box<Account<'info, Position>>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, token::mint = token_a_mint, has_one = owner)]
    pub user_token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::mint = token_b_mint, has_one = owner)]
    pub user_token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = pool.token_a_vault)]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = pool.token_b_vault)]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_a_program: Interface<'info, TokenInterface>,
    pub token_b_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct ModifyLiquidity<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, has_one = owner @ MyError::Unauthorized, constraint = old_position.pool == pool.key() @ MyError::InvalidPool)]
    pub old_position: Box<Account<'info, Position>>,
    #[account(mut, has_one = owner @ MyError::Unauthorized, constraint = new_position.pool == pool.key() @ MyError::InvalidPool)]
    pub new_position: Box<Account<'info, Position>>,
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, token::mint = pool.token_a_mint)]
    pub user_token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::mint = pool.token_b_mint)]
    pub user_token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = pool.token_a_vault)]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, address = pool.token_b_vault)]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_a_program: Interface<'info, TokenInterface>,
    pub token_b_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, has_one = owner @ MyError::Unauthorized)]
    pub position: Box<Account<'info, Position>>,
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, token::mint = pool.token_a_mint)]
    pub user_token_a_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, token::mint = pool.token_b_mint)]
    pub user_token_b_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = pool.token_a_vault)]
    pub token_a_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = pool.token_b_vault)]
    pub token_b_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_a_program: Interface<'info, TokenInterface>,
    pub token_b_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct BurnEmptyPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner @ MyError::Unauthorized, constraint = position.liquidity == 0 @ MyError::PositionNotEmpty, close = owner)]
    pub position: Box<Account<'info, Position>>,
    #[account(mut, address = position.position_mint)]
    pub position_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, token::mint = position_mint)]
    pub user_position_nft_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub trader: Signer<'info>,
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub user_source_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_destination_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = (source_vault.key() == pool.token_a_vault && destination_vault.key() == pool.token_b_vault) || (source_vault.key() == pool.token_b_vault && destination_vault.key() == pool.token_a_vault) @ MyError::InvalidVault)]
    pub source_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = (source_vault.key() == pool.token_a_vault && destination_vault.key() == pool.token_b_vault) || (source_vault.key() == pool.token_b_vault && destination_vault.key() == pool.token_a_vault) @ MyError::InvalidVault)]
    pub destination_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_a_program: Interface<'info, TokenInterface>,
    pub token_b_program: Interface<'info, TokenInterface>,
}

#[event]
pub struct LiquidityRebalanced {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub old_position: Pubkey,
    pub new_position: Pubkey,
    pub liquidity_moved: u128,
    pub new_lower_bin_id: i32,
    pub new_upper_bin_id: i32,
}