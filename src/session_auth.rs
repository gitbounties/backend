use std::sync::Arc;

use axum_session::SessionNullPool;
use axum_session_auth::{AuthSession, AuthSessionLayer, Authentication};
use log::debug;
use serde::{Deserialize, Serialize};

pub type MyAuthSessionLayer = AuthSessionLayer<AuthUser, i64, SessionNullPool, NullPool>;
pub type MyAuthSession = AuthSession<AuthUser, i64, SessionNullPool, NullPool>;
pub type NullPool = Arc<Option<()>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: i64,
    pub anonymous: bool,
    // pub username: String,
}

#[axum::async_trait]
impl Authentication<AuthUser, i64, NullPool> for AuthUser {
    // This is ran when the user has logged in and has not yet been Cached in the system.
    // Once ran it will load and cache the user.
    async fn load_user(userid: i64, _pool: Option<&NullPool>) -> anyhow::Result<AuthUser> {
        Ok(AuthUser {
            id: userid,
            anonymous: true,
        })
    }

    // This function is used internally to deturmine if they are logged in or not.
    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
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
