//! State structs.
#![deny(missing_docs)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use vipers::unwrap_or_err;

/// A [SmartWallet] is a multisig wallet with Timelock capabilities.
#[account]
#[derive(Default, Debug, PartialEq)]
pub struct SmartWallet {
    /// Base used to derive.
    pub base: Pubkey,
    /// Bump seed for deriving PDA seeds.
    pub bump: u8,

    /// Minimum number of owner approvals needed to sign a [Transaction].
    pub threshold: u64,
    /// Minimum delay between approval and execution, in seconds.
    pub minimum_delay: i64,
    /// Time after the ETA until a [Transaction] expires.
    pub grace_period: i64,

    /// Sequence of the ownership set.
    ///
    /// This may be used to see if the owners on the multisig have changed
    /// since the last time the owners were checked. This is used on
    /// [Transaction] approval to ensure that owners cannot approve old
    /// transactions.
    pub owner_set_seqno: u32,
    /// Total number of [Transaction]s on this [SmartWallet].
    pub num_transactions: u64,

    /// Owners of the [SmartWallet].
    pub owners: Vec<Pubkey>,

    /// Extra space for program upgrades.
    pub reserved: [u64; 16],
}

impl SmartWallet {
    /// Computes the space a [SmartWallet] uses.
    pub fn space(max_owners: u8) -> usize {
        4 // Anchor discriminator
            + std::mem::size_of::<SmartWallet>()
            + 4 // 4 = the Vec discriminator
            + std::mem::size_of::<Pubkey>() * (max_owners as usize)
    }

    /// Gets the index of the key in the owners Vec, or error
    pub fn owner_index(&self, key: Pubkey) -> crate::Result<usize> {
        Ok(unwrap_or_err!(
            self.owners.iter().position(|a| *a == key),
            InvalidOwner
        ))
    }
}

/// A [Transaction] is a series of instructions that may be executed
/// by a [SmartWallet].
#[account]
#[derive(Debug, Default, PartialEq)]
pub struct Transaction {
    /// The [SmartWallet] account this transaction belongs to.
    pub smart_wallet: Pubkey,
    /// The auto-incremented integer index of the transaction.
    /// All transactions on the [SmartWallet] can be looked up via this index,
    /// allowing for easier browsing of a wallet's historical transactions.
    pub index: u64,
    /// Bump seed.
    pub bump: u8,

    /// The proposer of the [Transaction].
    pub proposer: Pubkey,
    /// The instruction.
    pub instructions: Vec<TXInstruction>,
    /// `signers[index]` is true iff `[SmartWallet]::owners[index]` signed the transaction.
    pub signers: Vec<bool>,
    /// Owner set sequence number.
    pub owner_set_seqno: u32,
    /// Estimated time the [Transaction] will be executed.
    ///
    /// - If set to [crate::NO_ETA], the transaction may be executed at any time.
    /// - Otherwise, the [Transaction] may be executed at any point after the ETA has elapsed.
    pub eta: i64,

    /// The account that executed the [Transaction].
    pub executor: Pubkey,
    /// When the transaction was executed. -1 if not executed.
    pub executed_at: i64,
}

impl Transaction {
    /// Computes the space a [Transaction] uses.
    pub fn space(instructions: Vec<TXInstruction>) -> usize {
        4  // Anchor discriminator
            + std::mem::size_of::<Transaction>()
            + 4 // Vec discriminator
            + (instructions.iter().map(|ix| ix.space()).sum::<usize>())
    }

    /// Number of signers.
    pub fn num_signers(&self) -> usize {
        self.signers.iter().filter(|&did_sign| *did_sign).count()
    }
}

/// Instruction.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq)]
pub struct TXInstruction {
    /// Pubkey of the instruction processor that executes this instruction
    pub program_id: Pubkey,
    /// Metadata for what accounts should be passed to the instruction processor
    pub keys: Vec<TXAccountMeta>,
    /// Opaque data passed to the instruction processor
    pub data: Vec<u8>,
}

impl TXInstruction {
    /// Space that a [TXInstruction] takes up.
    pub fn space(&self) -> usize {
        std::mem::size_of::<Pubkey>()
            + (self.keys.len() as usize) * std::mem::size_of::<TXAccountMeta>()
            + (self.data.len() as usize)
    }
}

/// Account metadata used to define [TXInstruction]s
#[derive(AnchorSerialize, AnchorDeserialize, Debug, PartialEq, Copy, Clone)]
pub struct TXAccountMeta {
    /// An account's public key
    pub pubkey: Pubkey,
    /// True if an Instruction requires a Transaction signature matching `pubkey`.
    pub is_signer: bool,
    /// True if the `pubkey` can be loaded as a read-write account.
    pub is_writable: bool,
}

impl From<&TXInstruction> for solana_program::instruction::Instruction {
    fn from(tx: &TXInstruction) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: tx.program_id,
            accounts: tx.keys.clone().into_iter().map(Into::into).collect(),
            data: tx.data.clone(),
        }
    }
}

impl From<TXAccountMeta> for solana_program::instruction::AccountMeta {
    fn from(
        TXAccountMeta {
            pubkey,
            is_signer,
            is_writable,
        }: TXAccountMeta,
    ) -> solana_program::instruction::AccountMeta {
        solana_program::instruction::AccountMeta {
            pubkey,
            is_signer,
            is_writable,
        }
    }
}

/// Type of Subaccount.
#[derive(
    AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord,
)]
#[repr(u8)]
pub enum SubaccountType {
    /// Requires the normal multisig approval process.
    Derived = 0,
    /// Any owner may sign an instruction  as this address.
    OwnerInvoker = 1,
}

impl Default for SubaccountType {
    fn default() -> Self {
        SubaccountType::Derived
    }
}

/// Mapping of a Subaccount to its [SmartWallet].
#[account]
#[derive(Copy, Default, Debug, PartialEq, Eq)]
pub struct SubaccountInfo {
    /// Smart wallet of the sub-account.
    pub smart_wallet: Pubkey,
    /// Type of sub-account.
    pub subaccount_type: SubaccountType,
    /// Index of the sub-account.
    pub index: u64,
}

/// An account which holds the data of a single [TXInstruction].
/// Creating this allows an owner-invoker to execute a transaction
/// with a minimal transaction size.
#[account]
#[derive(Default, Debug, PartialEq)]
pub struct StagedTXInstruction {
    /// The [SmartWallet] to execute this on.
    pub smart_wallet: Pubkey,
    /// The owner-invoker index.
    pub index: u64,
    /// Bump seed of the owner-invoker.
    pub owner_invoker_bump: u8,

    /// The owner which will execute the instruction.
    pub owner: Pubkey,
    /// Owner set sequence number.
    pub owner_set_seqno: u32,

    /// The instruction to execute.
    pub ix: TXInstruction,
}
