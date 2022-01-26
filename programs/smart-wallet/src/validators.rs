//! Account validators.

use crate::*;
use vipers::{assert_keys_eq, invariant, unwrap_int, validate::Validate};

impl<'info> Validate<'info> for CreateSmartWallet<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

impl<'info> Validate<'info> for Auth<'info> {
    fn validate(&self) -> ProgramResult {
        invariant!(
            self.smart_wallet.to_account_info().is_signer,
            "smart_wallet.is_signer"
        );
        Ok(())
    }
}

impl<'info> Validate<'info> for CreateTransaction<'info> {
    fn validate(&self) -> ProgramResult {
        // owner_index check happens later
        Ok(())
    }
}

impl<'info> Validate<'info> for Approve<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.smart_wallet,
            self.transaction.smart_wallet,
            "smart_wallet"
        );
        require!(
            self.smart_wallet.owner_set_seqno == self.transaction.owner_set_seqno,
            OwnerSetChanged
        );
        Ok(())
    }
}

impl<'info> Validate<'info> for ExecuteTransaction<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.smart_wallet,
            self.transaction.smart_wallet,
            "smart_wallet"
        );
        require!(
            self.smart_wallet.owner_set_seqno == self.transaction.owner_set_seqno,
            OwnerSetChanged
        );

        // Has this been executed already?
        require!(self.transaction.executed_at == -1, AlreadyExecuted);

        let eta = self.transaction.eta;
        let clock = Clock::get()?;
        let current_ts = clock.unix_timestamp;
        msg!("current_ts: {}; eta: {}", current_ts, eta);
        // Has transaction surpassed timelock?
        require!(current_ts >= eta, TransactionNotReady);
        if eta != NO_ETA {
            // Has grace period passed?
            require!(
                current_ts <= unwrap_int!(eta.checked_add(self.smart_wallet.grace_period)),
                TransactionIsStale
            );
        }

        // Do we have enough signers to execute the TX?
        let sig_count = self.transaction.num_signers();
        require!(
            (sig_count as u64) >= self.smart_wallet.threshold,
            NotEnoughSigners
        );

        // ensure that the owner is a signer
        // this prevents common frontrunning/flash loan attacks
        self.smart_wallet.owner_index(self.owner.key())?;

        Ok(())
    }
}

impl<'info> Validate<'info> for OwnerInvokeInstruction<'info> {
    fn validate(&self) -> ProgramResult {
        self.smart_wallet.owner_index(self.owner.key())?;
        Ok(())
    }
}

impl<'info> Validate<'info> for CreateSubaccountInfo<'info> {
    fn validate(&self) -> ProgramResult {
        // no validation necessary
        Ok(())
    }
}

impl<'info> Validate<'info> for CreateStagedTXInstruction<'info> {
    fn validate(&self) -> ProgramResult {
        // validate the owner
        self.smart_wallet.owner_index(self.owner.key())?;
        Ok(())
    }
}

impl<'info> Validate<'info> for OwnerInvokeStagedInstruction<'info> {
    fn validate(&self) -> ProgramResult {
        // ensure that the owner set is still fresh.
        invariant!(
            self.smart_wallet.owner_set_seqno == self.staged_tx_instruction.owner_set_seqno,
            OwnerSetChanged
        );

        // check to see that this account is authorized to execute
        // this transaction.
        assert_keys_eq!(self.smart_wallet, self.staged_tx_instruction.smart_wallet);
        assert_keys_eq!(self.owner, self.staged_tx_instruction.owner);
        Ok(())
    }
}
