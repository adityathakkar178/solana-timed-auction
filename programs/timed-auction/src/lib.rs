#![allow(clippy::result_large_err)]

use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        metadata::{
            create_master_edition_v3, create_metadata_accounts_v3,
            mpl_token_metadata::types::{Collection, DataV2},
            CreateMasterEditionV3, CreateMetadataAccountsV3, Metadata,
        },
        token::{mint_to, Mint, MintTo, Token, TokenAccount, Transfer},
    },
};

declare_id!("GVJZWCRBZYNMcdbUaeBfgs5pyAJ6yNX97RQTfheaiHAc");

#[program]
pub mod timed_auction {
    use anchor_lang::system_program;
    use anchor_spl::token;

    use super::*;

    pub fn mint_collection(
        ctx: Context<CreateCollection>,
        collection_name: String,
        collection_symbol: String,
        collection_uri: String,
    ) -> Result<()> {
        msg!("Creating collection");

        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.collection_mint_account.to_account_info(),
                    to: ctx
                        .accounts
                        .collection_associated_token_account
                        .to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            1,
        )?;

        msg!("Creating metadata account");

        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.collection_metadata_account.to_account_info(),
                    mint: ctx.accounts.collection_mint_account.to_account_info(),
                    mint_authority: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.payer.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            DataV2 {
                name: collection_name,
                symbol: collection_symbol,
                uri: collection_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false,
            true,
            None,
        )?;

        msg!("Creating master edition account");

        create_master_edition_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    edition: ctx.accounts.collection_edition_account.to_account_info(),
                    mint: ctx.accounts.collection_mint_account.to_account_info(),
                    update_authority: ctx.accounts.payer.to_account_info(),
                    mint_authority: ctx.accounts.payer.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    metadata: ctx.accounts.collection_metadata_account.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            None,
        )?;

        msg!("Collection has minted successfully ");

        Ok(())
    }

    pub fn mint_nft(
        ctx: Context<CreateToken>,
        nft_name: String,
        nft_symbol: String,
        nft_uri: String,
        collection_address: Pubkey,
    ) -> Result<()> {
        msg!("Minting Tokens");

        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint_account.to_account_info(),
                    to: ctx.accounts.associated_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            1,
        )?;

        msg!("Creating metadata account");

        let collection = Collection {
            verified: false,
            key: collection_address,
        };

        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    mint: ctx.accounts.mint_account.to_account_info(),
                    mint_authority: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.payer.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            DataV2 {
                name: nft_name,
                symbol: nft_symbol,
                uri: nft_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: Some(collection),
                uses: None,
            },
            false,
            true,
            None,
        )?;

        msg!("Creating master edition account");

        create_master_edition_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    edition: ctx.accounts.edition_account.to_account_info(),
                    mint: ctx.accounts.mint_account.to_account_info(),
                    update_authority: ctx.accounts.payer.to_account_info(),
                    mint_authority: ctx.accounts.payer.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            None,
        )?;

        msg!("NFT minted successfully");

        Ok(())
    }

    pub fn start_auction(
        ctx: Context<StartAuction>,
        start_time: i64,
        end_time: i64,
        starting_price: u64,
    ) -> Result<()> {
        msg!("Strating the auction");

        let pda_account = &mut ctx.accounts.pda_account;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.seller_token_account.to_account_info(),
                    to: ctx.accounts.pda_token_account.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            1,
        )?;

        pda_account.mint = ctx.accounts.mint.key();
        pda_account.seller = ctx.accounts.seller.key();
        pda_account.highest_bid = starting_price;
        pda_account.start_time = start_time;
        pda_account.end_time = end_time;

        msg!("Auction has started");

        Ok(())
    }

    pub fn place_bid(ctx: Context<PlaceBid>, bid_amount: u64) -> Result<()> {
        msg!("Bidding started");

        let auction = &mut ctx.accounts.pda_account;
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            current_time >= auction.start_time,
            AuctionError::AuctionNotStarted
        );
        require!(current_time < auction.end_time, AuctionError::AuctionEnded);
        require!(bid_amount > auction.highest_bid, AuctionError::BidTooLow);

        auction.highest_bid = bid_amount;
        auction.highest_bidder = ctx.accounts.bidder.key();

        msg!("Bid has been placed");

        Ok(())
    }

    pub fn end_auction(ctx: Context<EndAuction>) -> Result<()> {
        msg!("Ending the auction");

        let auction = &mut ctx.accounts.pda_account;
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            current_time > auction.end_time,
            AuctionError::AuctionNotEnded
        );

        let bump = &[ctx.bumps.pda_signer];
        let binding = ctx.accounts.mint.key();
        let signer_seeds = &[&[b"sale", binding.as_ref(), bump][..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pda_token_account.to_account_info(),
                    to: ctx.accounts.highest_bidder_token_account.to_account_info(),
                    authority: ctx.accounts.pda_signer.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.highest_bidder.to_account_info(),
                    to: ctx.accounts.seller.to_account_info(),
                },
            ),
            auction.highest_bid,
        )?;

        Ok(())
    }

    pub fn cancel_auction(ctx: Context<CancelAuction>) -> Result<()> {
        msg!("Canceling the auction");

        let auction = &mut ctx.accounts.pda_account;
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            current_time >= auction.start_time,
            AuctionError::AuctionNotStarted
        );
        require!(current_time < auction.end_time, AuctionError::AuctionEnded);
        require!(
            auction.highest_bidder == Pubkey::default(),
            AuctionError::BidsPlaced
        );

        let bump = &[ctx.bumps.pda_signer];
        let binding = ctx.accounts.mint.key();
        let signer_seeds = &[&[b"sale", binding.as_ref(), bump][..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pda_token_account.to_account_info(),
                    to: ctx.accounts.seller_token_account.to_account_info(),
                    authority: ctx.accounts.pda_signer.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        auction.close(ctx.accounts.seller.to_account_info())?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub edition_account: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer.key(),
        mint::freeze_authority = payer.key(),
    )]
    pub mint_account: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_account,
        associated_token::authority = payer,
    )]
    pub associated_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_metadata_account: UncheckedAccount<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint_account.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_edition_account: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer.key(),
        mint::freeze_authority = payer.key(),
    )]
    pub collection_mint_account: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = collection_mint_account,
        associated_token::authority = payer,
    )]
    pub collection_associated_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct Auction {
    pub mint: Pubkey,
    pub seller: Pubkey,
    pub highest_bidder: Pubkey,
    pub highest_bid: u64,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Accounts)]
