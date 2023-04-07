use std::convert::Infallible;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::handlers;
use crate::proc::ProcCache;

pub fn all(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list_procs(Arc::clone(&cache))
        .or(refresh_procs(Arc::clone(&cache)))
        .or(search_procs(Arc::clone(&cache)))
        .or(stream_procs(Arc::clone(&cache)))
}

pub fn list_procs(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("processes")
        .and(warp::get())
        .and(with_cache(cache))
        .and_then(handlers::list_procs)
}

pub fn refresh_procs(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("acquire_process_list")
        .and(warp::post())
        .and(with_cache(cache))
        .and_then(handlers::refresh_procs)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub pid: Option<u32>,
    pub uid: Option<u32>,
    pub name: Option<String>,
    pub username: Option<String>,
}

pub fn search_procs(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("search")
        .and(warp::get())
        .and(warp::query::<SearchQuery>())
        .and(with_cache(cache))
        .and_then(handlers::search_procs)
}

pub fn stream_procs(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("data")
        .and(warp::get())
        .and(with_cache(cache))
        .and_then(handlers::stream_procs)
}

fn with_cache(cache: ProcCache) -> impl Filter<Extract = (ProcCache,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&cache))
}
