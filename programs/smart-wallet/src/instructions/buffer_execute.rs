//! Executes an instruction off of the [InstructionBuffer].

use anchor_lang::solana_program::program::invoke_signed;

use crate::*;

#[derive(Accounts)]
pub struct ExecuteIx<'info> {
    #[account(mut)]
    pub buffer: Account<'info, InstructionBuffer>,
    #[account(mut)]
    pub transaction: Box<Account<'info, Transaction>>,
    /// The [SmartWallet].
    pub smart_wallet: Account<'info, SmartWallet>,
}

pub fn handle_with_invoker<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteIx<'info>>) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    let tx = &ctx.accounts.transaction;
    let ix = &tx.instructions[buffer.exec_count as usize];
    // TODO(michael): invoke_signed(&ix.into(), ctx.remaining_accounts, &[])?;

    buffer.exec_count += 1;

    Ok(())
}

impl<'info> Validate<'info> for ExecuteIx<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.buffer.transaction, self.transaction);
        invariant!(self.buffer.ready);
        invariant!(self.buffer.exec_count < self.transaction.instructions.len() as u8);

        Ok(())
    }
}
