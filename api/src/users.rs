use serde::{ser::SerializeStruct, Deserialize, Serialize};
use sqlx::{prelude::FromRow, Pool, Postgres};
use std::{
    collections::hash_map::DefaultHasher,
    f32::consts::E,
    fmt::{self, Formatter},
    hash::Hasher,
};
use time::Duration;
use uuid::timestamp::context::NoContext;
use uuid::{Timestamp, Uuid};
use warp::reject::Reject;

use crate::auth::Claim;
use crate::error::Error;

fn get_string_hash(s: &str) -> i32 {
    let mut hasher = DefaultHasher::new();
    hasher.write(s.as_bytes());
    let hash = hasher.finish();
    (hash & (1 << 32) - 1 ^ hash >> 32) as i32
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    email: String,
    full_name: String,
    login: String,
    password: String,
}
impl CreateUserRequest {
    pub async fn execute(&self, pool: &Pool<Postgres>) -> Result<User, sqlx::Error> {
        let uuid = uuid::Uuid::new_v7(Timestamp::now(NoContext::default()));

        sqlx::query!(
            r#" INSERT INTO login (login, email, password, userid, role)
            VALUES ($1, $2, $3, $4, $5)"#,
            self.login,
            self.email,
            self.get_password_hash(),
            uuid,
            UserRole::User as _
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            r#"INSERT INTO users (email, full_name, login, password, userid)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            self.email,
            self.full_name,
            self.login,
            self.get_password_hash(),
            uuid
        )
        .execute(pool)
        .await?;

        Ok(User {
            userid: uuid,
            login: self.login.to_owned(),
            password: self.get_password_hash(),
            email: self.email.to_owned(),
            full_name: self.full_name.to_owned(),
        })
    }
    pub fn get_password_hash(&self) -> i32 {
        get_string_hash(&self.password)
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCompanyRequest {
    email: String,
    full_name: String,
    login: String,
    password: String,
    nip: i64,
    company_name: String,
}
impl CreateCompanyRequest {
    pub async fn execute(&self, pool: &Pool<Postgres>) -> Result<Company, Error> {
        if !self.validate_nip() {
            return Err(Error::ImproperNIP);
        }
        let uuid = uuid::Uuid::new_v7(Timestamp::now(NoContext::default()));

        sqlx::query!(
            r#" INSERT INTO login (login, email, password, userid, role)
            VALUES ($1, $2, $3, $4, $5)"#,
            self.login,
            self.email,
            self.get_password_hash(),
            uuid,
            UserRole::Company as _
        )
        .execute(pool)
        .await?;
        sqlx::query!(
            r#"INSERT INTO companies (email, full_name, login, password, nip, company_name, userid)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            self.email,
            self.full_name,
            self.login,
            self.get_password_hash(),
            self.nip,
            self.company_name,
            uuid
        )
        .execute(pool)
        .await?;

        Ok(Company {
            userid: uuid,
            login: self.login.to_owned(),
            email: self.email.to_owned(),
            password: self.get_password_hash(),
            nip: self.nip,
            company_name: self.company_name.to_owned(),
            full_name: self.full_name.to_owned(),
        })
    }
    pub fn get_password_hash(&self) -> i32 {
        get_string_hash(&self.password)
    }
    fn validate_nip(&self) -> bool {
        let nip_str = self.nip.to_string();
        let nip = nip_str.as_bytes();
        if nip.len() < 10 || nip.len() > 10 {
            return false;
        }
        let mul = {
            (nip[0] - b'0') as u16 * 6
                + (nip[1] - b'0') as u16 * 5
                + (nip[2] - b'0') as u16 * 7
                + (nip[3] - b'0') as u16 * 2
                + (nip[4] - b'0') as u16 * 3
                + (nip[5] - b'0') as u16 * 4
                + (nip[6] - b'0') as u16 * 5
                + (nip[7] - b'0') as u16 * 6
                + (nip[8] - b'0') as u16 * 7
        };

        let m = mul % 11;
        if m == 10 || m != *nip.last().unwrap() as u16 {
            return false;
        }
        true
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "role", rename_all = "lowercase")]
pub enum UserRole {
    Company,
    User,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Company => write!(f, "Company"),
            Self::User => write!(f, "User"),
        }
    }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
struct LoginData {
    login: String,
    email: String,
    password: i32,
    userid: Uuid,
    role: UserRole,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct LoginRequest {
    login: String,
    password: String,
}
impl LoginRequest {
    pub fn get_password_hash(&self) -> i32 {
        get_string_hash(&self.password)
    }
    pub fn get_login(&self) -> String {
        self.login.to_owned()
    }
    pub async fn login(&self, pool: &Pool<Postgres>) -> Result<Claim, Error> {
        let data = sqlx::query_as!(
            LoginData,
            r#"SELECT
            login,
            email, 
            password,
            userid,
            role "role: UserRole"
            FROM login WHERE
            email = $1 OR
            login = $1
            "#,
            self.login
        )
        .fetch_one(pool)
        .await?;
        if self.get_password_hash() != data.password {
            return Err(Error::BadPassword);
        }
        let uuid = data.userid;
        let role = data.role;

        Ok(Claim {
            uuid: uuid.as_simple().to_string(),
            role: role.to_string(),
            exp: (time::OffsetDateTime::now_utc() + Duration::days(7)).unix_timestamp(),
        })
    }
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
#[allow(unused)]
pub struct User {
    pub userid: sqlx::types::Uuid,
    login: String,
    password: i32,
    email: String,
    full_name: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
#[allow(unused)]
pub struct Company {
    pub userid: sqlx::types::Uuid,
    login: String,
    email: String,
    password: i32,
    nip: i64,
    company_name: String,
    full_name: String,
}
