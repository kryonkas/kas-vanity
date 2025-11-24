//! Module: Vanity
//! But: Core logic for mnemonic generation and address derivation.
//!
//! *Signed: kryon.kas*

use bip39::Mnemonic;
use bip32::XPrv;
use kaspa_addresses::{Address, Prefix};
use rand::RngCore;

// --- Génération ---

/// Generates a random BIP-39 mnemonic.
pub fn generate_random_mnemonic(word_count: usize) -> Mnemonic {
    let mut rng = rand::rng();
    
    // Entropy selection:
    // 12 words = 128 bits
    // 24 words = 256 bits
    let entropy_len = match word_count {
        12 => 16,
        24 => 32,
        _ => 32, // Défaut: 24 words
    };
    
    let mut entropy = vec![0u8; entropy_len];
    rng.fill_bytes(&mut entropy);

    Mnemonic::from_entropy(&entropy).expect("Failed to generate mnemonic")
}

// --- Dérivation ---

/// Derives a batch of Kaspa addresses from a mnemonic.
/// Optimizes performance by deriving the account key once and iterating indices.
pub fn derive_batch(mnemonic: &Mnemonic, limit: u32) -> Vec<(u32, Address)> {
    // Seed generation (no passphrase)
    let seed = mnemonic.to_seed("");

    // Master extended private key
    let Ok(xprv) = XPrv::new(seed) else { return vec![] };

    // Kaspa Account Derivation Path: m/44'/111111'/0'/0
    let path = "m/44'/111111'/0'/0";
    let Ok(derivation_path) = bip32::DerivationPath::from_str(path) else { return vec![] };

    // Derive the account/chain extended private key
    let mut account_xprv = xprv;
    for child in derivation_path {
        if let Ok(child_key) = account_xprv.derive_child(child) {
            account_xprv = child_key;
        } else {
            return vec![];
        }
    }

    let mut results = Vec::with_capacity(limit as usize);

    for index in 0..limit {
        // Derive final child (Address Index)
        if let Ok(child_xprv) = account_xprv.derive_child(bip32::ChildNumber::new(index, false).unwrap()) {
            // Public Key Extraction
            let extended_pubkey = child_xprv.public_key();
            let public_key = extended_pubkey.public_key();

            // Compression
            let compressed_pubkey = public_key.to_encoded_point(true);
            let compressed_bytes = compressed_pubkey.as_bytes();

            // X-Only Public Key (Schnorr)
            let x_only_pubkey = &compressed_bytes[1..];

            // Address Creation
            let address = Address::new(Prefix::Mainnet, kaspa_addresses::Version::PubKey, x_only_pubkey);
            
            results.push((index, address));
        }
    }

    results
}



// --- Helpers ---

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_generation() {
        let mnemonic = generate_random_mnemonic(12);
        assert_eq!(mnemonic.word_count(), 12);
        
        let mnemonic = generate_random_mnemonic(24);
        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn test_derive_batch() {
        let mnemonic = generate_random_mnemonic(12);
        let results = derive_batch(&mnemonic, 10);
        
        assert_eq!(results.len(), 10);
        for (i, (index, address)) in results.iter().enumerate() {
            assert_eq!(*index, i as u32);
            assert!(address.to_string().starts_with("kaspa:"));
        }
    }
}
