use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::Duration;
use uuid::Uuid;

use crate::users::{UserError, UserRole};

const JWT_SECRET: &'static [u8] = b"Very secret secret";
const ALG: Algorithm = Algorithm::HS512;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claim {
    pub uuid: String,
    pub role: String,
    pub exp: i64,
}
impl Claim {
    pub fn get_uuid(&self) -> Result<Uuid, uuid::Error> {
        uuid::Uuid::parse_str(&self.uuid)
    }
    pub fn get_role(&self) -> UserRole {
        match &self.role.to_lowercase()[..] {
            "company" => UserRole::Company,
            _ => UserRole::User,
        }
    }
}

pub fn create_jwt(uid: uuid::Uuid, role: &UserRole) -> Result<String, impl std::error::Error> {
    let expiration = time::OffsetDateTime::now_utc()
        .checked_add(Duration::seconds(60))
        .expect("Invalid timestamp")
        .unix_timestamp();

    let claims = Claim {
        uuid: uid.as_simple().to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    let header = Header::new(ALG);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
}

pub fn decode_jwt(jwt: String) -> Option<Claim> {
    match decode::<Claim>(
        &jwt,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::new(ALG),
    ) {
        Ok(v) => {
            if time::OffsetDateTime::now_utc().unix_timestamp() > v.claims.exp {
                None
            } else {
                Some(v.claims)
            }
        }
        Err(_) => None,
    }
}
