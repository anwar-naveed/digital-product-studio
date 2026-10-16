//! AES-256-GCM encryption for platform credentials at rest.
//!
//! Key source: CREDENTIALS_ENCRYPTION_KEY env — 64 hex chars (32 bytes).
//! If unset, the API refuses credential writes (fail-closed, never plaintext).

use ring::{
    aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN},
    error::Unspecified,
};
use rand::rngs::OsRng;
use rand::RngCore;

pub struct CredentialCipher {
    key: LessSafeKey,
}

#[derive(Debug)]
pub enum CipherError {
    KeyNotConfigured,
    BadKey,
    Sealing(Unspecified),
}

impl std::fmt::Display for CipherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CipherError::KeyNotConfigured => write!(f, "CREDENTIALS_ENCRYPTION_KEY not configured"),
            CipherError::BadKey => write!(f, "CREDENTIALS_ENCRYPTION_KEY must be 64 hex chars (32 bytes)"),
            CipherError::Sealing(_) => write!(f, "encryption failed"),
        }
    }
}

impl CredentialCipher {
    pub fn from_env() -> Result<Self, CipherError> {
        let hex = std::env::var("CREDENTIALS_ENCRYPTION_KEY")
            .map_err(|_| CipherError::KeyNotConfigured)?;
        let bytes = hex_to_bytes(&hex).ok_or(CipherError::BadKey)?;
        let unbound = UnboundKey::new(&AES_256_GCM, &bytes).map_err(|_| CipherError::BadKey)?;
        Ok(Self { key: LessSafeKey::new(unbound) })
    }

    /// Returns (nonce, ciphertext-with-tag).
    pub fn seal(&self, plaintext: &str) -> Result<(Vec<u8>, Vec<u8>), CipherError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = plaintext.as_bytes().to_vec();
        self.key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(CipherError::Sealing)?;
        Ok((nonce_bytes.to_vec(), in_out))
    }

    pub fn open(&self, nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CipherError> {
        if nonce.len() != NONCE_LEN {
            return Err(CipherError::BadKey);
        }
        let mut nonce_bytes = [0u8; NONCE_LEN];
        nonce_bytes.copy_from_slice(nonce);
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = ciphertext.to_vec();
        let plain = self
            .key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(CipherError::Sealing)?;
        Ok(plain.to_vec())
    }
}

fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
    if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    (0..32)
        .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok())
        .collect()
}
