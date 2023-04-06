use std::convert::Infallible;

use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::handlers;
use crate::proc::Cache;

pub fn all(
    cache: Cache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list_processes(cache.clone())
        .or(refresh_processes(cache.clone()))
        .or(search_processes(cache.clone()))
}

pub fn list_processes(
    cache: Cache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("processes")
        .and(warp::get())
        .and(with_cache(cache))
        .and_then(handlers::list_processes)
}

pub fn refresh_processes(
    cache: Cache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("acquire_process_list")
        .and(warp::post())
        .and(with_cache(cache))
        .and_then(handlers::refresh_processes)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub pid: Option<u32>,
    pub username: Option<String>,
}

pub fn search_processes(
    cache: Cache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("search")
        .and(warp::get())
        .and(warp::query::<SearchQuery>())
        .and(with_cache(cache))
        .and_then(handlers::search_processes)
}

fn with_cache(cache: Cache) -> impl Filter<Extract = (Cache,), Error = Infallible> + Clone {
    warp::any().map(move || cache.clone())
}
