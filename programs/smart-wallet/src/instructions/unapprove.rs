//! Instruction handler for [smart_wallet::unapprove].

use crate::*;

/// Instruction handler for [smart_wallet::unapprove].
pub fn handler(ctx: Context<Approve>) -> Result<()> {
    let owner_index = ctx
        .accounts
        .smart_wallet
        .try_owner_index(ctx.accounts.owner.key())?;
    ctx.accounts.transaction.signers[owner_index] = false;

    emit!(TransactionUnapproveEvent {
        smart_wallet: ctx.accounts.smart_wallet.key(),
        transaction: ctx.accounts.transaction.key(),
        owner: ctx.accounts.owner.key(),
        timestamp: Clock::get()?.unix_timestamp
    });
    Ok(())
}