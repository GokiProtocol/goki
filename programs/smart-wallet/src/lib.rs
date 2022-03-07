//! Multisig Solana wallet with Timelock capabilities.
//!
//! This program can be used to allow a smart wallet to govern anything a regular
//! [Pubkey] can govern. One can use the smart wallet as a BPF program upgrade
//! authority, a mint authority, etc.
//!
//! To use, one must first create a [SmartWallet] account, specifying two important
//! parameters:
//!
//! 1. Owners - the set of addresses that sign transactions for the smart wallet.
//! 2. Threshold - the number of signers required to execute a transaction.
//! 3. Minimum Delay - the minimum amount of time that must pass before a [Transaction]
//!                    can be executed. If 0, this is ignored.
//!
//! Once the [SmartWallet] account is created, one can create a [Transaction]
//! account, specifying the parameters for a normal Solana instruction.
//!
//! To sign, owners should invoke the [smart_wallet::approve] instruction, and finally,
//! [smart_wallet::execute_transaction], once enough (i.e. [SmartWallet::threshold]) of the owners have
//! signed.
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]
#![deny(clippy::unwrap_used)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use vipers::prelude::*;

mod events;
mod instructions;
mod state;
mod validators;

pub use events::*;
pub use instructions::*;
pub use state::*;

/// Number of seconds in a day.
pub const SECONDS_PER_DAY: i64 = 60 * 60 * 24;

/// Maximum timelock delay.
pub const MAX_DELAY_SECONDS: i64 = 365 * SECONDS_PER_DAY;

/// Default number of seconds until a transaction expires.
pub const DEFAULT_GRACE_PERIOD: i64 = 14 * SECONDS_PER_DAY;

/// Constant declaring that there is no ETA of the transaction.
pub const NO_ETA: i64 = -1;

declare_id!("GokivDYuQXPZCWRkwMhdH2h91KpDQXBEmpgBgs55bnpH");

#[program]
/// Goki smart wallet program.
pub mod smart_wallet {
    use super::*;

    /// Initializes a new [SmartWallet] account with a set of owners and a threshold.
    #[access_control(ctx.accounts.validate())]
    pub fn create_smart_wallet(
        ctx: Context<CreateSmartWallet>,
        _bump: u8,
        max_owners: u8,
        owners: Vec<Pubkey>,
        threshold: u64,
        minimum_delay: i64,
    ) -> Result<()> {
        invariant!(minimum_delay >= 0, "delay must be positive");
        invariant!(minimum_delay < MAX_DELAY_SECONDS, DelayTooHigh);

        invariant!((max_owners as usize) >= owners.len(), "max_owners");

        let smart_wallet = &mut ctx.accounts.smart_wallet;
        smart_wallet.base = ctx.accounts.base.key();
        smart_wallet.bump = *unwrap_int!(ctx.bumps.get("smart_wallet"));

        smart_wallet.threshold = threshold;
        smart_wallet.minimum_delay = minimum_delay;
        smart_wallet.grace_period = DEFAULT_GRACE_PERIOD;

        smart_wallet.owner_set_seqno = 0;
        smart_wallet.num_transactions = 0;

        smart_wallet.owners = owners.clone();

        emit!(WalletCreateEvent {
            smart_wallet: ctx.accounts.smart_wallet.key(),
            owners,
            threshold,
            minimum_delay,
            timestamp: Clock::get()?.unix_timestamp
        });
        Ok(())
    }

    /// Sets the owners field on the smart_wallet. The only way this can be invoked
    /// is via a recursive call from execute_transaction -> set_owners.
    #[access_control(ctx.accounts.validate())]
    pub fn set_owners(ctx: Context<Auth>, owners: Vec<Pubkey>) -> Result<()> {
        let smart_wallet = &mut ctx.accounts.smart_wallet;
        if (owners.len() as u64) < smart_wallet.threshold {
            smart_wallet.threshold = owners.len() as u64;
        }

        smart_wallet.owners = owners.clone();
        smart_wallet.owner_set_seqno = unwrap_int!(smart_wallet.owner_set_seqno.checked_add(1));

        emit!(WalletSetOwnersEvent {
            smart_wallet: ctx.accounts.smart_wallet.key(),
            owners,
            timestamp: Clock::get()?.unix_timestamp
        });
        Ok(())
    }

