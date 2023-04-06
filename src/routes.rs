use std::convert::Infallible;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::handlers;
use crate::proc::ProcCache;

pub fn all(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list_processes(Arc::clone(&cache))
        .or(refresh_processes(Arc::clone(&cache)))
        .or(search_processes(Arc::clone(&cache)))
        .or(stream_processes(Arc::clone(&cache)))
}

// TODO: shorten these names with "procs"?
pub fn list_processes(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("processes")
        .and(warp::get())
        .and(with_cache(cache))
        .and_then(handlers::list_processes)
}

pub fn refresh_processes(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("acquire_process_list")
        .and(warp::post())
        .and(with_cache(cache))
        .and_then(handlers::refresh_processes)
}

// TODO: add remaining fields as a bonus.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub pid: Option<u32>,
    pub username: Option<String>,
}

pub fn search_processes(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("search")
        .and(warp::get())
        .and(warp::query::<SearchQuery>())
        .and(with_cache(cache))
        .and_then(handlers::search_processes)
}

pub fn stream_processes(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("data")
        .and(warp::get())
        .and(with_cache(cache))
        .and_then(handlers::stream_processes)
}

fn with_cache(cache: ProcCache) -> impl Filter<Extract = (ProcCache,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&cache))
}
