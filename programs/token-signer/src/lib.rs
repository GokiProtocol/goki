//! Sign transactions by owning a token

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::Key;
use anchor_spl::token::TokenAccount;
use vipers::assert_keys;
use vipers::validate::Validate;

mod account_validators;

declare_id!("NFTUJzSHuUCsMMqMRJpB7PmbsaU7Wm51acdPk2FXMLn");

#[program]
pub mod token_signer {
    use super::*;

    #[access_control(ctx.accounts.validate())]
    pub fn invoke_signed_instruction(
        ctx: Context<InvokeSignedInstruction>,
        bump: u8,
        data: Vec<u8>,
    ) -> ProgramResult {
        let mint = ctx.accounts.nft_account.mint.to_bytes();
        let seeds: &[&[u8]] = &[b"GokiTokenSigner", &mint, &[bump]];
        let nft_addr = Pubkey::create_program_address(seeds, ctx.program_id)?;
        assert_keys!(nft_addr, ctx.accounts.nft_pda, "nft_pda");

        let accounts: Vec<AccountMeta> = ctx
            .remaining_accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: acc.key(),
                is_signer: if acc.key() == ctx.accounts.nft_pda.key() {
                    true
                } else {
                    acc.is_signer
                },
                is_writable: acc.is_writable,
            })
            .collect();

        // invoke the tx, signed by the PDA
        let ix = Instruction {
            program_id: ctx.accounts.delegate_program_id.key(),
            accounts,
            data,
        };
        invoke_signed(&ix, ctx.remaining_accounts, &[seeds])?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InvokeSignedInstruction<'info> {
    /// Authority attempting to sign.
    pub owner_authority: Signer<'info>,

    /// Account containing at least one token.
    /// This must belong to `owner_authority`.
    pub nft_account: Account<'info, TokenAccount>,

    pub delegate_program_id: UncheckedAccount<'info>,

    /// PDA associated with the NFT.
    #[account(
        seeds = [
            b"GokiTokenSigner",
            nft_account.mint.as_ref()
        ],
        bump = bump,
    )]
    pub nft_pda: UncheckedAccount<'info>,
}

#[error]
pub enum ErrorCode {
    #[msg("Unauthorized.")]
    Unauthorized,
}
