use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint, TokenAccount};

use crate::state::Pool;

#[derive(Accounts)]
pub struct InitPool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: PDA used only as a CPI signer (mint authority). No data stored.
    #[account(
        seeds = [b"mint_authority".as_ref()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    pub mint_a: Account<'info, Mint>,

    pub mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = signer,
        space = 8 + std::mem::size_of::<Pool>(),
        seeds = [b"pool".as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = signer,
        token::mint = mint_a,
        token::authority = mint_authority,
        seeds = [b"token_vault".as_ref(), pool.key().as_ref(), mint_a.key().as_ref()],
        bump
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        token::mint = mint_b,
        token::authority = mint_authority,
        seeds = [b"token_vault".as_ref(), pool.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 6,
        mint::authority = mint_authority,
        seeds = [b"lp_mint".as_ref(), pool.key().as_ref()],
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<InitPool>, pool_name: String) -> Result<()> {

    let pool = &mut ctx.accounts.pool;

    pool.name = pool_name;
    pool.mint_authority = ctx.accounts.mint_authority.key();
    pool.lp_mint = ctx.accounts.lp_mint.key();
    pool.token_a_mint = ctx.accounts.mint_a.key();
    pool.token_b_mint = ctx.accounts.mint_b.key();
    pool.token_a_vault = ctx.accounts.token_a_vault.key();
    pool.token_b_vault = ctx.accounts.token_b_vault.key();
    pool.token_a_amount = 0;
    pool.token_b_amount = 0;
    pool.total_lp_supply = 0;

    msg!("Pool initialized with name: {}", pool.name);

    Ok(())
}