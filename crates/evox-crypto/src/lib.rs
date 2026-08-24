// crates/evox-crypto/src/lib.rs

pub use ed25519_dalek::{Signature, Signer, Verifier, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Key generation error")]
    KeyGenError,
}

/// ساختاری برای نگهداری جفت کلیدها (برای استفاده در تست و مدیریت محلی)
pub struct KeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

/// تولید یک جفت کلید جدید با استفاده از Random Number Generator سیستم
pub fn generate_keypair() -> KeyPair {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    KeyPair {
        signing_key,
        verifying_key,
    }
}

/// امضا کردن یک پیام (یا داده‌های تراکنش)
pub fn sign(signing_key: &SigningKey, message: &[u8]) -> Signature {
    signing_key.sign(message)
}

/// تأیید صحت امضا با استفاده از کلید عمومی
pub fn verify(verifying_key: &VerifyingKey, message: &[u8], signature: &Signature) -> Result<(), CryptoError> {
    verifying_key
        .verify(message, signature)
        .map_err(|_| CryptoError::InvalidSignature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        // 1. تولید کلید
        let keypair = generate_keypair();
        let message = b"EvoX_Transaction_Data_Example";

        // 2. امضا کردن
        let signature = sign(&keypair.signing_key, message);

        // 3. تأیید صحیح
        let result = verify(&keypair.verifying_key, message, &signature);
        assert!(result.is_ok());

        // 4. تست امضای جعلی (تغییر در پیام)
        let tampered_message = b"EvoX_Transaction_Data_Tampered";
        let result_tampered = verify(&keypair.verifying_key, tampered_message, &signature);
        assert!(result_tampered.is_err());
    }

    #[test]
    fn test_invalid_key() {
        let keypair_a = generate_keypair();
        let keypair_b = generate_keypair();
        let message = b"Hello EvoX";

        let signature = sign(&keypair_a.signing_key, message);

        // سعی در تأیید امضای A با کلید عمومی B
        let result = verify(&keypair_b.verifying_key, message, &signature);
        assert!(result.is_err());
    }
}
