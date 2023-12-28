use sqlx::{postgres::PgPoolOptions, Executor};
use warp::Filter;

mod jobs;

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
    let rc = match sqlx::query_as!(
        jobs::Job,
        "SELECT jobid,
        owner,
        creation_time,
        job_location,
        contract_type \"contract_type: jobs::ContractType\",
        mode \"mode: jobs::JobMode\",
        hours \"hours: jobs::JobHours\",
        description,
        tags 
        FROM jobs"
    )
    .fetch_all(&pool)
    .await
    {
        Ok(v) => v.len(),
        Err(_) => 0,
    };

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(move |name| format!("Hello, {}!\nThere is {} open jobs\n", name, rc));
    // let hello2 = warp::path("hello")
    //     .and(warp::path::end())
    //     .map(|| format!("Hello, World!"));

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}
