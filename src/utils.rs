use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: usize,
    exp: usize,
    iss: String,
    alg: String,
}
/// To access github api as the application, we need to generate a jwt to use with github's api
pub fn generate_github_jwt() -> String {
    use std::time::SystemTime;

    let private_key = std::env::var("CLIENT_ID").unwrap();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        iat: now,
        exp: now + (60 * 10), // TODO expiry currently hardcoded to 10 min
        iss: std::env::var("CLIENT_ID").unwrap(),
        alg: "RS256".into(),
    };

    let header = Header {
        alg: Algorithm::RS256,
        ..Default::default()
    };
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(private_key.as_bytes()),
    )
    .expect("Failed encoding jwt token")
}
