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
    /// An owner on the smart wallet.
    pub owner: Signer<'info>,
}

pub fn handle<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteIx<'info>>) -> Result<()> {
    let smart_wallet = &ctx.accounts.smart_wallet;
    let wallet_seeds: &[&[&[u8]]] = &[&[
        b"GokiSmartWallet" as &[u8],
        &smart_wallet.base.to_bytes(),
        &[smart_wallet.bump],
    ]];

    let buffer = &mut ctx.accounts.buffer;
    let tx = &mut ctx.accounts.transaction;
    let ix = &tx.instructions[buffer.exec_count as usize];
    invoke_signed(&ix.into(), ctx.remaining_accounts, wallet_seeds)?;

    buffer.exec_count += 1;
    if buffer.exec_count == tx.instructions.len() as u8 {
        tx.executor = ctx.accounts.owner.key();
        tx.executed_at = Clock::get()?.unix_timestamp;

        emit!(TransactionExecuteEvent {
            smart_wallet: ctx.accounts.smart_wallet.key(),
            transaction: ctx.accounts.transaction.key(),
            executor: ctx.accounts.owner.key(),
            timestamp: Clock::get()?.unix_timestamp
        });
    }

    Ok(())
}

impl<'info> Validate<'info> for ExecuteIx<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.buffer.transaction, self.transaction);
        invariant!(self.buffer.finalized_at > 0);
        invariant!(self.buffer.exec_count < self.transaction.instructions.len() as u8);

        assert_keys_eq!(
            self.smart_wallet,
            self.transaction.smart_wallet,
            "smart_wallet"
        );

        // Has this been executed already?
        invariant!(self.transaction.executed_at == -1, AlreadyExecuted);

        let eta = self.transaction.eta;
        let clock = Clock::get()?;
        let current_ts = clock.unix_timestamp;
        msg!("current_ts: {}; eta: {}", current_ts, eta);
        // Has transaction surpassed timelock?
        invariant!(current_ts >= eta, TransactionNotReady);
        if eta != NO_ETA {
            // Has grace period passed?
            invariant!(
                current_ts <= unwrap_int!(eta.checked_add(self.smart_wallet.grace_period)),
                TransactionIsStale
            );
        }

        // Do we have enough signers to execute the TX?
        let sig_count = self.transaction.num_signers();
        invariant!(
            (sig_count as u64) >= self.smart_wallet.threshold,
            NotEnoughSigners
        );

        // ensure that the owner is a signer
        // this prevents common frontrunning/flash loan attacks
        self.smart_wallet.owner_index(self.owner.key())?;

        Ok(())
    }
}
