use std::sync::{Arc, Mutex};

use anyhow::Result;
use warp::Filter;

mod proc;
use proc::ProcInfo;

#[tokio::main]
async fn main() {
    let procs = Arc::new(Mutex::new(Vec::<ProcInfo>::new()));

    warp::serve(
        warp::path("acquire_process_list")
            .and(warp::filters::method::post())
            .map(move || {
                *procs.lock().unwrap() = ProcInfo::collect_all().unwrap();
                ""
            }),
    )
    .run(([127, 0, 0, 1], 8080))
    .await;
}
