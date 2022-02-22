//! Events emitted.
#![deny(missing_docs)]

use crate::*;

/// Emitted when a [SmartWallet] is created.
#[event]
pub struct WalletCreateEvent {
    /// The [SmartWallet].
    #[index]
    pub smart_wallet: Pubkey,
    /// The owners of the created [SmartWallet].
    pub owners: Vec<Pubkey>,
    /// The [SmartWallet::threshold] at the time of creation.
    pub threshold: u64,
    /// The [SmartWallet::minimum_delay] at the time of creation.
    pub minimum_delay: i64,
    /// The Unix timestamp when the event was emitted.
    pub timestamp: i64,
}

/// Emitted when the owners of a [SmartWallet] are changed.
#[event]
pub struct WalletSetOwnersEvent {
    /// The [SmartWallet].
    #[index]
    pub smart_wallet: Pubkey,
    /// The new owners of the [SmartWallet].
    pub owners: Vec<Pubkey>,
    /// The Unix timestamp when the event was emitted.
    pub timestamp: i64,
}

/// Emitted when the threshold of a [SmartWallet] is changed.
#[event]
pub struct WalletChangeThresholdEvent {
    /// The [SmartWallet].
    #[index]
    pub smart_wallet: Pubkey,
    /// The new [SmartWallet::threshold].
    pub threshold: u64,
    /// The Unix timestamp when the event was emitted.
    pub timestamp: i64,
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

/// Emitted when a [Transaction] is approved.
#[event]
pub struct TransactionApproveEvent {
    /// The [SmartWallet].
    #[index]
    pub smart_wallet: Pubkey,
    /// The [Transaction].
    #[index]
    pub transaction: Pubkey,
    /// The owner which approved the transaction.
    pub owner: Pubkey,
    /// The Unix timestamp when the event was emitted.
    pub timestamp: i64,
}

/// Emitted when a [Transaction] is unapproved.
#[event]
pub struct TransactionUnapproveEvent {
    /// The [SmartWallet].
    #[index]
    pub smart_wallet: Pubkey,
    /// The [Transaction].
    #[index]
    pub transaction: Pubkey,
    /// The owner that unapproved the transaction.
    pub owner: Pubkey,
    /// The Unix timestamp when the event was emitted.
    pub timestamp: i64,
}

/// Emitted when a [Transaction] is executed.
#[event]
pub struct TransactionExecuteEvent {
    /// The [SmartWallet].
    #[index]
    pub smart_wallet: Pubkey,
    /// The [Transaction] executed.
    #[index]
    pub transaction: Pubkey,
    /// The owner that executed the transaction.
    pub executor: Pubkey,
    /// The Unix timestamp when the event was emitted.
    pub timestamp: i64,
}
