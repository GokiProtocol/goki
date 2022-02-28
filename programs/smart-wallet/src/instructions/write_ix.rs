//! Writes an instruction to the [TXInstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct WriteIx<'info> {
    #[account(mut)]
    pub buffer: Account<'info, TXInstructionBuffer>,
    pub writer: Signer<'info>,
}

pub fn handler(ctx: Context<WriteIx>, ix: TXInstruction) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.staged_tx_instructions.push(ix);

    Ok(())
}

impl<'info> Validate<'info> for WriteIx<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.writer.key(), self.buffer.writer);

        Ok(())
    }
}
