//! Init or close an [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct InitBuffer<'info> {
    #[account(init, seeds = [b"GokiBuffer".as_ref(), transaction.key.as_ref()], bump, payer = payer)]
    pub buffer: Account<'info, InstructionBuffer>,
    /// The [SmartWallet].
    #[account(mut)]
    pub smart_wallet: Account<'info, SmartWallet>,
    /// The [Transaction]
    #[account(zero)]
    pub transaction: Account<'info, Transaction>,
    /// One of the owners. Checked in the handler via [SmartWallet::owner_index].
    pub proposer: Signer<'info>,
    /// CHECK: Arbitrary account that can write to the buffer.
    pub writer: UncheckedAccount<'info>,
    /// Payer to create the [Transaction].
    #[account(mut)]
    pub payer: Signer<'info>,
    /// The [System] program.
    pub system_program: Program<'info, System>,
}

/// Emitted when a [InstructionBuffer] is initialized.
#[event]
pub struct InitBufferEvent {
    /// The buffer.
    pub buffer: Pubkey,
    /// The [InstructionBuffer::writer].
    #[index]
    pub writer: Pubkey,
}

pub fn handle(ctx: Context<InitBuffer>) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.writer = ctx.accounts.writer.key();
    buffer.transaction = ctx.accounts.transaction.key();

    emit!(InitBufferEvent {
        buffer: buffer.key(),
        writer: buffer.writer,
    });

    Ok(())
}

impl<'info> Validate<'info> for InitBuffer<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}
