//! This modules defines the routes for the API: which requests are accepted,
//! with which methods, paths, paramaters, and which [`handlers`] they are sent
//! to.

use std::convert::Infallible;
use std::sync::Arc;

use serde::Deserialize;
use warp::Filter;

use crate::handlers;
use crate::proc::ProcCache;

/// Global route that dispatches to all the other effective routes defined in
/// the [module](`self`).
pub fn all(
    cache: &ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list_procs(Arc::clone(cache))
        .or(refresh_procs(Arc::clone(cache)))
        .or(search_procs(Arc::clone(cache)))
        .or(stream_procs(Arc::clone(cache)))
}

/// Route defining the read-only endpoint retrieving currently-cached processes
/// and returning a JSON array with the requested information.
///
/// See also: [`handlers::list_procs`].
pub fn list_procs(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("processes")
        .and(warp::get())
        .and(with_cache(cache))
        .and_then(handlers::list_procs)
}

/// Route defining the POST endpoint requesting a refreshing of the cache.
///
/// See also: [`handlers::refresh_procs`].
pub fn refresh_procs(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("acquire_process_list")
        .and(warp::post())
        .and(with_cache(cache))
        .and_then(handlers::refresh_procs)
}

/// Defines the acceptable parameters for the [`search_procs`] query.
///
/// Basically an all-optional version of [`crate::proc::ProcInfo`].
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub pid: Option<u32>,
    pub uid: Option<u32>,
    pub name: Option<String>,
    pub username: Option<String>,
}

/// Route defining the read-only endpoint equivalent of [`list_procs`], but
/// with filtering capabilities parsed from the request's URL parameters.
///
/// See also: [`handlers::search_procs`].
pub fn search_procs(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("search")
        .and(warp::get())
        .and(warp::query::<SearchQuery>())
        .and(with_cache(cache))
        .and_then(handlers::search_procs)
}

/// Route defining the SSE endpoint streaming currently-cached processes and
/// newly-discovered ones when a request is sent to the refresh endpoint.
///
/// See also: [`handlers::stream_procs`].
pub fn stream_procs(
    cache: ProcCache,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("data")
        .and(warp::get())
        .and(with_cache(cache))
        .and_then(handlers::stream_procs)
}

/// Convenience shortcut to add the current cache as an argument of each handler.
fn with_cache(cache: ProcCache) -> impl Filter<Extract = (ProcCache,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&cache))
}
