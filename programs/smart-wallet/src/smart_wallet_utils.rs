use vipers::unwrap_opt;

use crate::*;

impl SmartWallet {
    /// Gets the index of the key in the owners Vec, or error
    pub fn owner_index(&self, key: Pubkey) -> Result<usize> {
        Ok(unwrap_opt!(
            self.owners.iter().position(|a| *a == key),
            InvalidOwner
        ))
    }
}