    /// Changes the execution threshold of the smart_wallet. The only way this can be
    /// invoked is via a recursive call from execute_transaction ->
    /// change_threshold.
    #[access_control(ctx.accounts.validate())]
    pub fn change_threshold(ctx: Context<Auth>, threshold: u64) -> Result<()> {
        invariant!(
            threshold <= ctx.accounts.smart_wallet.owners.len() as u64,
            InvalidThreshold
        );
        let smart_wallet = &mut ctx.accounts.smart_wallet;
        smart_wallet.threshold = threshold;

        emit!(WalletChangeThresholdEvent {
            smart_wallet: ctx.accounts.smart_wallet.key(),
            threshold,
            timestamp: Clock::get()?.unix_timestamp
        });
        Ok(())
    }

    /// Creates a new [Transaction] account, automatically signed by the creator,
    /// which must be one of the owners of the smart_wallet.
    pub fn create_transaction(
        ctx: Context<CreateTransaction>,
        bump: u8,
        instructions: Vec<TXInstruction>,
    ) -> Result<()> {
        create_transaction_with_timelock(ctx, bump, instructions, NO_ETA)
    }

    /// Creates a new [Transaction] account with time delay.
    #[access_control(ctx.accounts.validate())]
    pub fn create_transaction_with_timelock(
        ctx: Context<CreateTransaction>,
        _bump: u8,
        instructions: Vec<TXInstruction>,
        eta: i64,
    ) -> Result<()> {
        let smart_wallet = &ctx.accounts.smart_wallet;
        let owner_index = smart_wallet.owner_index(ctx.accounts.proposer.key())?;

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
        let smart_wallet = &mut ctx.accounts.smart_wallet;
        smart_wallet.num_transactions = unwrap_int!(smart_wallet.num_transactions.checked_add(1));

        // init the TX
        let tx = &mut ctx.accounts.transaction;
        tx.smart_wallet = smart_wallet.key();
        tx.index = index;
        tx.bump = *unwrap_int!(ctx.bumps.get("transaction"));

        tx.proposer = ctx.accounts.proposer.key();
        tx.instructions = instructions.clone();
        tx.signers = signers;
        tx.owner_set_seqno = smart_wallet.owner_set_seqno;
        tx.eta = eta;

        tx.executor = Pubkey::default();
        tx.executed_at = -1;

        emit!(TransactionCreateEvent {
            smart_wallet: ctx.accounts.smart_wallet.key(),
            transaction: ctx.accounts.transaction.key(),
            proposer: ctx.accounts.proposer.key(),
            instructions,
            eta,
            timestamp: Clock::get()?.unix_timestamp
        });
        Ok(())
    }

    /// Approves a transaction on behalf of an owner of the smart_wallet.
    #[access_control(ctx.accounts.validate())]
    pub fn approve(ctx: Context<Approve>) -> Result<()> {
        let owner_index = ctx
            .accounts
            .smart_wallet
            .owner_index(ctx.accounts.owner.key())?;
        ctx.accounts.transaction.signers[owner_index] = true;

        emit!(TransactionApproveEvent {
            smart_wallet: ctx.accounts.smart_wallet.key(),
            transaction: ctx.accounts.transaction.key(),
            owner: ctx.accounts.owner.key(),
            timestamp: Clock::get()?.unix_timestamp
        });
        Ok(())
    }

