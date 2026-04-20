use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    pub receiver: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"token-account".as_ref(), sender.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub sender_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"token-account".as_ref(), receiver.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
    msg!("Transferring {} tokens from {:?} to {:?}",
        amount,
        ctx.accounts.sender_token_account.key(),
        ctx.accounts.recipient_token_account.key()
    );

    let cpi_accounts = anchor_spl::token::Transfer {
        from: ctx.accounts.sender_token_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.sender.to_account_info(),
    };

    let cpi_program = *ctx.accounts.token_program.key;
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    anchor_spl::token::transfer(cpi_ctx, amount)?;

    Ok(())
}