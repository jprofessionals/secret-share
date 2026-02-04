use bip39::Mnemonic;
use rand::Rng;

use crate::error::AppError;

/// Generate a 3-word passphrase using BIP39 wordlist
pub fn generate_passphrase() -> Result<String, AppError> {
    // Generate 16 bytes of entropy for a 12-word mnemonic
    let mut rng = rand::rng();
    let entropy: [u8; 16] = rng.random();

    // Create mnemonic from entropy and take first 3 words for passphrase
    let mnemonic = Mnemonic::from_entropy(&entropy)?;
    let words: Vec<&str> = mnemonic.words().take(3).collect();

    Ok(words.join("-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_passphrase() {
        let passphrase = generate_passphrase().unwrap();
        let parts: Vec<&str> = passphrase.split('-').collect();

        assert_eq!(parts.len(), 3);
    }
}
