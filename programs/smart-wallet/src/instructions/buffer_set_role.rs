use vipers::program_err;

use crate::*;

/// Side of a vote.
#[derive(Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum BufferRoles {
    /// Unknown buffer role.
    Unknown = 0,
    /// The admin role to with permission to set the [InstructionBuffer::writer] or [InstructionBuffer::executer].
    Admin = 1,
    /// Writer role to write to the [InstructionBuffer].
    Writer = 2,
    /// Executer role to execute instructions off the [InstructionBuffer].
    Executer = 3,
}

impl From<BufferRoles> for u8 {
    fn from(role: BufferRoles) -> Self {
        role as u8
    }
}

impl From<u8> for BufferRoles {
    fn from(raw_role: u8) -> BufferRoles {
        match raw_role {
            1 => BufferRoles::Admin,
            2 => BufferRoles::Writer,
            3 => BufferRoles::Executer,
            _ => BufferRoles::Unknown,
        }
    }
}

#[derive(Accounts)]
pub struct SetBufferRole<'info> {
    #[account(mut)]
    pub buffer: Account<'info, InstructionBuffer>,
    pub admin: Signer<'info>,
}

/// Emitted when an instruction is written to the [InstructionBuffer].
#[event]
pub struct SetBufferRoleEvent {
    /// The [InstructionBuffer].
    pub buffer: Pubkey,
    /// The buffer role that was set.
    pub role: u8,
    /// The old key for the role.
    pub old_role_key: Pubkey,
    /// The new key for the role.
    pub new_role_key: Pubkey,
}

pub fn handler(ctx: Context<SetBufferRole>, role: BufferRoles, role_key: Pubkey) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    let mut old_role_key = buffer.admin;

    match role {
        BufferRoles::Admin => buffer.admin = role_key,
        BufferRoles::Writer => {
            old_role_key = buffer.writer;
            buffer.writer = role_key;
        }
        BufferRoles::Executer => {
            old_role_key = buffer.executer;
            buffer.executer = role_key;
        }
        BufferRoles::Unknown => return program_err!(BufferRoleInvalid),
    }

    emit!(SetBufferRoleEvent {
        buffer: buffer.key(),
        role: role.into(),
        old_role_key,
        new_role_key: role_key
    });

    Ok(())
}

impl<'info> Validate<'info> for SetBufferRole<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.admin, self.buffer.admin);
        invariant!(!self.buffer.finalized_at == 0);

        Ok(())
    }
}
