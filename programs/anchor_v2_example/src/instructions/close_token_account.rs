use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct CloseTokenAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token-account".as_ref(), signer.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CloseTokenAccount>) -> Result<()> {

    msg!("Closing token account: {:?}", ctx.accounts.token_account.key());

    let cpi_accounts = anchor_spl::token::CloseAccount {
        account: ctx.accounts.token_account.to_account_info(),
        destination: ctx.accounts.signer.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_context = CpiContext::new(*ctx.accounts.token_program.key, cpi_accounts);

    anchor_spl::token::close_account(cpi_context)?;

    msg!("Token account closed: {:?}", ctx.accounts.token_account.key());

    Ok(())
}