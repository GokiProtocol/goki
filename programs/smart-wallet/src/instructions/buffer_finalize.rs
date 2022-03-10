//! Finalize an [InstructionBuffer] for execution.

use crate::*;

#[derive(Accounts)]
pub struct FinalizeBuffer<'info> {
    #[account(mut)]
    pub buffer: Account<'info, InstructionBuffer>,
    pub authority: Signer<'info>,
}

/// Emitted when a [InstructionBuffer] is finalized.
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
        assert_keys_eq!(self.buffer.authority, self.authority);
        invariant!(self.buffer.finalized_at == 0);

        Ok(())
    }
}
