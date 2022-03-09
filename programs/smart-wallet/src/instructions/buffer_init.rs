//! Init or close an [InstructionLoader].

use crate::*;

#[derive(Accounts)]
pub struct InitBuffer<'info> {
    #[account(zero)]
    pub buffer: Account<'info, InstructionLoader>,
    pub smart_wallet: Account<'info, SmartWallet>,
}

/// Emitted when a [InstructionLoader] is initialized.
#[event]
pub struct InitBufferEvent {
    /// The buffer.
    pub buffer: Pubkey,
    #[index]
    /// The [InstructionLoader::smart_wallet].
    pub smart_wallet: Pubkey,
}

pub fn handler(ctx: Context<InitBuffer>, eta: i64, writer: Pubkey, executer: Pubkey) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.eta = eta;
    buffer.owner_set_seqno = ctx.accounts.smart_wallet.owner_set_seqno;
    buffer.executer = executer;
    buffer.smart_wallet = ctx.accounts.smart_wallet.key();
    buffer.writer = writer;

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
