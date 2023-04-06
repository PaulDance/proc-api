mod proc;
use proc::Cache;
mod handlers;
mod routes;

#[tokio::main]
async fn main() {
    warp::serve(routes::all(Cache::default()))
        .run(([127, 0, 0, 1], 8080))
        .await;
}
