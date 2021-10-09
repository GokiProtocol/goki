use crate::*;

impl Transaction {
    /// Number of signers.
    pub fn num_signers(&self) -> usize {
        self.signers.iter().filter(|&did_sign| *did_sign).count()
    }
}
