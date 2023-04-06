use std::sync::{Arc, Mutex};

use warp::Filter;

mod proc;
use proc::ProcInfo;

#[tokio::main]
async fn main() {
    let procs = Arc::new(Mutex::new(Vec::<ProcInfo>::new()));
    let p1 = Arc::clone(&procs);
    let p2 = Arc::clone(&procs);

    // TODO: try to bubble errors up instead of unwrapping.
    warp::serve(
        warp::path("acquire_process_list")
            .and(warp::post())
            .map(move || {
                *p1.lock().unwrap() = ProcInfo::collect_all().unwrap();
                ""
            })
            .or(warp::path("processes")
                .and(warp::get())
                .map(move || warp::reply::json(&*p2.lock().unwrap()))),
    )
    .run(([127, 0, 0, 1], 8080))
    .await;
}
