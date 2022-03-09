//! Init or close an [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct FinalizeBuffer<'info> {
    #[account(mut)]
    pub buffer: Account<'info, InstructionBuffer>,
    pub smart_wallet: Account<'info, SmartWallet>,
    pub transaction: Box<Account<'info, Transaction>>,
    pub owner: Signer<'info>,
}

/// Emitted when a [InstructionBuffer] is initialized.
#[event]
pub struct FinalizeBufferEvent {
    /// The buffer.
    pub buffer: Pubkey,
    /// The time [InstructionBuffer] is finalized.
    #[index]
    pub time: i64,
}

pub fn handler(ctx: Context<FinalizeBuffer>) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.finalized_at = Clock::get()?.unix_timestamp;

    emit!(FinalizeBufferEvent {
        buffer: buffer.key(),
        time: buffer.finalized_at,
    });

    Ok(())
}

impl<'info> Validate<'info> for FinalizeBuffer<'info> {
    fn validate(&self) -> Result<()> {
        invariant!(self.buffer.finalized_at == 0);

        invariant!(
            self.smart_wallet.owner_set_seqno == self.transaction.owner_set_seqno,
            OwnerSetChanged
        );

        // ensure that the owner is a signer
        // this prevents common frontrunning/flash loan attacks
        self.smart_wallet.owner_index(self.owner.key())?;

        Ok(())
    }
}
