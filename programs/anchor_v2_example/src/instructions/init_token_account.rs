use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitTokenAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = signer,
        seeds = [b"token-account".as_ref(), signer.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<InitTokenAccount>) -> Result<()> {
    msg!("Token account initialized: {:?}", ctx.accounts.token_account.key());
    Ok(())
}