//! Init or close an [InstructionBuffer].

use vipers::program_err;

use crate::*;

#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
pub enum BufferRole {
    Admin,
    Writer,
    Executer,
}

impl TryFrom<u8> for BufferRole {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(BufferRole::Admin),
            1 => Ok(BufferRole::Writer),
            2 => Ok(BufferRole::Executer),
            _ => program_err!(InvalidBufferRole),
        }
    }
}

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
pub struct SetBufferRole<'info> {
    #[account(mut)]
    pub buffer: Box<Account<'info, InstructionBuffer>>,
    pub admin: Signer<'info>,
}

pub fn handle_set_role(ctx: Context<SetBufferRole>, role: BufferRole, key: Pubkey) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;

    match role {
        BufferRole::Admin => buffer.admin = key,
        BufferRole::Writer => buffer.writer = key,
        BufferRole::Executer => {
            assert_keys_eq!(buffer.executor, Pubkey::default());
            buffer.executor = key
        }
    }

    Ok(())
}

impl<'info> Validate<'info> for SetBufferRole<'info> {
    fn validate(&self) -> Result<()> {
        invariant!(self.buffer.exec_count == 0);
        assert_keys_eq!(self.buffer.admin, self.admin);

        Ok(())
    }
}
