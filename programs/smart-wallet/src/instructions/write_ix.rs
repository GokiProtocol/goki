//! Writes an instruction to the [TXInstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct WriteIx {}

pub fn handler<'info>(_ctx: Context<WriteIx>) -> Result<()> {
    unimplemented!();
}
