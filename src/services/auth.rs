use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordVerifier, PasswordHash,
};

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<(), String> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "Invalid credentials".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "secure_password";
        let hash = hash_password(password).unwrap();
        assert_ne!(password, hash);
        
        assert!(verify_password(password, &hash).is_ok());
        assert!(verify_password("wrong_password", &hash).is_err());
    }
}
