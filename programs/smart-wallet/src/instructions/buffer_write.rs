//! Writes an instruction to the [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct WriteBuffer<'info> {
    pub buffer: Account<'info, InstructionBuffer>,
    #[account(mut)]
    pub transaction: Box<Account<'info, Transaction>>,
    pub writer: Signer<'info>,
}

/// Emitted when an instruction is written to the [InstructionBuffer].
#[event]
pub struct WriteIxEvent {
    /// The [InstructionBuffer].
    pub buffer: Pubkey,
    /// The [InstructionBuffer::writer].
    pub writer: Pubkey,
    /// Transaction written to.
    pub transaction: Pubkey,
}

pub fn handler(ctx: Context<WriteBuffer>, ix: TXInstruction) -> Result<()> {
    ctx.accounts.transaction.instructions.push(ix);

    let buffer = &ctx.accounts.buffer;
    emit!(WriteIxEvent {
        buffer: buffer.key(),
        writer: buffer.writer,
        transaction: buffer.transaction,
    });

    Ok(())
}

impl<'info> Validate<'info> for WriteBuffer<'info> {
    fn validate(&self) -> Result<()> {
        invariant!(self.buffer.finalized_at == 0);
        assert_keys_eq!(self.writer.key(), self.buffer.writer);
        assert_keys_eq!(self.transaction.key(), self.buffer.transaction);

        Ok(())
    }
}
