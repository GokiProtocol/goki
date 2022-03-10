//! Closes an [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct CloseBuffer<'info> {
    /// The [InstructionBuffer].
    #[account(mut, close = authority_or_executor)]
    pub buffer: Account<'info, InstructionBuffer>,
    /// The [InstructionBuffer::authority] or [InstructionBuffer::executor].
    pub authority_or_executor: Signer<'info>,
}

/// Emitted when an [InstructionBuffer] is closed.
#[event]
pub struct CloseBufferEvent {
    /// The [InstructionBuffer].
    pub buffer: Pubkey,
    /// The [InstructionBuffer::authority].
    #[index]
    pub authority_or_executor: Pubkey,
    /// Time when the buffer was closed.
    pub time: i64,
}

pub fn handler(ctx: Context<CloseBuffer>) -> Result<()> {
    emit!(CloseBufferEvent {
        buffer: ctx.accounts.buffer.key(),
        authority_or_executor: ctx.accounts.authority_or_executor.key(),
        time: Clock::get()?.unix_timestamp
    });
    Ok(())
}

impl<'info> Validate<'info> for CloseBuffer<'info> {
    fn validate(&self) -> Result<()> {
        if self.buffer.is_finalized() {
            assert_keys_eq!(self.authority_or_executor, self.buffer.executor);
        } else {
            assert_keys_eq!(self.authority_or_executor, self.buffer.authority);
        }

        Ok(())
    }
}
