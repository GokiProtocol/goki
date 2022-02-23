use crate::InvokeSignedInstruction;
use anchor_lang::prelude::*;
use vipers::{assert_keys_eq, invariant, validate::Validate};

impl<'info> Validate<'info> for InvokeSignedInstruction<'info> {
    fn validate(&self) -> Result<()> {
        // NFT account must be owned by the `owner_authority`.
        assert_keys_eq!(
            self.owner_authority,
            self.nft_account.owner,
            "owner_authority"
        );

        // Check NFT ownership.
        invariant!(self.nft_account.amount == 1, Unauthorized);

        Ok(())
    }
}
