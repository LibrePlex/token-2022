use crate::{
    constants::LISTING,
    state::{ActivityCounters, ListingIndex},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount},
};

use libreplex_shared::operations::transfer_generic_spl;
use spl_token_2022::{instruction::close_account, ID as TOKEN_2022_PROGRAM_ID};

#[event]
pub struct DelistEvent {
    pub id: Pubkey,
}

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
    pub lister: Signer<'info>,

    #[account()]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut,
        close = lister,
        seeds=[b"mint_index", mint.key().as_ref()], 
        bump)]
    pub mint_index: Account<'info, ListingIndex>,

    #[account(mut,
        seeds = [b"activity_counters", lister.key().as_ref()],
        bump
        )]
    pub activity_counters_lister: Account<'info, ActivityCounters>,

    #[account(mut,
        close = lister,
        // needed as otherwise could feed in a random mint 
        // to close an unrelated lister_index
        constraint = lister_index.mint == mint.key(),
        seeds=[b"lister_index", lister.key().as_ref(), &lister_index.lister_index.to_le_bytes()], 
        bump)]
    pub lister_index: Account<'info, ListingIndex>,

    /// CHECK: Checked in logic
    #[account(mut,
        token::authority = mint_index.key(),
        token::mint = mint.key())]
    pub escrow_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Is allowed to be empty in which case we create it
    #[account(mut)]
    pub lister_token_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Checked against ID constraint
    #[account(
        constraint = token_program_2022.key.eq(&TOKEN_2022_PROGRAM_ID)
    )]
    pub token_program_2022: UncheckedAccount<'info>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Delist<'info>>) -> Result<()> {
    let escrow_token_account = &ctx.accounts.escrow_token_account;
    let lister_token_account = &ctx.accounts.lister_token_account;
    let mint = &ctx.accounts.mint;
    let mint_index = &ctx.accounts.mint_index;
    let lister = &ctx.accounts.lister;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    let token_program_2022 = &ctx.accounts.token_program_2022;
    let activity_counters_lister = &mut ctx.accounts.activity_counters_lister;
    activity_counters_lister.actor = lister.key();
    activity_counters_lister.listing_count -= 1;

    // let's handle both 2022 and trad here just in case.
    // let mint_token_program = match escrow_token_account.owner {
    //     &TOKEN_2022_PROGRAM_ID => token_program_2022.to_account_info(),
    //     _ => token_program.to_account_info(),
    // };

    let mint_key = &mint.key();

    let auth_seeds = &[b"mint_index", (mint_key.as_ref()), &[ctx.bumps.mint_index]];

    let lister_account_info = &ctx.accounts.lister.to_account_info().clone();

    let spl_token_program = match mint.to_account_info().owner {
        &TOKEN_2022_PROGRAM_ID => token_program_2022.to_account_info(),
        _ => token_program.to_account_info(),
    };

    transfer_generic_spl(
        &spl_token_program,
        &escrow_token_account.to_account_info(),
        &lister_token_account.to_account_info(),
        &mint_index.to_account_info(),
        &mint.to_account_info(),
        &lister.to_account_info(),
        &associated_token_program.to_account_info(),
        &system_program.to_account_info(),
        Some(&[auth_seeds]),
        &lister.to_account_info(),
        mint.decimals,
        mint_index.amount_to_sell,
    )?;

    solana_program::program::invoke_signed(
        &close_account(
            &token_program_2022.key(),
            &escrow_token_account.key(),
            &mint_index.lister,
            &mint_index.key(),
            &[],
        )?,
        &[
            escrow_token_account.to_account_info().clone(),
            lister_account_info.clone(),
            mint_index.to_account_info().clone(),
            token_program_2022.to_account_info().clone(),
        ],
        &[auth_seeds],
    )?;

    emit!(DelistEvent {
        id: mint_index.key()
    });

    Ok(())
}
