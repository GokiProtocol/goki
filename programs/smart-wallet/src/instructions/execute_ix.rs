//! Executes an instruction off of the [TXInstructionBuffer].

use anchor_lang::solana_program::program::invoke;

use crate::*;

#[derive(Accounts)]
pub struct ExecuteIx<'info> {
    #[account(mut)]
    pub buffer: Account<'info, TXInstructionBuffer>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteIx<'info>>) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    let ix = &buffer.staged_tx_instructions[buffer.exec_count as usize];
    invoke(&ix.into(), ctx.remaining_accounts)?;
    buffer.exec_count += 1;

    Ok(())
}

impl<'info> Validate<'info> for ExecuteIx<'info> {
    fn validate(&self) -> Result<()> {
        invariant!(self.buffer.exec_count < self.buffer.staged_tx_instructions.len() as u8);

        Ok(())
    }
}
