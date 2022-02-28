//! Creates a [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct InitIxBuffer<'info> {
    #[account(zero)]
    pub buffer: Account<'info, InstructionBuffer>,
    /// CHECK: Writer account that can write to the buffer.
    pub writer: UncheckedAccount<'info>,
}

pub fn handler<'info>(ctx: Context<InitIxBuffer>) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.writer = ctx.accounts.writer.key();

    Ok(())
}

impl<'info> Validate<'info> for InitIxBuffer<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}
