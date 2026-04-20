use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::state::Pool;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    pub mint_a: Box<Account<'info, Mint>>,

    pub mint_b: Box<Account<'info, Mint>>,

    /// CHECK: PDA used as LP-mint authority. No data stored.
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

pub fn handler(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, crate::error::ErrorCode::ZeroAmount);

    let pool = &mut ctx.accounts.pool;

    let cpi_program = *ctx.accounts.token_program.key;

    token::transfer(
        CpiContext::new(
            cpi_program,
            Transfer {
                from: ctx.accounts.provider_token_a_account.to_account_info(),
                to: ctx.accounts.token_a_vault.to_account_info(),
                authority: ctx.accounts.provider.to_account_info(),
            },
        ),
        amount_a,
    )?;

    token::transfer(
        CpiContext::new(
            cpi_program,
            Transfer {
                from: ctx.accounts.provider_token_b_account.to_account_info(),
                to: ctx.accounts.token_b_vault.to_account_info(),
                authority: ctx.accounts.provider.to_account_info(),
            },
        ),
        amount_b,
    )?;

    let lp_to_mint: u64 = if pool.total_lp_supply == 0 {
        let product = (amount_a as u128)
            .checked_mul(amount_b as u128)
            .ok_or(crate::error::ErrorCode::MathOverflow)?;
        integer_sqrt(product) as u64
    } else {
        let from_a = (amount_a as u128)
            .checked_mul(pool.total_lp_supply as u128)
            .ok_or(crate::error::ErrorCode::MathOverflow)?
            / pool.token_a_amount as u128;
        let from_b = (amount_b as u128)
            .checked_mul(pool.total_lp_supply as u128)
            .ok_or(crate::error::ErrorCode::MathOverflow)?
            / pool.token_b_amount as u128;
        std::cmp::min(from_a, from_b) as u64
    };

    require!(lp_to_mint > 0, crate::error::ErrorCode::InsufficientLiquidityMinted);

    let bump = ctx.bumps.mint_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[b"mint_authority".as_ref(), &[bump]]];

    token::mint_to(
        CpiContext::new_with_signer(
            cpi_program,
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.provider_lp_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        lp_to_mint,
    )?;

    pool.token_a_amount = pool
        .token_a_amount
        .checked_add(amount_a)
        .ok_or(crate::error::ErrorCode::MathOverflow)?;
    pool.token_b_amount = pool
        .token_b_amount
        .checked_add(amount_b)
        .ok_or(crate::error::ErrorCode::MathOverflow)?;
    pool.total_lp_supply = pool
        .total_lp_supply
        .checked_add(lp_to_mint)
        .ok_or(crate::error::ErrorCode::MathOverflow)?;

    msg!(
        "Added liquidity: a={}, b={}, lp_minted={}",
        amount_a,
        amount_b,
        lp_to_mint
    );

    Ok(())
}

fn integer_sqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}