    /// Unapproves a transaction on behalf of an owner of the smart_wallet.
    #[access_control(ctx.accounts.validate())]
    pub fn unapprove(ctx: Context<Approve>) -> Result<()> {
        let owner_index = ctx
            .accounts
            .smart_wallet
            .owner_index(ctx.accounts.owner.key())?;
        ctx.accounts.transaction.signers[owner_index] = false;

        emit!(TransactionUnapproveEvent {
            smart_wallet: ctx.accounts.smart_wallet.key(),
            transaction: ctx.accounts.transaction.key(),
            owner: ctx.accounts.owner.key(),
            timestamp: Clock::get()?.unix_timestamp
        });
        Ok(())
    }

    /// Executes the given transaction if threshold owners have signed it.
    #[access_control(ctx.accounts.validate())]
    pub fn execute_transaction(ctx: Context<ExecuteTransaction>) -> Result<()> {
        let smart_wallet = &ctx.accounts.smart_wallet;
        let wallet_seeds: &[&[&[u8]]] = &[&[
            b"GokiSmartWallet" as &[u8],
            &smart_wallet.base.to_bytes(),
            &[smart_wallet.bump],
        ]];
        do_execute_transaction(ctx, wallet_seeds)
    }

    /// Executes the given transaction signed by the given derived address,
    /// if threshold owners have signed it.
    /// This allows a Smart Wallet to receive SOL.
    #[access_control(ctx.accounts.validate())]
    pub fn execute_transaction_derived(
        ctx: Context<ExecuteTransaction>,
        index: u64,
        bump: u8,
    ) -> Result<()> {
        let smart_wallet = &ctx.accounts.smart_wallet;
        // Execute the transaction signed by the smart_wallet.
        let wallet_seeds: &[&[&[u8]]] = &[&[
            b"GokiSmartWalletDerived" as &[u8],
            &smart_wallet.key().to_bytes(),
            &index.to_le_bytes(),
            &[bump],
        ]];
        do_execute_transaction(ctx, wallet_seeds)
    }

    /// Invokes an arbitrary instruction as a PDA derived from the owner,
    /// i.e. as an "Owner Invoker".
    ///
    /// This is useful for using the multisig as a whitelist or as a council,
    /// e.g. a whitelist of approved owners.
    #[access_control(ctx.accounts.validate())]
    pub fn owner_invoke_instruction(
        ctx: Context<OwnerInvokeInstruction>,
        index: u64,
        bump: u8,
        ix: TXInstruction,
    ) -> Result<()> {
        let smart_wallet = &ctx.accounts.smart_wallet;
        // Execute the transaction signed by the smart_wallet.
        let invoker_seeds: &[&[&[u8]]] = &[&[
            b"GokiSmartWalletOwnerInvoker" as &[u8],
            &smart_wallet.key().to_bytes(),
            &index.to_le_bytes(),
            &[bump],
        ]];

        solana_program::program::invoke_signed(
            &(&ix).into(),
            ctx.remaining_accounts,
            invoker_seeds,
        )?;

        Ok(())
    }

    /// Invokes an arbitrary instruction as a PDA derived from the owner,
    /// i.e. as an "Owner Invoker".
    ///
    /// This is useful for using the multisig as a whitelist or as a council,
    /// e.g. a whitelist of approved owners.
    ///
    /// # Arguments
    /// - `index` - The index of the owner-invoker.
    /// - `bump` - Bump seed of the owner-invoker.
    /// - `invoker` - The owner-invoker.
    /// - `data` - The raw bytes of the instruction data.
    #[access_control(ctx.accounts.validate())]
    pub fn owner_invoke_instruction_v2(
        ctx: Context<OwnerInvokeInstruction>,
        index: u64,
        bump: u8,
        invoker: Pubkey,
        data: Vec<u8>,
    ) -> Result<()> {
        let smart_wallet = &ctx.accounts.smart_wallet;
        // Execute the transaction signed by the smart_wallet.
        let invoker_seeds: &[&[&[u8]]] = &[&[
            b"GokiSmartWalletOwnerInvoker" as &[u8],
            &smart_wallet.key().to_bytes(),
            &index.to_le_bytes(),
            &[bump],
        ]];

        let program_id = ctx.remaining_accounts[0].key();
        let accounts: Vec<AccountMeta> = ctx.remaining_accounts[1..]
            .iter()
            .map(|v| AccountMeta {
                pubkey: *v.key,
                is_signer: if v.key == &invoker { true } else { v.is_signer },
                is_writable: v.is_writable,
            })
            .collect();
        let ix = &solana_program::instruction::Instruction {
            program_id,
            accounts,
            data,
        };

        solana_program::program::invoke_signed(ix, ctx.remaining_accounts, invoker_seeds)?;
        Ok(())
    }

