//! Executes an instruction off of the [InstructionBuffer].

use anchor_lang::solana_program::program::invoke_signed;

use crate::*;

#[derive(Accounts)]
pub struct ExecuteIx<'info> {
    #[account(mut)]
    pub buffer: Box<Account<'info, InstructionBuffer>>,
    pub executor: Signer<'info>,
}

pub fn handle_with_invoker<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteIx<'info>>,
    index: u64,
    bump: u8,
    smart_wallet: Pubkey,
) -> Result<()> {
    // Execute the transaction signed by the smart_wallet.
    let invoker_seeds: &[&[&[u8]]] = &[&[
        b"GokiSmartWalletOwnerInvoker" as &[u8],
        &smart_wallet.to_bytes(),
        &index.to_le_bytes(),
        &[bump],
    ]];

    let buffer = &mut ctx.accounts.buffer;
    let ix = &buffer.staged_tx_instructions[buffer.exec_count as usize];
    invoke_signed(&ix.into(), ctx.remaining_accounts, invoker_seeds)?;

    buffer.exec_count += 1;

    Ok(())
}

impl<'info> Validate<'info> for ExecuteIx<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.buffer.executor, self.executor);
        invariant!(self.buffer.exec_count < self.buffer.staged_tx_instructions.len() as u8);

        Ok(())
    }
}
