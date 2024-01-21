use anchor_lang::prelude::*;

use anchor_lang::{AnchorDeserialize, AnchorSerialize};

/*
used for two PDAS - once for [mint] and another for
[lister, <lister index>] */
#[account]
pub struct ListingIndex {
    pub mint: Pubkey,
    pub lister: Pubkey,
    pub price_in_lamports: u64,
    pub creation_time: i64,
    pub lister_index: u8,
    pub amount_to_sell: u64,
    pub bump: u8
}

impl ListingIndex {
    // add a bit of padding. it's not too bad because the lister
    // will get it back when the item sells.
    pub const SIZE: usize = 8 + 32 + 32 + 32 + 8 + 1 + 8 + 8 + 100;
}

const MAX_NAME_LEN: usize = 32;

const MAX_AVATAR_URL_LEN: usize = 256;

#[account]
#[derive(InitSpace)]
pub struct ListerConfig {
    #[max_len(MAX_NAME_LEN)]
    name: String,
    #[max_len(MAX_AVATAR_URL_LEN)]
    avatar_url: String,
    wallet_id: Pubkey,
}

impl ListerConfig {
    // add a bit of padding. it's not too bad because the lister
    // will get it back when the item sells.
    pub const SIZE: usize = 8 + MAX_NAME_LEN + MAX_AVATAR_URL_LEN + 32 + 100; // add some padding
}

#[account]
pub struct ActivityCounters {
    pub actor: Pubkey,
    pub listing_count: u64,
    pub sold_count: u64,
    pub purchase_count: u64,
    pub total_amount_sold: u64,
    pub total_amount_bought: u64
}

impl ActivityCounters {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 100;
}