    /// Creates a struct containing a reverse mapping of a subaccount to a
    /// [SmartWallet].
    #[access_control(ctx.accounts.validate())]
    pub fn create_subaccount_info(
        ctx: Context<CreateSubaccountInfo>,
        _bump: u8,
        subaccount: Pubkey,
        smart_wallet: Pubkey,
        index: u64,
        subaccount_type: SubaccountType,
    ) -> Result<()> {
        let (address, _derived_bump) = match subaccount_type {
            SubaccountType::Derived => Pubkey::find_program_address(
                &[
                    b"GokiSmartWalletDerived" as &[u8],
                    &smart_wallet.to_bytes(),
                    &index.to_le_bytes(),
                ],
                &crate::ID,
            ),
            SubaccountType::OwnerInvoker => Pubkey::find_program_address(
                &[
                    b"GokiSmartWalletOwnerInvoker" as &[u8],
                    &smart_wallet.to_bytes(),
                    &index.to_le_bytes(),
                ],
                &crate::ID,
            ),
        };

        invariant!(address == subaccount, SubaccountOwnerMismatch);

        let info = &mut ctx.accounts.subaccount_info;
        info.smart_wallet = smart_wallet;
        info.subaccount_type = subaccount_type;
        info.index = index;

        Ok(())
    }

    #[access_control(ctx.accounts.validate())]
    pub fn init_ix_buffer(ctx: Context<InitIxBuffer>) -> Result<()> {
        instructions::buffer::handle_init(ctx)
    }

    #[access_control(ctx.accounts.validate())]
    pub fn close_ix_buffer(ctx: Context<CloseIxBuffer>) -> Result<()> {
        instructions::buffer::handle_close(ctx)
    }

    #[access_control(ctx.accounts.validate())]
    pub fn execute_ix_with_invoker<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteIx<'info>>,
        index: u64,
        bump: u8,
        smart_wallet: Pubkey,
    ) -> Result<()> {
        instructions::execute_ix::handle_with_invoker(ctx, index, bump, smart_wallet)
    }

    #[access_control(ctx.accounts.validate())]
    pub fn write_ix(ctx: Context<WriteIx>, ix: TXInstruction) -> Result<()> {
        instructions::write_ix::handler(ctx, ix)
    }

    #[access_control(ctx.accounts.validate())]
    pub fn set_buffer_role(ctx: Context<SetBufferRole>, role: u8, key: Pubkey) -> Result<()> {
        instructions::buffer::handle_set_role(ctx, role.try_into()?, key)
    }
}

/// Accounts for [smart_wallet::create_smart_wallet].
#[derive(Accounts)]
#[instruction(bump: u8, max_owners: u8)]
pub struct CreateSmartWallet<'info> {
    /// Base key of the SmartWallet.
    pub base: Signer<'info>,

    /// The [SmartWallet] to create.
    #[account(
        init,
        seeds = [
            b"GokiSmartWallet".as_ref(),
            base.key().to_bytes().as_ref()
        ],
        bump,
        payer = payer,
        space = SmartWallet::space(max_owners),
    )]
    pub smart_wallet: Account<'info, SmartWallet>,

    /// Payer to create the smart_wallet.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [smart_wallet::set_owners] and [smart_wallet::change_threshold].
#[derive(Accounts)]
pub struct Auth<'info> {
    /// The [SmartWallet].
    #[account(mut, signer)]
    pub smart_wallet: Account<'info, SmartWallet>,
}

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

