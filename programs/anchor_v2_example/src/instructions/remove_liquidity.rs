use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::state::Pool;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    pub mint_a: Box<Account<'info, Mint>>,

    pub mint_b: Box<Account<'info, Mint>>,

    /// CHECK: PDA signer for vault transfers. No data stored.
    #[account(
        seeds = [b"mint_authority".as_ref()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"pool".as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        seeds = [b"token_vault".as_ref(), pool.key().as_ref(), mint_a.key().as_ref()],
        bump
    )]
    pub token_a_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"token_vault".as_ref(), pool.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub token_b_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"lp_mint".as_ref(), pool.key().as_ref()],
        bump
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [b"token-account".as_ref(), provider.key().as_ref(), mint_a.key().as_ref()],
        bump
    )]
    pub provider_token_a_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"token-account".as_ref(), provider.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub provider_token_b_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"token-account".as_ref(), provider.key().as_ref(), lp_mint.key().as_ref()],
        bump
    )]
    pub provider_lp_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<RemoveLiquidity>, lp_amount: u64) -> Result<()> {
    require!(lp_amount > 0, crate::error::ErrorCode::ZeroAmount);

    let pool = &mut ctx.accounts.pool;

    require!(
        pool.total_lp_supply > 0,
        crate::error::ErrorCode::InsufficientLiquidityMinted
    );

    // Proportional share:
    //   out_a = reserve_a * lp_amount / total_lp_supply
    //   out_b = reserve_b * lp_amount / total_lp_supply
    let out_a: u64 = ((pool.token_a_amount as u128)
        .checked_mul(lp_amount as u128)
        .ok_or(crate::error::ErrorCode::MathOverflow)?
        / pool.total_lp_supply as u128) as u64;

    let out_b: u64 = ((pool.token_b_amount as u128)
        .checked_mul(lp_amount as u128)
        .ok_or(crate::error::ErrorCode::MathOverflow)?
        / pool.total_lp_supply as u128) as u64;

    require!(
        out_a > 0 && out_b > 0,
        crate::error::ErrorCode::InsufficientLiquidityMinted
    );

    let cpi_program = *ctx.accounts.token_program.key;

    // 1. Burn LP tokens from the provider.
    token::burn(
        CpiContext::new(
            cpi_program,
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.provider_lp_token_account.to_account_info(),
                authority: ctx.accounts.provider.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    // 2. Transfer token A + B from vaults back to provider.
    //    Vaults' authority is the mint_authority PDA, so we need to sign with its seeds.
    let bump = ctx.bumps.mint_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[b"mint_authority".as_ref(), &[bump]]];

    token::transfer(
        CpiContext::new_with_signer(
            cpi_program,
            Transfer {
                from: ctx.accounts.token_a_vault.to_account_info(),
                to: ctx.accounts.provider_token_a_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        out_a,
    )?;

    token::transfer(
        CpiContext::new_with_signer(
            cpi_program,
            Transfer {
                from: ctx.accounts.token_b_vault.to_account_info(),
                to: ctx.accounts.provider_token_b_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        out_b,
    )?;

    // 3. Update pool reserves + supply.
    pool.token_a_amount = pool
        .token_a_amount
        .checked_sub(out_a)
        .ok_or(crate::error::ErrorCode::MathOverflow)?;
    pool.token_b_amount = pool
        .token_b_amount
        .checked_sub(out_b)
        .ok_or(crate::error::ErrorCode::MathOverflow)?;
    pool.total_lp_supply = pool
        .total_lp_supply
        .checked_sub(lp_amount)
        .ok_or(crate::error::ErrorCode::MathOverflow)?;

    msg!(
        "Removed liquidity: lp_burned={}, out_a={}, out_b={}",
        lp_amount,
        out_a,
        out_b
    );

    Ok(())
}
