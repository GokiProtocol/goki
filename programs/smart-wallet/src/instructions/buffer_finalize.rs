//! Finalize an [InstructionBuffer] for execution.

use crate::*;

#[derive(Accounts)]
pub struct FinalizeBuffer<'info> {
    #[account(mut)]
    /// The [InstructionBuffer].
    pub buffer: Account<'info, InstructionBuffer>,
    /// The [InstructionBuffer::authority].
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
    buffer.authority = Pubkey::default();

    emit!(FinalizeBufferEvent {
        buffer: buffer.key(),
        time: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

impl<'info> Validate<'info> for FinalizeBuffer<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.buffer.authority, self.authority);

        Ok(())
    }
}
