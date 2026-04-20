use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub name: String,
    pub mint_authority: Pubkey,
    pub lp_mint: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub total_lp_supply: u64,
}