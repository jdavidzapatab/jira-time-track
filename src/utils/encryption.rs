use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::env;

pub fn encrypt(data: &str) -> String {
    let key_hex = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set");
    let key_bytes = hex::decode(key_hex).expect("Invalid ENCRYPTION_KEY hex");
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).expect("Invalid key length");

    let mut nonce_bytes = [0u8; 12];
    rand::fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .expect("encryption failure!");

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    STANDARD.encode(combined)
}

pub fn decrypt(encrypted_data: &str) -> String {
    let key_hex = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set");
    let key_bytes = hex::decode(key_hex).expect("Invalid ENCRYPTION_KEY hex");
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).expect("Invalid key length");

    let combined = STANDARD.decode(encrypted_data).expect("Invalid base64");
    if combined.len() < 12 {
        panic!("Invalid encrypted data");
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .expect("decryption failure!");

    String::from_utf8(plaintext).expect("Invalid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_encrypt_decrypt() {
        // Set a dummy key for testing
        let test_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        unsafe {
            env::set_var("ENCRYPTION_KEY", test_key);
        }

        let original_text = "secret password 123";
        let encrypted = encrypt(original_text);
        assert_ne!(original_text, encrypted);

        let decrypted = decrypt(&encrypted);
        assert_eq!(original_text, decrypted);
    }
}
