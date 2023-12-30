use auth::Claim;
use jobs::{add_job, get_all_jobs, Job, JobCreateRequest, JobQuery};
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use time::{Duration, OffsetDateTime};
use users::UserRole;
use warp::{http::StatusCode, reject::Reject, Filter};

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
        Err(e) => return Err(warp::reject::custom(ServerError::PostgresError(e))),
    };
    Ok(warp::reply::json(&jobs))
}

async fn job_post(
    request: JobCreateRequest,
    owner_claim: Claim,
    pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let uuid = match uuid::Uuid::parse_str(&owner_claim.uuid) {
        Ok(u) => u,
        Err(_e) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&"Invalid uuid".to_owned()),
                StatusCode::BAD_REQUEST,
            ))
        }
    };
    if UserRole::Company != owner_claim.get_role() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"You are not allowed to create a job posting".to_owned()),
            StatusCode::FORBIDDEN,
        ));
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
        Err(_) => {
            warp::reply::with_status(warp::reply::json(&job), StatusCode::INTERNAL_SERVER_ERROR)
        }
    })
}

// println!(
//     "{}",
//     auth::create_jwt(uuid::Uuid::max(), &users::UserRole::User).unwrap()
// );
// let mut h = DefaultHasher::new();
// h.write(b"admin123");
// let hash = h.finish();
// println!("{}", (hash & (1 << 32) - 1 ^ hash >> 32) as i32);
// test::main().await;

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

    job_post(
        JobCreateRequest {
            job_location: "Somewhere".to_owned(),
            contract_type: jobs::ContractType::Praca,
            mode: jobs::JobMode::Mobile,
            hours: jobs::JobHours::Week,
            description: "Some random descryption".to_owned(),
            tags: vec!["Job".to_owned(), "Mobile".to_owned()].into(),
        },
        Claim {
            uuid: "12e1b078-4e42-47ed-a2c7-d6cd6269a2d0".to_owned(),
            role: UserRole::Company.to_string(),
            exp: OffsetDateTime::now_utc()
                .checked_add(Duration::seconds(10))
                .unwrap()
                .unix_timestamp(),
        },
        pool.clone(),
    )
    .await
    .unwrap();

    let pool_filter = warp::any().map(move || pool.clone());
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .and(pool_filter.clone())
        .and_then(hello);
    let jobs = warp::post()
        .and(warp::path("api"))
        .and(warp::path("get_jobs"))
        .and(warp::path::end())
        .and(job_query())
        .and(pool_filter.clone())
        .and_then(query_jobs);

    let routes = hello.or(jobs);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
