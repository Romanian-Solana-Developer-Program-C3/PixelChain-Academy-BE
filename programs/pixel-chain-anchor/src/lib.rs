use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo};
use anchor_spl::metadata::Metadata;
use anchor_spl::metadata::{
    create_metadata_accounts_v3,
    mpl_token_metadata::types::DataV2,
};

declare_id!("HZMxjQynSqiu4uAydbNwXy8uMkYtMa2FV1i6UpvDuDY8");

const CHALLENGE_PREFIX: &[u8] = b"challenge";

#[program]
pub mod pixel_chain_anchor {
    use super::*;

    pub fn init_player(ctx: Context<InitPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.authority = ctx.accounts.authority.key();
        player.xp = 0;
        player.completed_bitmap = vec![0u8; 32]; // 256 provocări
        Ok(())
    }

    pub fn complete_challenge(
        ctx: Context<CompleteChallenge>,
        challenge_id: u8,
    ) -> Result<()> {
        {                                   // ↙︎ bloc nou
            let player = &mut ctx.accounts.player;
    
            // 1) verifică bit
            let byte_i = (challenge_id / 8) as usize;
            let bit_i  = challenge_id % 8;
            require!(
                player.completed_bitmap[byte_i] & (1 << bit_i) == 0,
                ErrorCode::AlreadyCompleted
            );
    
            // 2) marchează + XP
            player.completed_bitmap[byte_i] |= 1 << bit_i;
            player.xp = player.xp.checked_add(10).unwrap();
        }                                   // ↖︎ împrumutul mutabil se termină aici
    
        // 3) acum putem împrumuta `&ctx` (immutabil)
        mint_nft_cpi(&ctx, challenge_id)?;
    
        emit!(ChallengeCompleted {
            player: ctx.accounts.player.key(),
            challenge_id,
            mint: ctx.accounts.reward_mint.key(),
        });
    
        Ok(())
    }    

    pub fn admin_add_challenge(
        ctx: Context<AdminAddChallenge>,
        challenge_id: u8,
        uri: String,
    ) -> Result<()> {
        let ch = &mut ctx.accounts.challenge;
        ch.id  = challenge_id;
        ch.uri = uri;
        Ok(())
    }
}

/* -------- context structs -------- */

#[derive(Accounts)]
pub struct InitPlayer<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Player::INIT_SPACE,
        seeds = [b"player", authority.key().as_ref()],
        bump,
    )]
    pub player: Account<'info, Player>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

//  ---- AdminAddChallenge ----
#[derive(Accounts)]
#[instruction(challenge_id: u8)]
pub struct AdminAddChallenge<'info> {
    #[account(
        init,
        payer  = authority,
        space  = 8 + Challenge::INIT_SPACE,
        seeds  = [CHALLENGE_PREFIX, &[challenge_id]],
        bump,
    )]
    pub challenge: Account<'info, Challenge>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ---- CompleteChallenge ----
#[derive(Accounts)]
#[instruction(challenge_id: u8)]
pub struct CompleteChallenge<'info> {
    #[account(mut, seeds = [b"player", authority.key().as_ref()], bump)]
    pub player: Account<'info, Player>,

    #[account(seeds = [CHALLENGE_PREFIX, &[challenge_id]], bump)]
    pub challenge: Account<'info, Challenge>,

    /// CHECK: PDA mint – validat în cod
    #[account(mut)]
    pub reward_mint: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    // SPL & Metadata
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/* -------- state -------- */

#[account]
pub struct Player {
    pub authority: Pubkey,
    pub xp: u32,
    pub completed_bitmap: Vec<u8>, // dynamic
}
impl Player {
    pub const INIT_SPACE: usize = 32 + 4 + (4 + 32); // authority + xp + vec header + 32 bytes
}

#[account]
pub struct Challenge {
    pub id: u8,
    pub uri: String, // arweave / ipfs
}
impl Challenge {
    pub const INIT_SPACE: usize = 1 + 4 + 200; // id + string header + 200 bytes
}

/* -------- events -------- */

#[event]
pub struct ChallengeCompleted {
    pub player: Pubkey,
    pub challenge_id: u8,
    pub mint: Pubkey,
}

/* -------- helper & errors -------- */

fn mint_nft_cpi(ctx: &Context<CompleteChallenge>, challenge_id: u8) -> Result<()> {
    // (1) Creează metadata; (2) Mint 1 token către ATA jucătorului
    // TODO: implementează logică completă + derive seeds pt reward_mint
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Challenge already completed")]
    AlreadyCompleted,
}
