use std::convert::Infallible;

use warp::http::StatusCode;

use crate::proc::{Cache, ProcInfo};

pub async fn refresh_processes(cache: Cache) -> Result<impl warp::Reply, Infallible> {
    match ProcInfo::collect_all() {
        Ok(ps) => {
            *cache.lock().await = ps;
            Ok(StatusCode::OK)
        }
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_processes(cache: Cache) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&*cache.lock().await))
}
