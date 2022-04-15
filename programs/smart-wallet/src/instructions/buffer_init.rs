//! Initializes an [InstructionBuffer].

use crate::*;

/// Accounts for [smart_wallet::init_ix_buffer].
#[derive(Accounts)]
pub struct InitBuffer<'info> {
    #[account(zero)]
    /// The [InstructionBuffer].
    pub buffer: Account<'info, InstructionBuffer>,
    /// The [InstructionBuffer::smart_wallet].
    pub smart_wallet: Account<'info, SmartWallet>,
    /// CHECK: Arbitrary account allowed.
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Arbitrary account allowed.
    pub executor: UncheckedAccount<'info>,
}

/// Emitted when a [InstructionBuffer] is initialized.
#[event]
pub struct InitBufferEvent {
    /// The buffer.
    pub buffer: Pubkey,
    #[index]
    /// The [InstructionBuffer::smart_wallet].
    pub smart_wallet: Pubkey,
}

/// Handler for [smart_wallet::init_ix_buffer].
pub fn handle_init(ctx: Context<InitBuffer>, eta: i64) -> Result<()> {
    init_internal(ctx.accounts, eta, 0)
}

/// Handler for [smart_wallet::init_ix_buffer_with_bundles].
pub fn handle_init_with_bundles(ctx: Context<InitBuffer>, eta: i64, num_bundles: u8) -> Result<()> {
    init_internal(ctx.accounts, eta, num_bundles)
}

fn init_internal(accounts: &mut InitBuffer, eta: i64, num_bundles: u8) -> Result<()> {
    let buffer = &mut accounts.buffer;
    buffer.eta = eta;
    buffer.owner_set_seqno = accounts.smart_wallet.owner_set_seqno;

    buffer.executor = accounts.executor.key();
    buffer.authority = accounts.authority.key();
    buffer.smart_wallet = accounts.smart_wallet.key();

    if num_bundles > 0 {
        buffer
            .bundles
            .resize(usize::from(num_bundles), InstructionBundle::default());
    }

    emit!(InitBufferEvent {
        buffer: buffer.key(),
        smart_wallet: buffer.smart_wallet,
    });

    Ok(())
}

impl<'info> Validate<'info> for InitBuffer<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_neq!(self.executor, Pubkey::default());

        Ok(())
    }
}
