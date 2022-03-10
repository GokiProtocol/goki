//! Executes an instruction off of the [InstructionBuffer].

use crate::*;

use anchor_lang::solana_program::program::invoke_signed;

#[derive(Accounts)]
pub struct ExecuteBufferBundle<'info> {
    #[account(mut)]
    /// The [InstructionBuffer].
    pub buffer: Account<'info, InstructionBuffer>,
    /// The buffer's [SmartWallet]
    pub smart_wallet: Account<'info, SmartWallet>,
    /// The [InstructionBuffer::executor].
    pub executor: Signer<'info>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteBufferBundle<'info>>,
    bundle_index: u8,
) -> Result<()> {
    let smart_wallet = &ctx.accounts.smart_wallet;
    let wallet_seeds: &[&[&[u8]]] = &[&[
        b"GokiSmartWallet" as &[u8],
        &smart_wallet.base.to_bytes(),
        &[smart_wallet.bump],
    ]];

    let buffer = &mut ctx.accounts.buffer;
    let mut bundle = unwrap_opt!(buffer.get_bundle(bundle_index), BufferBundleNotFound);
    invariant!(!bundle.is_executed(), BufferBundleExecuted);

    let ix = &bundle.instructions[usize::from(bundle.exec_count)];
    invoke_signed(&ix.into(), ctx.remaining_accounts, wallet_seeds)?;

    bundle.exec_count = unwrap_int!(bundle.exec_count.checked_add(1));
    buffer.set_bundle(bundle_index, &bundle)?;

    Ok(())
}

impl<'info> Validate<'info> for ExecuteBufferBundle<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.buffer.executor, self.executor);
        assert_keys_eq!(self.buffer.smart_wallet, self.smart_wallet);

        invariant!(
            self.smart_wallet.owner_set_seqno == self.buffer.owner_set_seqno,
            OwnerSetChanged
        );
        invariant!(self.buffer.is_finalized(), BufferBundleNotFinalized);

        let current_ts = Clock::get()?.unix_timestamp;
        // Has buffer surpassed timelock?
        invariant!(current_ts >= self.buffer.eta, TransactionNotReady);

        Ok(())
    }
}
