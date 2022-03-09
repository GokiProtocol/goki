//! Closes an [InstructionBuffer] for execution.

use crate::*;

#[derive(Accounts)]
pub struct CloseBuffer<'info> {
    #[account(mut, close = executer)]
    pub buffer: Account<'info, InstructionBuffer>,
    pub executer: Signer<'info>,
}

/// Emitted when an [InstructionBuffer] is closed.
#[event]
pub struct CloseBufferEvent {
    /// The [InstructionBuffer].
    pub buffer: Pubkey,
    /// The [InstructionBuffer::writer].
    #[index]
    pub executer: Pubkey,
}

pub fn handler(ctx: Context<CloseBuffer>) -> Result<()> {
    emit!(CloseBufferEvent {
        buffer: ctx.accounts.buffer.key(),
        executer: ctx.accounts.executer.key(),
    });
    Ok(())
}

impl<'info> Validate<'info> for CloseBuffer<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.executer.key(), self.buffer.executer);

        Ok(())
    }
}
