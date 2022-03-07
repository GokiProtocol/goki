//! Init or close an [InstructionBuffer].

use crate::*;

#[derive(Accounts)]
pub struct InitIxBuffer<'info> {
    #[account(zero)]
    pub buffer: Account<'info, InstructionBuffer>,
    /// CHECK: This can be an arbitrary account.
    pub admin: UncheckedAccount<'info>,
}

/// Emitted when a [InstructionBuffer] is initialized.
#[event]
pub struct InitBufferEvent {
    /// The [InstructionBuffer::writer].
    #[index]
    pub admin: Pubkey,
    /// The buffer.
    pub buffer: Pubkey,
}

pub fn handle_init(ctx: Context<InitIxBuffer>) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.admin = ctx.accounts.admin.key();
    buffer.writer = ctx.accounts.admin.key();

    emit!(InitBufferEvent {
        admin: buffer.admin,
        buffer: buffer.key()
    });

    Ok(())
}

impl<'info> Validate<'info> for InitIxBuffer<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CloseIxBuffer<'info> {
    #[account(mut, close = writer)]
    pub buffer: Account<'info, InstructionBuffer>,
    pub writer: Signer<'info>,
}

/// Emitted when an [InstructionBuffer] is closed.
#[event]
pub struct CloseBufferEvent {
    /// The [InstructionBuffer::writer].
    #[index]
    pub writer: Pubkey,
    /// The buffer.
    pub buffer: Pubkey,
}

pub fn handle_close(ctx: Context<CloseIxBuffer>) -> Result<()> {
    emit!(CloseBufferEvent {
        writer: ctx.accounts.writer.key(),
        buffer: ctx.accounts.buffer.key(),
    });
    Ok(())
}

impl<'info> Validate<'info> for CloseIxBuffer<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.writer.key(), self.buffer.writer);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetBufferExecuter<'info> {
    #[account(mut)]
    pub buffer: Box<Account<'info, InstructionBuffer>>,
    pub admin: Signer<'info>,
}

pub fn handle_set_executor<'info>(ctx: Context<SetBufferExecuter>, executer: Pubkey) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.executor = executer;

    Ok(())
}

impl<'info> Validate<'info> for SetBufferExecuter<'info> {
    fn validate(&self) -> Result<()> {
        invariant!(self.buffer.exec_count == 0);
        assert_keys_eq!(self.buffer.admin, self.admin);
        assert_keys_eq!(self.buffer.executor, Pubkey::default());

        Ok(())
    }
}
