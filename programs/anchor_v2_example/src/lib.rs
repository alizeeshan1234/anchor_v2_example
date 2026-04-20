pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("B5SqvrzM1UTWxpMi5GqwfAj6L6fjEdD5uANdyBH11pvE");

#[program]
pub mod anchor_v2_example {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn init_token_account(ctx: Context<InitTokenAccount>) -> Result<()> {
        init_token_account::handler(ctx)
    }

    pub fn transfer_tokens(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
        transfer_tokens::handler(ctx, amount)
    }

    pub fn close_token_account(ctx: Context<CloseTokenAccount>) -> Result<()> {
        close_token_account::handler(ctx)
    }

    pub fn init_pool(ctx: Context<InitPool>, pool_name: String) -> Result<()> {
        init_pool::handler(ctx, pool_name)
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        add_liquidity::handler(ctx, amount_a, amount_b)
    }
}
