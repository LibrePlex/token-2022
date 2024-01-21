use crate::state::{ActivityCounters, ListingIndex};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount},
};
use libreplex_shared::operations::transfer_generic_spl;

use spl_token_2022::{instruction::close_account, ID as TOKEN_2022_PROGRAM_ID};

#[event]
pub struct ExecuteEvent {
    pub id: Pubkey,
}

pub mod sysvar_instructions_program {
    use anchor_lang::declare_id;
    declare_id!("Sysvar1nstructions1111111111111111111111111");
}

pub mod protocol_fee_key {
    use anchor_lang::declare_id;
    declare_id!("11111111111111111111111111111111");
}

pub const MAKER_FEE: u64 = 2_000_000;

pub const TAKER_FEE: u64 = 5_000_000;

#[derive(Accounts)]
pub struct Execute<'info> {
    /// CHECK: checked against listing.lister in macro
    #[account(mut)]
    pub lister: UncheckedAccount<'info>,

    // this might not exist yet
    #[account(mut,
        seeds = [b"activity_counters", lister.key().as_ref()],
        bump
        )]
    pub activity_counters_lister: Account<'info, ActivityCounters>,

    #[account(init_if_needed,
        payer=lister,
        space = ActivityCounters::SIZE,
        seeds = [b"activity_counters", buyer.key().as_ref()],
        bump
        )]
    pub activity_counters_buyer: Account<'info, ActivityCounters>,

    /// CHECK: Checked against ID constraint
    #[account()]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut,
        close = lister,
        seeds=[b"mint_index", mint.key().as_ref()], 
        bump)]
    pub mint_index: Account<'info, ListingIndex>,

    #[account(mut,
        close = lister,
            constraint = lister_index.mint == mint.key(), 
            seeds=[b"lister_index", lister.key().as_ref(), &lister_index.lister_index.to_le_bytes()], 
            bump)]
    pub lister_index: Account<'info, ListingIndex>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    // mint_index holds the escrow
    #[account(mut,
        token::authority = mint_index.key(),
        token::mint = mint.key())]
    pub escrow_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Is allowed to be empty in which case we create it
    #[account(mut)]
    pub buyer_token_account: UncheckedAccount<'info>,

    /// CHECK: done
    #[account(mut,
        constraint = protocol_fee_account.key() == protocol_fee_key::ID)]
    pub protocol_fee_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Checked against ID constraint
    #[account(
        constraint = token_program_2022.key.eq(&TOKEN_2022_PROGRAM_ID)
    )]
    pub token_program_2022: UncheckedAccount<'info>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Execute<'info>>) -> Result<()> {
    let lister = &ctx.accounts.lister;
    let escrow_token_account = &ctx.accounts.escrow_token_account;
    let recipient_token_account = &ctx.accounts.buyer_token_account;
    let mint = &ctx.accounts.mint;
    let mint_index = &ctx.accounts.mint_index;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    let token_program_2022 = &ctx.accounts.token_program_2022;
    let buyer_account_info = &ctx.accounts.buyer.to_account_info().clone();
    let lister_account_info = &ctx.accounts.lister.to_account_info().clone();
    let protocol_fee_account_info = &ctx.accounts.protocol_fee_account.to_account_info().clone();

    let activity_counters_lister = &mut ctx.accounts.activity_counters_lister;
    activity_counters_lister.sold_count += 1;
    activity_counters_lister.total_amount_sold += mint_index.price_in_lamports;

    let activity_counters_buyer = &mut ctx.accounts.activity_counters_buyer;
    activity_counters_buyer.purchase_count += 1;
    activity_counters_buyer.total_amount_bought += mint_index.price_in_lamports;

    let mint_key = &mint.key();
    let auth_seeds = &[b"mint_index", mint_key.as_ref(), &[ctx.bumps.mint_index]];

    // let's handle both 2022 and trad here just in case.
    let mint_token_program = match mint.to_account_info().owner {
        &TOKEN_2022_PROGRAM_ID => token_program_2022.to_account_info(),
        _ => token_program.to_account_info(),
    };

    transfer_generic_spl(
        &mint_token_program,
        &escrow_token_account.to_account_info(),
        &recipient_token_account.to_account_info(),
        &mint_index.to_account_info(),
        &mint.to_account_info(),
        buyer_account_info,
        &associated_token_program.to_account_info(),
        &system_program.to_account_info(),
        Some(&[auth_seeds]),
        buyer_account_info,
        mint.decimals,
        mint_index.amount_to_sell,
    )?;

    system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: buyer_account_info.clone(),
                to: lister_account_info.clone(),
            },
        ),
        mint_index.price_in_lamports,
    )?;

    // transfer part of the taker fee to protocol
    system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: buyer_account_info.clone(),
                to: protocol_fee_account_info.clone(),
            },
        ),
        TAKER_FEE - MAKER_FEE,
    )?;

    // transfer the remainder (the maker fee) to the lister
    system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: buyer_account_info.clone(),
                to: lister.to_account_info(),
            },
        ),
        MAKER_FEE,
    )?;

    // close listing account and refund rent to the lister
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
            lister.to_account_info().clone(),
            mint_index.to_account_info().clone(),
            token_program_2022.to_account_info().clone(),
        ],
        &[auth_seeds],
    )?;

    emit!(ExecuteEvent {
        id: mint_index.key()
    });

    Ok(())
}
