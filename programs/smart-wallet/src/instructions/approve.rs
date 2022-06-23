//! Instruction handler for [smart_wallet::approve].

use crate::*;

/// Instruction handler for [smart_wallet::approve].
pub fn handler(ctx: Context<Approve>) -> Result<()> {
    let owner_index = ctx
        .accounts
        .smart_wallet
        .try_owner_index(ctx.accounts.owner.key())?;
    ctx.accounts.transaction.signers[owner_index] = true;

    emit!(TransactionApproveEvent {
        smart_wallet: ctx.accounts.smart_wallet.key(),
        transaction: ctx.accounts.transaction.key(),
        owner: ctx.accounts.owner.key(),
        timestamp: Clock::get()?.unix_timestamp
    });
    Ok(())
}

/// This validator is used for both approve and unapprove.
impl<'info> Validate<'info> for Approve<'info> {
    fn validate(&self) -> Result<()> {
        // The TX in question should belong to the smart wallet.
        assert_keys_eq!(self.smart_wallet, self.transaction.smart_wallet);

        // If the owner set has changed, we should not allow approvals/unapprovals to change.
        // This can cause someone to be able to approve/unapprove someone else's TXs.
        invariant!(
            self.smart_wallet.owner_set_seqno == self.transaction.owner_set_seqno,
            OwnerSetChanged
        );

        // no point in approving/unapproving if the TX is already executed.
        invariant!(self.transaction.executed_at == -1, AlreadyExecuted);

        Ok(())
    }
}

/// Accounts for [smart_wallet::approve].
#[derive(Accounts)]
pub struct Approve<'info> {
    /// The [SmartWallet].
    pub smart_wallet: Account<'info, SmartWallet>,
    /// The [Transaction].
    #[account(mut, has_one = smart_wallet)]
    pub transaction: Account<'info, Transaction>,
    /// One of the smart_wallet owners. Checked in the handler.
    pub owner: Signer<'info>,
}
