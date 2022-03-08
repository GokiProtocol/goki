use crate::*;

/// Accounts for [smart_wallet::create_transaction].
#[derive(Accounts)]
#[instruction(bump: u8, instructions: Vec<TXInstruction>)]
pub struct CreateTransaction<'info> {
    /// The [SmartWallet].
    #[account(mut)]
    pub smart_wallet: Account<'info, SmartWallet>,
    /// The [Transaction].
    #[account(
        init,
        seeds = [
            b"GokiTransaction".as_ref(),
            smart_wallet.key().to_bytes().as_ref(),
            smart_wallet.num_transactions.to_le_bytes().as_ref()
        ],
        bump,
        payer = payer,
        space = Transaction::space(instructions),
    )]
    pub transaction: Account<'info, Transaction>,
    /// One of the owners. Checked in the handler via [SmartWallet::owner_index].
    pub proposer: Signer<'info>,
    /// Payer to create the [Transaction].
    #[account(mut)]
    pub payer: Signer<'info>,
    /// The [System] program.
    pub system_program: Program<'info, System>,
}

/// Emitted when a [Transaction] is proposed.
#[event]
pub struct TransactionCreateEvent {
    /// The [SmartWallet].
    #[index]
    pub smart_wallet: Pubkey,
    /// The [Transaction].
    #[index]
    pub transaction: Pubkey,
    /// The owner which proposed the transaction.
    pub proposer: Pubkey,
    /// Instructions associated with the [Transaction].
    pub instructions: Vec<TXInstruction>,
    /// The [Transaction::eta].
    pub eta: i64,
    /// The Unix timestamp when the event was emitted.
    pub timestamp: i64,
}

pub fn handler(
    bump: u8,
    eta: i64,
    proposer: Pubkey,
    instructions: &[TXInstruction],
    smart_wallet: &mut Account<SmartWallet>,
    tx: &mut Account<Transaction>,
) -> Result<()> {
    let owner_index = smart_wallet.owner_index(proposer)?;

    let clock = Clock::get()?;
    let current_ts = clock.unix_timestamp;
    if smart_wallet.minimum_delay != 0 {
        invariant!(
            eta >= unwrap_int!(current_ts.checked_add(smart_wallet.minimum_delay as i64)),
            InvalidETA
        );
    }
    if eta != NO_ETA {
        invariant!(eta >= 0, "ETA must be positive");
        let delay = unwrap_int!(eta.checked_sub(current_ts));
        invariant!(delay >= 0, "ETA must be in the future");
        invariant!(delay <= MAX_DELAY_SECONDS, DelayTooHigh);
    }

    // generate the signers boolean list
    let owners = &smart_wallet.owners;
    let mut signers = Vec::new();
    signers.resize(owners.len(), false);
    signers[owner_index] = true;

    let index = smart_wallet.num_transactions;
    smart_wallet.num_transactions = unwrap_int!(smart_wallet.num_transactions.checked_add(1));

    // init the TX
    tx.smart_wallet = smart_wallet.key();
    tx.index = index;
    tx.bump = bump;

    tx.proposer = proposer;
    tx.instructions = instructions.to_vec();
    tx.signers = signers;
    tx.owner_set_seqno = smart_wallet.owner_set_seqno;
    tx.eta = eta;

    tx.executor = Pubkey::default();
    tx.executed_at = -1;

    emit!(TransactionCreateEvent {
        smart_wallet: smart_wallet.key(),
        transaction: tx.key(),
        proposer,
        instructions: instructions.to_vec(),
        eta,
        timestamp: Clock::get()?.unix_timestamp
    });

    Ok(())
}

impl<'info> Validate<'info> for CreateTransaction<'info> {
    fn validate(&self) -> Result<()> {
        // owner_index check happens later
        Ok(())
    }
}
