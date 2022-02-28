//! Executes an instruction off of the [TXInstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct ExecuteIx {}

pub fn handler<'info>(_ctx: Context<ExecuteIx>) -> Result<()> {
    unimplemented!();
}
