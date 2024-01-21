use anchor_lang::prelude::*;
use instructions::*;

use anchor_lang::{AnchorDeserialize, AnchorSerialize};

pub mod constants;
pub mod instructions;
pub mod state;

pub mod errors;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod shop22 {

    use super::*;
    pub fn list_v1(ctx: Context<List>, list_input: ListInput) -> Result<()> {
        instructions::list::handler(ctx, list_input)
    }

    pub fn delist_v1<'info>(ctx: Context<'_, '_, '_, 'info, Delist<'info>>) -> Result<()> {
        instructions::delist::handler(ctx)
    }

    pub fn execute_v1<'info>(ctx: Context<'_, '_, '_, 'info, Execute<'info>>) -> Result<()> {
        instructions::execute::handler(ctx)
    }

}
