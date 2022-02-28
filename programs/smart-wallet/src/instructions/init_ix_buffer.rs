//! Creates a [TXInstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct InitIxBuffer {}

pub fn handler<'info>(_ctx: Context<InitIxBuffer>) -> Result<()> {
    unimplemented!();
}
