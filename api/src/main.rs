use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use jobs::get_all_jobs;
use sqlx::postgres::PgPoolOptions;
use warp::Filter;

mod jobs;
mod test;
pub mod users;
mod auth;

#[tokio::main]
async fn main() {
    let mut h = DefaultHasher::new();
    h.write(b"admin123");
    let hash = h.finish();
    println!("{}", (hash & (1 << 32) - 1 ^ hash >> 32) as i32);
    test::main().await;
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/nieletnipracuja")
        .await
    {
        Ok(p) => p,
        Err(_) => panic!(),
    };

    let rc = get_all_jobs(&pool).await.unwrap().len();
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(move |name| format!("Hello, {}!\nThere is {} open jobs\n", name, rc));
    // let hello2 = warp::path("hello")
    //     .and(warp::path::end())
    //     .map(|| format!("Hello, World!"));

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}
