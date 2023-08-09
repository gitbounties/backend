use std::sync::Arc;

use axum_session::SessionNullPool;
use axum_session_auth::{
    AuthSession, AuthSessionLayer, Authentication, HasPermission, SessionSqlitePool,
};
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub type MyAuthSessionLayer = AuthSessionLayer<AuthUser, String, SessionSqlitePool, SqlitePool>;
pub type MyAuthSession = AuthSession<AuthUser, String, SessionSqlitePool, SqlitePool>;
pub type NullPool = Arc<Option<()>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    // pub username: String,
}

#[axum::async_trait]
impl Authentication<AuthUser, String, SqlitePool> for AuthUser {
    // This is ran when the user has logged in and has not yet been Cached in the system.
    // Once ran it will load and cache the user.
    async fn load_user(userid: String, _pool: Option<&SqlitePool>) -> anyhow::Result<AuthUser> {
        Ok(AuthUser { id: userid })
    }

    // This function is used internally to deturmine if they are logged in or not.
    fn is_authenticated(&self) -> bool {
        true
    }

    fn is_active(&self) -> bool {
        true
    }

    fn is_anonymous(&self) -> bool {
        false
    }
}

#[axum::async_trait]
impl HasPermission<SqlitePool> for AuthUser {
    async fn has(&self, perm: &str, _pool: &Option<&SqlitePool>) -> bool {
        true
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
