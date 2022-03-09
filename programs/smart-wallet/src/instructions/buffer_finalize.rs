//! Init or close an [InstructionLoader].

use crate::*;

#[derive(Accounts)]
pub struct FinalizeBuffer<'info> {
    #[account(mut)]
    pub buffer: Account<'info, InstructionLoader>,
    pub writer: Signer<'info>,
}

/// Emitted when a [InstructionLoader] is initialized.
#[event]
pub struct FinalizeBufferEvent {
    /// The buffer.
    pub buffer: Pubkey,
    /// The time [InstructionLoader] is finalized.
    #[index]
    pub time: i64,
}

pub fn handler(ctx: Context<FinalizeBuffer>) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.finalized_at = Clock::get()?.unix_timestamp;

    emit!(FinalizeBufferEvent {
        buffer: buffer.key(),
        time: buffer.finalized_at,
    });

    Ok(())
}

impl<'info> Validate<'info> for FinalizeBuffer<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.buffer.writer, self.writer);
        invariant!(self.buffer.finalized_at == 0);

        Ok(())
    }
}
