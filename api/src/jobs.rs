use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "contract", rename_all = "lowercase")]
pub enum ContractType {
    Praca,
    Dzielo,
    Zlecenie,
    Tmp,
}
#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "job_hours", rename_all = "lowercase")]
pub enum JobHours {
    Weekend,
    Holiday,
    Week,
    Elastic,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "job_mode", rename_all = "lowercase")]
pub enum JobMode {
    Stationary,
    Home,
    Hybrid,
    Mobile,
}
#[derive(Debug, sqlx::FromRow)]
pub struct Job {
    pub(crate) jobid: i32,
    pub(crate) owner: sqlx::types::Uuid,
    pub(crate) creation_time: time::Time,
    pub(crate) job_location: Option<String>,
    pub(crate) contract_type: ContractType,
    pub(crate) mode: JobMode,
    pub(crate) hours: JobHours,
    pub(crate) description: Option<String>,
    pub(crate) tags: Option<JsonValue>,
}