pub struct StartAuction<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = seller,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 32 + 32,
        seeds = [b"sale", mint.key().as_ref()],
        bump,
    )]
    pub pda_account: Account<'info, Auction>,

    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = mint,
        associated_token::authority = pda_signer,
    )]
    pub pda_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: Validate address by deriving pda
    #[account(
        seeds = [b"sale", mint.key().as_ref()],
        bump,
    )]
    pub pda_signer: AccountInfo<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,

    #[account(mut)]
    pub pda_account: Account<'info, Auction>,
}

#[derive(Accounts)]
pub struct EndAuction<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(mut)]
    pub highest_bidder: Signer<'info>,

    #[account(mut)]
    pub pda_account: Account<'info, Auction>,

    #[account(mut)]
    pub pda_token_account: Account<'info, TokenAccount>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"sale", mint.key().as_ref()],
        bump,
    )]
    pub pda_signer: AccountInfo<'info>,

    #[account(mut)]
    pub highest_bidder_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CancelAuction<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(mut)]
    pub pda_account: Account<'info, Auction>,

    #[account(mut)]
    pub pda_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"sale", mint.key().as_ref()],
        bump,
    )]
    pub pda_signer: AccountInfo<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[error_code]
pub enum AuctionError {
    #[msg("Auction has not started yet.")]
    AuctionNotStarted,
    #[msg("Auction has already ended.")]
    AuctionEnded,
    #[msg("Bid amount is too low.")]
    BidTooLow,
    #[msg("Auction has not ended yet.")]
    AuctionNotEnded,
    #[msg("Cannot cancel auction because bids have been placed.")]
    BidsPlaced,
}
