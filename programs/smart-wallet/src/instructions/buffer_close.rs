//! Closes an [InstructionBuffer] for execution.

use crate::*;

#[derive(Accounts)]
pub struct CloseBuffer<'info> {
    #[account(mut, close = executor)]
    pub buffer: Account<'info, InstructionBuffer>,
    pub executor: Signer<'info>,
}

/// Emitted when an [InstructionBuffer] is closed.
#[event]
pub struct CloseBufferEvent {
    /// The [InstructionBuffer].
    pub buffer: Pubkey,
    /// The [InstructionBuffer::authority].
    #[index]
    pub executor: Pubkey,
}

pub fn handler(ctx: Context<CloseBuffer>) -> Result<()> {
    emit!(CloseBufferEvent {
        buffer: ctx.accounts.buffer.key(),
        executor: ctx.accounts.executor.key(),
    });
    Ok(())
}

impl<'info> Validate<'info> for CloseBuffer<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.executor.key(), self.buffer.executor);

        Ok(())
    }
}
