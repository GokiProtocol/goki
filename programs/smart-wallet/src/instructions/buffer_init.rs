//! Initializes an [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct InitBuffer<'info> {
    #[account(zero)]
    pub buffer: Account<'info, InstructionBuffer>,
    pub smart_wallet: Account<'info, SmartWallet>,
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

pub fn handler(
    ctx: Context<InitBuffer>,
    eta: i64,
    admin: Pubkey,
    writer: Pubkey,
    executer: Pubkey,
) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.eta = eta;
    buffer.owner_set_seqno = ctx.accounts.smart_wallet.owner_set_seqno;

    buffer.admin = admin;
    buffer.executer = executer;
    buffer.writer = writer;

    buffer.smart_wallet = ctx.accounts.smart_wallet.key();

    emit!(InitBufferEvent {
        buffer: buffer.key(),
        smart_wallet: buffer.smart_wallet,
    });

    Ok(())
}

impl<'info> Validate<'info> for InitBuffer<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}
