use serde::{Deserialize, Serialize};
use sqlx::{types::JsonValue, Pool, Postgres};

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "contract", rename_all = "lowercase")]
pub enum ContractType {
    Praca,
    Dzielo,
    Zlecenie,
    Tmp,
}
#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "job_hours", rename_all = "lowercase")]
pub enum JobHours {
    Weekend,
    Holiday,
    Week,
    Elastic,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "job_mode", rename_all = "lowercase")]
pub enum JobMode {
    Stationary,
    Home,
    Hybrid,
    Mobile,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct JobCreateRequest {
    pub(crate) job_location: String,
    pub(crate) contract_type: ContractType,
    pub(crate) mode: JobMode,
    pub(crate) hours: JobHours,
    pub(crate) description: String,
    pub(crate) tags: JsonValue,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct JobQuery {
    job_location: Option<String>,
    contract_type: Option<ContractType>,
    mode: Option<JobMode>,
    hours: Option<JobHours>,
    tags: Vec<String>,
    text: String,
}
impl JobQuery {
    pub async fn get_result(&self, pool: &Pool<Postgres>) -> Result<Vec<Job>, sqlx::Error> {
        sqlx::query_as!(
            Job,
            r#"SELECT 
                jobid,
                owner,
                creation_time,
                job_location,
                contract_type "contract_type: ContractType",
                mode "mode: JobMode",
                hours "hours: JobHours",
                description,
                tags 
            FROM jobs WHERE
                tags ?& $1 AND
                ($2::text Is NULL OR job_location = $2::text) AND
                ($3::contract Is NULL OR contract_type = $3::contract) AND
                ($4::job_mode Is NULL OR mode = $4::job_mode) AND
                ($5::job_hours Is NULL OR hours = $5::job_hours) AND
                description like $6
            "#,
            self.tags.as_slice(),
            &self.job_location as &Option<String>,
            &self.contract_type as &Option<_>,
            &self.mode as &Option<_>,
            &self.hours as &Option<_>,
            "%".to_owned() + &self.text + "%"
        )
        .fetch_all(pool)
        .await
    }
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
#[allow(unused)]
pub struct Job {
    pub(crate) jobid: i32,
    pub(crate) owner: sqlx::types::Uuid,
    pub(crate) creation_time: time::OffsetDateTime,
    pub(crate) job_location: Option<String>,
    pub(crate) contract_type: ContractType,
    pub(crate) mode: JobMode,
    pub(crate) hours: JobHours,
    pub(crate) description: Option<String>,
    pub(crate) tags: Option<JsonValue>,
}

pub async fn get_all_jobs(pool: &Pool<Postgres>) -> Result<Vec<Job>, sqlx::Error> {
    sqlx::query_as!(
        Job,
        "SELECT jobid,
        owner,
        creation_time,
        job_location,
        contract_type \"contract_type: ContractType\",
        mode \"mode: JobMode\",
        hours \"hours: JobHours\",
        description,
        tags 
        FROM jobs"
    )
    .fetch_all(pool)
    .await
}

pub async fn add_job(pool: &Pool<Postgres>, job: &Job) -> Result<(), sqlx::Error> {
    let added_jobs = sqlx::query_as!(
        Job,
        "INSERT INTO jobs (owner, creation_time, job_location, contract_type, mode, hours, description, tags)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING jobid, owner, creation_time, job_location, contract_type \"contract_type: ContractType\", mode \"mode: JobMode\", hours \"hours: JobHours\", description, tags",
        job.owner,
        job.creation_time,
        job.job_location,
        job.contract_type as _,
        job.mode as _,
        job.hours as _,
        job.description,
        job.tags
    ).fetch_all(pool).await?;
    assert_eq!(added_jobs.len(), 1);
    Ok(())
}
