//! Appends an instruction to the [InstructionBuffer::bundle] at the bundle_index.

use crate::*;

#[derive(Accounts)]
pub struct AppendBufferIX<'info> {
    #[account(mut)]
    pub buffer: Box<Account<'info, InstructionBuffer>>,
    pub authority: Signer<'info>,
}

/// Emitted when an instruction is written to the [InstructionBuffer].
#[event]
pub struct AppendIxEvent {
    /// The index of the bundle at [Instruction::bundles].
    pub bundle_index: u8,
    /// The [InstructionBuffer].
    pub buffer: Pubkey,
    /// The [InstructionBuffer::authority].
    pub authority: Pubkey,
}

pub fn handler(ctx: Context<AppendBufferIX>, bundle_index: u8, ix: TXInstruction) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    let new_bundle = &mut match buffer.get_bundle(bundle_index) {
        Some(b) => b,
        None => InstructionBundle {
            instructions: [].to_vec(),
            exec_count: 0,
        },
    };

    new_bundle.instructions.push(ix);

    buffer.set_bundle(bundle_index, &new_bundle)?;

    emit!(AppendIxEvent {
        bundle_index,
        buffer: buffer.key(),
        authority: buffer.authority,
    });

    Ok(())
}

impl<'info> Validate<'info> for AppendBufferIX<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.authority, self.buffer.authority);
        invariant!(!self.buffer.is_finalized(), BufferFinalized);

        Ok(())
    }
}
