//! Init or close an [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct InitIxBuffer<'info> {
    #[account(zero)]
    pub buffer: Account<'info, InstructionBuffer>,
    /// CHECK: This can be an arbitrary account. Writer account that can write to the buffer.
    pub writer: UncheckedAccount<'info>,
}

/// Emitted when a [InstructionBuffer] is initialized.
#[event]
pub struct InitBufferEvent {
    /// The [InstructionBuffer::writer].
    #[index]
    pub writer: Pubkey,
    /// The buffer.
    pub buffer: Pubkey,
}

pub fn handle_init(ctx: Context<InitIxBuffer>) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.writer = ctx.accounts.writer.key();

    emit!(InitBufferEvent {
        writer: buffer.writer,
        buffer: buffer.key()
    });

    Ok(())
}

impl<'info> Validate<'info> for InitIxBuffer<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CloseIxBuffer<'info> {
    #[account(mut, close = writer)]
    pub buffer: Account<'info, InstructionBuffer>,
    pub writer: Signer<'info>,
}

/// Emitted when an [InstructionBuffer] is closed.
#[event]
pub struct CloseBufferEvent {
    /// The [InstructionBuffer::writer].
    #[index]
    pub writer: Pubkey,
    /// The buffer.
    pub buffer: Pubkey,
}

pub fn handle_close(ctx: Context<CloseIxBuffer>) -> Result<()> {
    emit!(CloseBufferEvent {
        writer: ctx.accounts.writer.key(),
        buffer: ctx.accounts.buffer.key(),
    });
    Ok(())
}

impl<'info> Validate<'info> for CloseIxBuffer<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.writer.key(), self.buffer.writer);

        Ok(())
    }
}
