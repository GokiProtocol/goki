//! Executes an instruction off of the [InstructionBuffer].
use crate::*;

#[derive(Accounts)]
pub struct SetBufferExecuter<'info> {
    #[account(mut)]
    pub buffer: Box<Account<'info, InstructionBuffer>>,
    pub writer: Signer<'info>,
}

pub fn handler<'info>(ctx: Context<SetBufferExecuter>, executer: Pubkey) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.executor = executer;

    Ok(())
}

impl<'info> Validate<'info> for SetBufferExecuter<'info> {
    fn validate(&self) -> Result<()> {
        invariant!(self.buffer.exec_count == 0);
        assert_keys_eq!(self.buffer.executor, Pubkey::default());

        Ok(())
    }
}
