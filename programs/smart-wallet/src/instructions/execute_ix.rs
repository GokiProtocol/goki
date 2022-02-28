//! Executes an instruction off of the [TXInstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct ExecuteIx {}

pub fn handler(_ctx: Context<ExecuteIx>) -> Result<()> {
    unimplemented!()
}
