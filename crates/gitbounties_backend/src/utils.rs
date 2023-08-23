use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: usize,
    exp: usize,
    iss: usize,
    alg: String,
}
/// To access github api as the application, we need to generate a jwt to use with github's api
pub fn generate_github_jwt() -> String {
    use std::time::SystemTime;

    let private_key = std::env::var("CLIENT_PRIVATE_KEY").unwrap();

    debug!("private key from env var {:?}", private_key);

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        iat: now,
        exp: now + (60 * 10), // TODO expiry currently hardcoded to 10 min
        iss: std::env::var("APP_ID").unwrap().parse::<usize>().unwrap(),
        alg: "RS256".into(),
    };

    encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(private_key.as_bytes()).unwrap(),
    )
    .expect("Failed encoding jwt token")
}
