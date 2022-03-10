//! Appends an instruction to the [InstructionBuffer::bundle] at the bundle_index.

use crate::*;

#[derive(Accounts)]
pub struct AppendBufferIX<'info> {
    #[account(mut)]
    pub buffer: Box<Account<'info, InstructionBuffer>>,
    pub writer: Signer<'info>,
}

/// Emitted when an instruction is written to the [InstructionBuffer].
#[event]
pub struct AppendIxEvent {
    /// The index of the bundle at [Instruction::bundles].
    pub bundle_index: u8,
    /// The [InstructionBuffer].
    pub buffer: Pubkey,
    /// The [InstructionBuffer::writer].
    pub writer: Pubkey,
}

pub fn handler(ctx: Context<AppendBufferIX>, bundle_index: u8, ix: TXInstruction) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    let new_bundle = &mut match buffer.get_bundle(bundle_index) {
        Some(b) => b,
        None => InstructionBundle {
            instructions: [].to_vec(),
            exec_count: 0,
            finalized_at: 0,
        },
    };

    invariant!(!new_bundle.is_finalized(), BufferBundleFinalized);
    new_bundle.instructions.push(ix);

    let bundle = unwrap_opt!(buffer.bundles.get_mut(usize::from(bundle_index)));
    *bundle = new_bundle.clone();

    emit!(AppendIxEvent {
        bundle_index,
        buffer: buffer.key(),
        writer: buffer.writer,
    });

    Ok(())
}

impl<'info> Validate<'info> for AppendBufferIX<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.writer.key(), self.buffer.writer);

        Ok(())
    }
}
