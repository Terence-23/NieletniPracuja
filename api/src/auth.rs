use json_web_token;
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::users::{UserError, UserRole};

const JWT_SECRET: &'static [u8; 18] = b"Very secret secret";

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

pub fn create_jwt(uid: &str, role: &UserRole) -> Result<String, impl std::error::Error> {
    let expiration = time::OffsetDateTime::now_utc()
        .checked_add(Duration::seconds(60))
        .expect("Invalid timestamp")
        .unix_timestamp();

    let claims = Claims {
        sub: uid.to_owned(),
        role: role.to_string(),
        exp: expiration as usize,
    };
    Ok::<_, UserError>("Not working".to_owned())
    // let header = Header::new(Algorithm::HS512);
    // encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
}
