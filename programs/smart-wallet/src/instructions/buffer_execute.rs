//! Executes an instruction off of the [InstructionBuffer].

use anchor_lang::solana_program::program::invoke_signed;
use num_traits::ToPrimitive;

use crate::*;

#[derive(Accounts)]
pub struct ExecuteIx<'info> {
    #[account(mut)]
    pub buffer: Account<'info, InstructionBuffer>,
    /// The buffer's [SmartWallet]
    pub smart_wallet: Account<'info, SmartWallet>,
    /// The [InstructionBuffer::executer].
    pub executor: Signer<'info>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteIx<'info>>) -> Result<()> {
    let smart_wallet = &ctx.accounts.smart_wallet;
    let wallet_seeds: &[&[&[u8]]] = &[&[
        b"GokiSmartWallet" as &[u8],
        &smart_wallet.base.to_bytes(),
        &[smart_wallet.bump],
    ]];

    let buffer = &mut ctx.accounts.buffer;
    let ix = &buffer.instructions[buffer.exec_count as usize];
    invoke_signed(&ix.into(), ctx.remaining_accounts, wallet_seeds)?;

    buffer.exec_count += 1;

    Ok(())
}

impl<'info> Validate<'info> for ExecuteIx<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.buffer.smart_wallet, self.smart_wallet.key());

        invariant!(self.buffer.finalized_at > 0, BufferNotFinalized);
        invariant!(self.buffer.exec_count < unwrap_opt!(self.buffer.instructions.len().to_u8()));

        let current_ts = Clock::get()?.unix_timestamp;
        // Has buffer surpassed timelock?
        invariant!(current_ts >= self.buffer.eta, TransactionNotReady);

        Ok(())
    }
}
