use crate::InvokeSignedInstruction;
use anchor_lang::prelude::*;
use anchor_spl::token;
use vipers::{assert_keys_eq, assert_owner, validate::Validate};

impl<'info> Validate<'info> for InvokeSignedInstruction<'info> {
    fn validate(&self) -> ProgramResult {
        // NFT account must be owned by the `owner_authority`.
        assert_keys_eq!(
            self.owner_authority,
            self.nft_account.owner,
            "owner_authority"
        );

        // NFT account must be associated with the token program.
        assert_owner!(self.nft_account, token::ID);

        // Check NFT ownership.
        require!(self.nft_account.amount == 1, Unauthorized);

        Ok(())
    }
}
