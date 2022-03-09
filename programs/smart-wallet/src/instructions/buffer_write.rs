//! Writes an instruction to the [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct WriteBuffer<'info> {
    pub buffer: Box<Account<'info, InstructionBuffer>>,
    pub writer: Signer<'info>,
}

/// Emitted when an instruction is written to the [InstructionBuffer].
#[event]
pub struct WriteBufferEvent {
    /// The [InstructionBuffer].
    pub buffer: Pubkey,
    /// The [InstructionBuffer::writer].
    pub writer: Pubkey,
}

pub fn handler(ctx: Context<WriteBuffer>, ix: TXInstruction) -> Result<()> {
    ctx.accounts.buffer.instructions.push(ix);

    let buffer = &ctx.accounts.buffer;
    emit!(WriteBufferEvent {
        buffer: buffer.key(),
        writer: buffer.writer,
    });

    Ok(())
}

impl<'info> Validate<'info> for WriteBuffer<'info> {
    fn validate(&self) -> Result<()> {
        invariant!(self.buffer.finalized_at == 0);
        assert_keys_eq!(self.writer.key(), self.buffer.writer);

        Ok(())
    }
}
