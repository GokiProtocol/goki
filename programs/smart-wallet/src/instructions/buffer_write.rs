//! Writes an instruction to the [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct WriteBuffer<'info> {
    #[account(mut)]
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
    let buffer = &mut ctx.accounts.buffer;
    buffer.instructions.push(ix);

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
