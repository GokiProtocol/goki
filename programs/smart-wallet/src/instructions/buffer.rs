//! Creates a [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct InitIxBuffer<'info> {
    #[account(zero)]
    pub buffer: Account<'info, InstructionBuffer>,
    /// CHECK: Writer account that can write to the buffer.
    pub writer: UncheckedAccount<'info>,
}

pub fn handle_init(ctx: Context<InitIxBuffer>) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.writer = ctx.accounts.writer.key();

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

pub fn handle_close(_ctx: Context<CloseIxBuffer>) -> Result<()> {
    Ok(())
}

impl<'info> Validate<'info> for CloseIxBuffer<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.writer.key(), self.buffer.writer);

        Ok(())
    }
}
