use auth::{async_decode, create_jwt, create_jwt_raw, decode_header, Claim};
use error::Error;
use jobs::{add_job, get_all_jobs, Job, JobCreateRequest, JobQuery};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use users::{CreateCompanyRequest, CreateUserRequest, LoginRequest, UserRole};
use warp::http::{HeaderMap, HeaderValue, StatusCode};
use warp::{filters::header::headers_cloned, reject::Reject, Filter};

mod error;

#[allow(unused)]
mod auth;
#[allow(unused)]
mod jobs;
#[allow(unused)]
mod test;
#[allow(unused)]
pub mod users;

#[derive(Serialize)]
struct HelloReply {
    name: String,
    jobs: Vec<Job>,
}

#[derive(Debug)]
enum ServerError {
    PostgresError(sqlx::Error),
}
impl Reject for ServerError {}

async fn hello(name: String, pool: Pool<Postgres>) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&HelloReply {
        name,
        jobs: match get_all_jobs(&pool).await {
            Ok(v) => v,
            Err(e) => return Err(warp::reject::custom(ServerError::PostgresError(e))),
        },
    }))
}

fn job_query() -> impl Filter<Extract = (JobQuery,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
async fn query_jobs(
    query: JobQuery,
    pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let jobs = match query.get_result(&pool).await {
        Ok(v) => v,
        Err(e) => return Err(warp::reject::custom(Error::SQLX(e))),
    };
    Ok(warp::reply::json(&jobs))
}

fn json_filter<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
where
    T: Send + DeserializeOwned,
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn claim_filter() -> impl Filter<Extract = (Claim,), Error = warp::Rejection> + Clone {
    headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| headers)
        .and_then(async_decode)
}

async fn job_post(
    request: JobCreateRequest,
    owner_claim: Claim,
    pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let uuid = match uuid::Uuid::parse_str(&owner_claim.uuid) {
        Ok(u) => u,
        Err(e) => return Err(warp::reject::custom(Error::from(e))),
    };
    if UserRole::Company != owner_claim.get_role() {
        return Err(warp::reject::custom(Error::Forbidden));
    }

    let job = Job {
        owner: uuid,
        jobid: -1,
        creation_time: time::OffsetDateTime::now_utc(),
        job_location: Some(request.job_location),
        contract_type: request.contract_type,
        mode: request.mode,
        hours: request.hours,
        description: Some(request.description),
        tags: Some(request.tags),
    };
    Ok(match add_job(&pool, &job).await {
        Ok(_) => warp::reply::with_status(warp::reply::json(&job), StatusCode::OK),
        Err(e) => return Err(warp::reject::custom(Error::from(e))),
    })
}

async fn login(
    req: LoginRequest,
    pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let claim = match req.login(&pool).await {
        Ok(c) => c,
        Err(e) => return Err(warp::reject::custom(e)),
    };
    let jwt = match create_jwt(claim) {
        Ok(jwt) => jwt,
        Err(e) => return Err(warp::reject::custom(e)),
    };
    return Ok(warp::reply::json(&jwt));
}

async fn register_user(
    req: CreateUserRequest,
    pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = match req.execute(&pool).await {
        Ok(u) => u,
        Err(e) => return Err(warp::reject::custom(Error::from(e))),
    };
    let jwt = match create_jwt_raw(res.userid, &UserRole::User) {
        Ok(jwt) => jwt,
        Err(e) => return Err(warp::reject::custom(Error::from(e))),
    };
    Ok(warp::reply::json(&jwt))
}

async fn register_company(
    req: CreateCompanyRequest,
    pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = match req.execute(&pool).await {
        Ok(u) => u,
        Err(e) => return Err(warp::reject::custom(Error::from(e))),
    };
    let jwt = match create_jwt_raw(res.userid, &UserRole::User) {
        Ok(jwt) => jwt,
        Err(e) => return Err(warp::reject::custom(Error::from(e))),
    };
    Ok(warp::reply::json(&jwt))
}

#[tokio::main]
async fn main() {
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/nieletnipracuja")
        .await
    {
        Ok(p) => p,
        Err(_) => panic!(),
    };

    // job_post(
    //     JobCreateRequest {
    //         job_location: "Somewhere".to_owned(),
    //         contract_type: jobs::ContractType::Praca,
    //         mode: jobs::JobMode::Mobile,
    //         hours: jobs::JobHours::Week,
    //         description: "Some random descryption".to_owned(),
    //         tags: vec!["Job".to_owned(), "Mobile".to_owned()].into(),
    //     },
    //     Claim {
    //         uuid: "12e1b078-4e42-47ed-a2c7-d6cd6269a2d0".to_owned(),
    //         role: UserRole::Company.to_string(),
    //         exp: OffsetDateTime::now_utc()
    //             .checked_add(Duration::seconds(10))
    //             .unwrap()
    //             .unix_timestamp(),
    //     },
    //     pool.clone(),
    // )
    // .await
    // .unwrap();

    let pool_filter = warp::any().map(move || pool.clone());
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .and(pool_filter.clone())
        .and_then(hello);
    let jobs = {
        warp::post()
            .and(warp::path("api"))
            .and(warp::path("get_jobs"))
            .and(warp::path::end())
            .and(job_query())
            .and(pool_filter.clone())
            .and_then(query_jobs)
    };
    let login = {
        warp::post()
            .and(warp::path("api"))
            .and(warp::path("login"))
            .and(warp::path::end())
            .and(json_filter::<LoginRequest>())
            .and(pool_filter.clone())
            .and_then(login)
    };
    let user_register = {
        warp::post()
            .and(warp::path("api"))
            .and(warp::path("register"))
            .and(warp::path("user"))
            .and(warp::path::end())
            .and(json_filter::<CreateUserRequest>())
            .and(pool_filter.clone())
            .and_then(register_user)
    };
    let company_register = {
        warp::post()
            .and(warp::path("api"))
            .and(warp::path("register"))
            .and(warp::path("company"))
            .and(warp::path::end())
            .and(json_filter::<CreateCompanyRequest>())
            .and(pool_filter.clone())
            .and_then(register_company)
    };
    let post_job = {
        warp::post()
            .and(warp::path("api"))
            .and(warp::path("post_job"))
            .and(warp::path::end())
            .and(json_filter::<JobCreateRequest>())
            .and(claim_filter())
            .and(pool_filter.clone())
            .and_then(job_post)
    };

    let routes = hello
        .or(jobs) // /api/get_jobs
        .or(login) // /api/login
        .or(user_register) // /api/register/user
        .or(company_register) // /api/register/company
        .or(post_job); // /api/post_job

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
