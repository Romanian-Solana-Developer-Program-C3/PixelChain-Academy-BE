use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use anchor_spl::associated_token::Create as CreateAta;
use anchor_spl::associated_token::{self, AssociatedToken};
use anchor_spl::token::{self, InitializeMint, Mint, MintTo, Token, TokenAccount};

declare_id!("FgBCXGMCMYimooD9cdHuasz6kczQP9jEXpfJYnH3GMfN");

const REWARD_MINT_SEED: &[u8] = b"reward_mint";

#[program]
pub mod pixel_chain_anchor {
    use anchor_spl::associated_token;

    use super::*;

    pub fn init_player(ctx: Context<InitPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.authority = ctx.accounts.authority.key();
        player.xp = 0;
        player.completed_bitmap = vec![0u8; 32];
        Ok(())
    }

    pub fn admin_add_challenge(
        ctx: Context<AdminAddChallenge>,
        challenge_id: u8,
        uri: String,
    ) -> Result<()> {
        let ch = &mut ctx.accounts.challenge;
        ch.id = challenge_id;
        ch.uri = uri;
        Ok(())
    }

    pub fn complete_challenge(ctx: Context<CompleteChallenge>, challenge_id: u8) -> Result<()> {
        {
            let player = &mut ctx.accounts.player;
            let byte_i = (challenge_id / 8) as usize;
            let bit_i = challenge_id % 8;
            require!(
                player.completed_bitmap[byte_i] & (1 << bit_i) == 0,
                ErrorCode::AlreadyCompleted
            );
            player.completed_bitmap[byte_i] |= 1 << bit_i;
            player.xp = player.xp.checked_add(10).unwrap();
        }

        let seeds: &[&[u8]] = &[
            REWARD_MINT_SEED,
            ctx.accounts.authority.key.as_ref(),
            &[challenge_id],
        ];
        let (_mint_pda, bump) = Pubkey::find_program_address(seeds, ctx.program_id);
        let signer_seeds: &[&[&[u8]]] = &[&[
            REWARD_MINT_SEED,
            ctx.accounts.authority.key.as_ref(),
            &[challenge_id],
            &[bump],
        ]];

        // init mint
        token::initialize_mint(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.reward_mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                signer_seeds,
            ),
            0,
            &ctx.accounts.authority.key(),
            None,
        )?;

        // create ATA
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            CreateAta {
                payer: ctx.accounts.authority.to_account_info(),
                associated_token: ctx.accounts.reward_ata.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                mint: ctx.accounts.reward_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        ))?;

        // mint_to
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.reward_mint.to_account_info(),
                    to: ctx.accounts.reward_ata.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        Ok(())
    }
}

#[account]
pub struct Challenge {
    pub id: u8,
    pub uri: String,
}

impl Challenge {
    pub const INIT_SPACE: usize = 1 + 4 + 200;
}

#[derive(Accounts)]
#[instruction(challenge_id: u8)]
pub struct AdminAddChallenge<'info> {
    #[account(
        init,
        payer  = authority,
        space  = 8 + Challenge::INIT_SPACE,
        seeds  = [
            &b"challenge"[..],
            &[challenge_id][..],
        ],
        bump
    )]
    pub challenge: Account<'info, Challenge>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitPlayer<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + Player::SIZE,
        seeds = [b"player", authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(challenge_id: u8)]
pub struct CompleteChallenge<'info> {
    #[account(mut)]
    pub player: Account<'info, Player>,

    #[account(seeds = [b"challenge", &[challenge_id]], bump)]
    pub challenge: Account<'info, Challenge>,

    /// CHECK: PDA pe care-l inițializăm manual cu CPI-ul de mint
    #[account(mut)]
    pub reward_mint: AccountInfo<'info>,

    /// CHECK: PDA-ul ATA pe care-l creăm manual
    #[account(mut)]
    pub reward_ata: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct Player {
    pub authority: Pubkey,
    pub xp: u32,
    pub completed_bitmap: Vec<u8>,
}
impl Player {
    pub const SIZE: usize = 32  // authority
        + 4   // xp
        + 4   // vec length
        + 32; // 32 bytes bitmap
}

#[error_code]
pub enum ErrorCode {
    #[msg("Challenge already completed")]
    AlreadyCompleted,
}
