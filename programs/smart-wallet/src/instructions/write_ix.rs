//! Writes an instruction to the [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct WriteIx<'info> {
    #[account(mut)]
    pub buffer: Box<Account<'info, InstructionBuffer>>,
    pub writer: Signer<'info>,
}

/// Emitted when an instruction is written to the [InstructionBuffer].
#[event]
pub struct WriteIxEvent {
    /// The [InstructionBuffer].
    pub buffer: Pubkey,
    /// The [InstructionBuffer::writer].
    pub writer: Pubkey,
    /// The program id of the instruction written.
    pub program_id: Pubkey,
}

pub fn handler(ctx: Context<WriteIx>, ix: TXInstruction) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;

    emit!(WriteIxEvent {
        buffer: buffer.key(),
        writer: buffer.writer,
        program_id: ix.program_id
    });

    buffer.staged_tx_instructions.push(ix);

    Ok(())
}

impl<'info> Validate<'info> for WriteIx<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.writer.key(), self.buffer.writer);

        Ok(())
    }
}