/// Accounts for [smart_wallet::approve].
#[derive(Accounts)]
pub struct Approve<'info> {
    /// The [SmartWallet].
    pub smart_wallet: Account<'info, SmartWallet>,
    /// The [Transaction].
    #[account(mut)]
    pub transaction: Account<'info, Transaction>,
    /// One of the smart_wallet owners. Checked in the handler.
    pub owner: Signer<'info>,
}

/// Accounts for [smart_wallet::execute_transaction].
#[derive(Accounts)]
pub struct ExecuteTransaction<'info> {
    /// The [SmartWallet].
    pub smart_wallet: Account<'info, SmartWallet>,
    /// The [Transaction] to execute.
    #[account(mut)]
    pub transaction: Account<'info, Transaction>,
    /// An owner of the [SmartWallet].
    pub owner: Signer<'info>,
}

/// Accounts for [smart_wallet::owner_invoke_instruction].
#[derive(Accounts)]
pub struct OwnerInvokeInstruction<'info> {
    /// The [SmartWallet].
    pub smart_wallet: Account<'info, SmartWallet>,
    /// An owner of the [SmartWallet].
    pub owner: Signer<'info>,
}

/// Accounts for [smart_wallet::owner_invoke_instruction].
#[derive(Accounts)]
pub struct OwnerInvokeInstructionV2<'info> {
    /// The [SmartWallet].
    pub smart_wallet: Account<'info, SmartWallet>,
    /// An owner of the [SmartWallet].
    pub owner: Signer<'info>,
}

/// Accounts for [smart_wallet::create_subaccount_info].
#[derive(Accounts)]
#[instruction(bump: u8, subaccount: Pubkey)]
pub struct CreateSubaccountInfo<'info> {
    /// The [SubaccountInfo] to create.
    #[account(
        init,
        seeds = [
            b"GokiSubaccountInfo".as_ref(),
            &subaccount.to_bytes()
        ],
        bump,
        payer = payer
    )]
    pub subaccount_info: Account<'info, SubaccountInfo>,
    /// Payer to create the [SubaccountInfo].
    #[account(mut)]
    pub payer: Signer<'info>,
    /// The [System] program.
    pub system_program: Program<'info, System>,
}

fn do_execute_transaction(ctx: Context<ExecuteTransaction>, seeds: &[&[&[u8]]]) -> Result<()> {
    for ix in ctx.accounts.transaction.instructions.iter() {
        solana_program::program::invoke_signed(&(ix).into(), ctx.remaining_accounts, seeds)?;
    }

    // Burn the transaction to ensure one time use.
    let tx = &mut ctx.accounts.transaction;
    tx.executor = ctx.accounts.owner.key();
    tx.executed_at = Clock::get()?.unix_timestamp;

    emit!(TransactionExecuteEvent {
        smart_wallet: ctx.accounts.smart_wallet.key(),
        transaction: ctx.accounts.transaction.key(),
        executor: ctx.accounts.owner.key(),
        timestamp: Clock::get()?.unix_timestamp
    });
    Ok(())
}

/// Program errors.
#[error_code]
pub enum ErrorCode {
    #[msg("The given owner is not part of this smart wallet.")]
    InvalidOwner,
    #[msg("Estimated execution block must satisfy delay.")]
    InvalidETA,
    #[msg("Delay greater than the maximum.")]
    DelayTooHigh,
    #[msg("Not enough owners signed this transaction.")]
    NotEnoughSigners,
    #[msg("Transaction is past the grace period.")]
    TransactionIsStale,
    #[msg("Transaction hasn't surpassed time lock.")]
    TransactionNotReady,
    #[msg("The given transaction has already been executed.")]
    AlreadyExecuted,
    #[msg("Threshold must be less than or equal to the number of owners.")]
    InvalidThreshold,
    #[msg("Owner set has changed since the creation of the transaction.")]
    OwnerSetChanged,
    #[msg("Subaccount does not belong to smart wallet.")]
    SubaccountOwnerMismatch,
    #[msg("Invlaid buffer role.")]
    InvalidBufferRole,
}
