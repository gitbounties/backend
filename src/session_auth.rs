use std::sync::Arc;

use axum_login::{memory_store::MemoryStore as AuthMemoryStore, secrecy::SecretVec};
use log::debug;
use serde::{Deserialize, Serialize};

pub type AuthContext =
    axum_login::extractors::AuthContext<usize, AuthUser, AuthMemoryStore<usize, AuthUser>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    // pub username: String,
}

impl axum_login::AuthUser<String> for AuthUser {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new("password".clone().into())
    }
}

pub fn hash_password(password: &str) -> String {
    use scrypt::{
        password_hash::{
            rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        },
        Scrypt,
    };

    // hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Scrypt
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    debug!("salted password {password_hash}");

    password_hash
}
