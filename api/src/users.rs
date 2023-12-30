use serde::{ser::SerializeStruct, Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::{
    collections::hash_map::DefaultHasher,
    fmt::{self, Formatter},
    hash::Hasher,
};

#[derive(Debug)]
pub enum UserError {
    NoSuchUser,
    BadPassword,
    WrongNIP,
    SQLX(sqlx::Error),
}

impl From<sqlx::Error> for UserError {
    fn from(value: sqlx::Error) -> Self {
        UserError::SQLX(value)
    }
}
impl fmt::Display for UserError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UserError::NoSuchUser => "There is no user with this login/email".to_owned(),
                UserError::BadPassword => "The password is incorrect".to_owned(),
                UserError::SQLX(v) => format!("Sqlx error: {}", v),
                UserError::WrongNIP => "The nip is incorrect".to_owned(),
            }
        )
    }
}
impl std::error::Error for UserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UserError::NoSuchUser => None,
            UserError::BadPassword => None,
            UserError::SQLX(v) => Some(v),
            UserError::WrongNIP => None,
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

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
    pub async fn execute(&self, pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO users (email, full_name, login, password)
            VALUES ($1, $2, $3, $4)
            "#,
            self.email,
            self.full_name,
            self.login,
            self.get_password_hash()
        )
        .execute(pool)
        .await?;

        Ok(())
    }
    pub fn get_password_hash(&self) -> i32 {
        get_string_hash(&self.password)
    }
}
pub struct CreateCompanyRequest {
    email: String,
    full_name: String,
    login: String,
    password: String,
    nip: i64,
    company_name: String,
}
impl CreateCompanyRequest {
    pub async fn execute(&self, pool: &Pool<Postgres>) -> Result<(), UserError> {
        if !self.validate_nip() {
            return Err(UserError::WrongNIP);
        }
        sqlx::query!(
            r#"INSERT INTO companies (email, full_name, login, password, nip, company_name)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            self.email,
            self.full_name,
            self.login,
            self.get_password_hash(),
            self.nip,
            self.company_name
        )
        .execute(pool)
        .await?;

        Ok(())
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

#[derive(Deserialize, Debug, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(unused)]
pub struct User {
    userid: sqlx::types::Uuid,
    login: String,
    password: i32,
    email: String,
    full_name: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(unused)]
pub struct Company {
    userid: sqlx::types::Uuid,
    login: String,
    email: String,
    password: i32,
    nip: i64,
    company_name: String,
    full_name: String,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("User", 5)?;
        state.serialize_field("user_id", &self.userid.to_string())?;
        state.serialize_field("login", &self.login)?;
        state.serialize_field("password", &self.password)?;
        state.serialize_field("email", &self.email)?;
        state.serialize_field("full_name", &self.full_name)?;

        state.end()
    }
}

impl User {
    pub async fn login(
        login: String,
        password_hash: i32,
        pool: &Pool<Postgres>,
    ) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT * FROM users 
            WHERE login = '$1' OR email = '$1' "#,
        )
        .bind(login)
        .fetch_all(pool)
        .await?;
        if user.len() < 1 {
            return Err(UserError::NoSuchUser);
        }
        if password_hash != user[0].password {
            return Err(UserError::BadPassword);
        }
        Ok(user[0].to_owned())
    }
}

impl Company {
    pub async fn login(
        login: String,
        password_hash: i32,
        pool: &Pool<Postgres>,
    ) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT * FROM companies 
            WHERE login = '$1' OR email = '$1' "#,
        )
        .bind(login)
        .fetch_all(pool)
        .await?;
        if user.len() < 1 {
            return Err(UserError::NoSuchUser);
        }

        if password_hash != user[0].password {
            return Err(UserError::BadPassword);
        }
        Ok(user[0].to_owned())
    }
}
