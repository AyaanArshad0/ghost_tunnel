use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce, Key
};
use anyhow::{Result, anyhow};

/// Wrapper around ChaCha20Poly1305 AEAD.
/// 
/// **AEAD Selection Rationale**:
/// We utilize ChaCha20Poly1305 over AES-GCM for two primary reasons:
/// 1. **Performance**: Superior throughput on ARMv8/mobile architecture lacking specialized AES extensions.
/// 2. **Security**: Constant-time execution in software prevents cache-timing side channels.
pub struct SessionGuard {
    cipher: ChaCha20Poly1305,
}

impl SessionGuard {
    /// Initialize the session security context.
    /// 
    /// FIXME: Hardcoded for prototype. Integrate Diffie-Hellman (Noise IK) for production
    /// to ensure Perfect Forward Secrecy (PFS) and eliminate static key distribution.
    pub fn new(key_bytes: &[u8; 32]) -> Self {
        let key = Key::from_slice(key_bytes);
        let cipher = ChaCha20Poly1305::new(key);
        Self { cipher }
    }

    /// Encrypts data into a wire-ready packet.
    /// Packet Structure: `[NONCE (12B) | CIPHERTEXT (N) | TAG (16B)]`
    /// Note: The Poly1305 tag is appended automatically by the AEAD crate.
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Unique nonce generation per packet to strictly strictly prevent key-stream reuse.
        // Trade-off: 12-byte expansion per frame vs. stateful counter synchronization execution complexity.
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); 
        
        let ciphertext = self.cipher.encrypt(&nonce, data)
            .map_err(|e| anyhow!("Encryption Failure: {}", e))?;
        
        // Prefix nonce to allow stateless decryption by the receiver
        let mut packet = nonce.to_vec();
        packet.extend(ciphertext);
        
        Ok(packet)
    }

    /// Decrypts a wire packet.
    /// Expects: `[NONCE (12B) | ...]`
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(anyhow!("Protocol Violation: Insufficient packet length ({} bytes)", data.len()));
        }

        let nonce = Nonce::from_slice(&data[0..12]);
        let ciphertext = &data[12..];

        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption Failure: {}", e))?;

        Ok(plaintext)
    }
}
