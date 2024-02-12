use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::Duration;
use uuid::Uuid;
use warp::http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use warp::reject::Reject;

use crate::error::{self, Error, WebResult};
use crate::users::UserRole;

use dotenv;
use lazy_static::lazy_static;

lazy_static! {
    static ref JWT_SECRET: Vec<u8> = dotenv::var("JWT_SECRET").unwrap().as_bytes().to_owned();
}
const ALG: Algorithm = Algorithm::HS512;
const BEARER: &str = "Bearer ";

fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> error::Result<String> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(Error::NoAuthHeaderError),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(Error::NoAuthHeaderError),
    };
    if !auth_header.starts_with(BEARER) {
        return Err(Error::InvalidAuthHeaderError);
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}
pub fn decode_header(headers: HeaderMap<HeaderValue>) -> error::Result<Claim> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(Error::NoAuthHeaderError),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(Error::NoAuthHeaderError),
    };
    if !auth_header.starts_with(BEARER) {
        return Err(Error::InvalidAuthHeaderError);
    }
    let jwt = auth_header.trim_start_matches(BEARER).to_owned();
    println!("JWT:{}", jwt);
    match decode::<Claim>(
        &jwt,
        &DecodingKey::from_secret(&JWT_SECRET),
        &Validation::new(ALG),
    ) {
        Ok(v) => {
            if time::OffsetDateTime::now_utc().unix_timestamp() > v.claims.exp {
                Err(Error::Expired)
            } else {
                Ok(v.claims)
            }
        }
        Err(e) => Err(Error::JWT(e)),
    }
}
pub async fn async_decode(headers: HeaderMap<HeaderValue>) -> WebResult<Claim> {
    decode_header(headers).map_err(|err| warp::reject::custom(err))
}

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
pub fn create_jwt(claim: Claim) -> Result<String, Error> {
    let header = Header::new(ALG);
    Ok(encode(
        &header,
        &claim,
        &EncodingKey::from_secret(&JWT_SECRET),
    )?)
}

pub fn create_jwt_raw(uid: uuid::Uuid, role: &UserRole) -> Result<String, Error> {
    let expiration = time::OffsetDateTime::now_utc()
        .checked_add(Duration::days(7))
        .expect("Invalid timestamp")
        .unix_timestamp();

    let claims = Claim {
        uuid: uid.as_simple().to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    let header = Header::new(ALG);
    Ok(encode(
        &header,
        &claims,
        &EncodingKey::from_secret(&JWT_SECRET),
    )?)
}

pub fn decode_jwt(jwt: String) -> Option<Claim> {
    match decode::<Claim>(
        &jwt,
        &DecodingKey::from_secret(&JWT_SECRET),
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
