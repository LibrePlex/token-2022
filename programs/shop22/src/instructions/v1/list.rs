use crate::state::{ListingIndex, ActivityCounters};
use anchor_lang::{
    accounts::interface_account::InterfaceAccount,
    context::CpiContext,
    prelude::{
        borsh, error, require_keys_neq, Account, AccountInfo, Accounts, Context, Program, Pubkey,
        Rent, Result as AnchorResult, Signer, SolanaSysvar, System, UncheckedAccount,
    },
    system_program, AnchorDeserialize, AnchorSerialize, Key, ToAccountInfo,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token, token_2022,
    token_interface::{Mint, TokenAccount},
};
use libreplex_shared::operations::transfer_generic_spl;
use solana_program::clock::Clock;

use super::execute::{protocol_fee_key, MAKER_FEE};

#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub struct ListInput {
    pub amount_to_sell: u64,
    pub listing_price_lamports: u64,
    pub lister_index: u8,
}

#[derive(Accounts)]
#[instruction(input: ListInput)]
pub struct List<'info> {
    #[account(mut)]
    pub lister: Signer<'info>,

    #[account(init_if_needed,
     payer=lister,
     space = ActivityCounters::SIZE,
    seeds = [b"activity_counters", lister.key().as_ref()],
        bump
    )]
    pub activity_counters_lister: Account<'info, ActivityCounters>,

    /// CHECK: Checked against ID constraint
    #[account()]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(init,
    payer=lister,
        space = ListingIndex::SIZE,
        seeds=[b"mint_index", mint.key().as_ref()], 
        bump)]
    pub mint_index: Account<'info, ListingIndex>,

    #[account(init,
        payer=lister,
            space = ListingIndex::SIZE,
            seeds=[b"lister_index", lister.key().as_ref(), &input.lister_index.to_le_bytes()], 
            bump)]
    pub lister_index: Account<'info, ListingIndex>,

    /// CHECK: Will need to be created, hence unchecked
    #[account(mut)]
    pub escrow_token_account: UncheckedAccount<'info>,

    /// CHECK: Is allowed to be empty in which case we create it

    #[account(mut,
        token::authority = lister.key(),
        token::mint = mint.key())]
    pub lister_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Checked in logic
    #[account(mut,
        constraint = protocol_fee_account.key() == protocol_fee_key::ID)]
    pub protocol_fee_account: UncheckedAccount<'info>,

    // Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Checked against ID constraint
    #[account(
        constraint = token_program.key.eq(&token_2022::ID) || token_program.key.eq(&token::ID)
    )]
    pub token_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<List>, input: ListInput) -> AnchorResult<()> {
    let lister_index = &mut ctx.accounts.lister_index;
    let mint_index: &mut Account<'_, ListingIndex> = &mut ctx.accounts.mint_index;

    let lister = &mut ctx.accounts.lister;
    let lister_token_account = &mut ctx.accounts.lister_token_account;
    let mint = &mut ctx.accounts.mint;
    let escrow_token_account = &mut ctx.accounts.escrow_token_account;
    let associated_token_program = &mut ctx.accounts.associated_token_program;
    let system_program = &mut ctx.accounts.system_program;
    let token_program = &mut ctx.accounts.token_program;
    let protocol_fee_account = &mut ctx.accounts.protocol_fee_account;
    let activity_counters_lister = &mut ctx.accounts.activity_counters_lister;
    activity_counters_lister.actor = lister.key();
    activity_counters_lister.listing_count += 1;
    // the maker fee) to the protocol (refunded when item sells)
    system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: lister.to_account_info(),
                to: protocol_fee_account.to_account_info(),
            },
        ),
        MAKER_FEE,
    )?;

    let clock = Clock::get()?;

    let creation_time = clock.unix_timestamp;

    populate_index(
        lister_index,
        mint.key(),
        lister.key(),
        input.listing_price_lamports,
        ctx.bumps.lister_index,
        creation_time,
        input.lister_index,
        input.amount_to_sell,
    );

    populate_index(
        mint_index,
        mint.key(),
        lister.key(),
        input.listing_price_lamports,
        ctx.bumps.lister_index,
        creation_time,
        input.lister_index,
        input.amount_to_sell,
    );

    transfer_generic_spl(
        &token_program.to_account_info(),
        &lister_token_account.to_account_info(),
        &escrow_token_account.to_account_info(),
        &lister.to_account_info(),
        &mint.to_account_info(),
        // use mint index as the escrow wallet. easier to track
        &mint_index.to_account_info(),
        &associated_token_program.to_account_info(),
        &system_program.to_account_info(),
        None,
        &lister.to_account_info(),
        mint.decimals,
        1,
    )?;

    Ok(())
}

fn populate_index(
    index: &mut ListingIndex,
    mint: Pubkey,
    lister: Pubkey,
    price_in_lamports: u64,
    bump: u8,
    creation_time: i64,
    lister_index: u8,
    amount_to_sell: u64,
) {
    index.mint = mint.key();
    index.lister = lister.key();
    index.price_in_lamports = price_in_lamports;
    index.creation_time = creation_time;
    index.lister_index = lister_index;
    index.amount_to_sell = amount_to_sell;
    index.bump = bump;
}
