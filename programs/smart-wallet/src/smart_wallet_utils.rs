use crate::*;

impl SmartWallet {
    /// Gets the index of the key in the owners Vec, or error
    pub fn owner_index(&self, key: Pubkey) -> Result<usize> {
        Ok(unwrap_or_err!(
            self.owners.iter().position(|a| *a == key),
            InvalidOwner
        ))
    }

    pub fn validate_eta(&self, current_ts: i64, eta: i64) -> ProgramResult {
        require!(
            eta >= unwrap_int!(current_ts.checked_add(self.minimum_delay as i64)),
            InvalidETA
        );
        Ok(())
    }
}
