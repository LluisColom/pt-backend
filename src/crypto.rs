use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{PasswordHash, PasswordHasher, PasswordVerifier};

pub fn calculate_hash(input: impl AsRef<str>) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2::Argon2::default();
    argon2
        .hash_password(input.as_ref().as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

pub fn verify_hash(password: &str, stored_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(stored_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    // Calculate Argon2 password hash
    let argon2 = argon2::Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
