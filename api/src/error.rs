use core::fmt;

use warp::reject::{Reject, Rejection};

#[derive(Debug)]
pub enum Error {
    NoSuchUser,
    BadPassword,
    ImproperNIP,
    SQLX(sqlx::Error),
    JWT(jsonwebtoken::errors::Error),
    UUID(uuid::Error),
    Forbidden,
    NoAuthHeaderError,
    InvalidAuthHeaderError,
}

pub type Result<T> = std::result::Result<T, Error>;
pub type WebResult<T> = std::result::Result<T, Rejection>;

impl Reject for Error {}
impl From<uuid::Error> for Error {
    fn from(value: uuid::Error) -> Self {
        Error::UUID(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::SQLX(value)
    }
}
impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Error::JWT(value)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::NoSuchUser => "There is no user with this login/email".to_owned(),
                Error::BadPassword => "The password is incorrect".to_owned(),
                Error::SQLX(e) => format!("Sqlx error: {}", e),
                Error::ImproperNIP => "The nip is incorrect".to_owned(),
                Error::JWT(e) => format!("JWT error: {}", e),
                Error::UUID(e) => format!("UUID error: {}", e),
                Error::Forbidden => "Forbidden".to_owned(),
                Error::NoAuthHeaderError => "No auth header".to_owned(),
                Error::InvalidAuthHeaderError => "Invalid auth header".to_owned(),
            }
        )
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::NoSuchUser => None,
            Error::BadPassword => None,
            Error::SQLX(e) => Some(e),
            Error::ImproperNIP => None,
            Error::JWT(e) => Some(e),
            Error::UUID(e) => Some(e),
            Error::Forbidden => None,
            Error::NoAuthHeaderError => None,
            Error::InvalidAuthHeaderError => None,
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}
