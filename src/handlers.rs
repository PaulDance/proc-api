//! This module defines the async functions handling requests sent by the API's
//! [`crate::routes`] and returning the desired information.

use std::convert::Infallible;

use async_stream::stream;
use futures_util::stream::{self, Stream, StreamExt};
use tokio::time::{self, Duration};
use warp::{http::StatusCode, sse};

use crate::proc::ProcCache;
use crate::routes::SearchQuery;

/// Timeout used in [`proc_sse_events`] in order to cancel the stream task
/// during testing, but not when running normally.
const SSE_TOUT: Duration = if cfg!(test) {
    Duration::from_secs(1)
} else {
    Duration::MAX
};

/// Handles [`crate::routes::list_procs`] by returning the currently-cached
/// process data as a JSON reply.
pub async fn list_procs(cache: ProcCache) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(cache.read().await.get()))
}

/// Handles [`crate::routes::refresh_procs`] by refreshing the cache and returning a
/// status code reflecting the success or failure of the operation.
pub async fn refresh_procs(cache: ProcCache) -> Result<impl warp::Reply, Infallible> {
    Ok(cache
        .write()
        .await
        .refresh()
        .map_or(StatusCode::INTERNAL_SERVER_ERROR, |_| StatusCode::OK))
}

/// Handles [`crate::routes::search_procs`] by filtering the results and then
/// doing what [`list_procs`] does.
pub async fn search_procs(
    query: SearchQuery,
    cache: ProcCache,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    match query {
        // Reject the all-None case: it would render the list endpoint useless
        // if it were to be accepted because the logic used below would never
        // filter any process out of the resulting vector.
        SearchQuery {
            pid: None,
            uid: None,
            name: None,
            username: None,
        } => Ok(Box::new(StatusCode::BAD_REQUEST)),
        _ => Ok(Box::new(warp::reply::json(
            &cache
                .read()
                .await
                .get()
                .iter()
                .filter(|&proc| {
                    // Logic for each filter:
                    //  * None means not received in the request => true;
                    //  * Some(match) means filter matches => true;
                    //  * Some(_) means filters does not match => false;
                    // AND them all to reach usual search functionnality.
                    (query.pid.is_none() || query.pid == Some(proc.pid))
                        && (query.uid.is_none() || query.uid == Some(proc.uid))
                        && query
                            .name
                            .as_ref()
                            .map_or(true, |name| name == proc.name.as_str())
                        && query
                            .username
                            .as_ref()
                            .map_or(true, |username| username == proc.username.as_str())
                })
                // Collect to a vector because uniqueness guarantees and
                // operations of sets need not be used anymore are this point.
                .collect::<Vec<_>>(),
        ))),
    }
}

/// Handles [`crate::routes::stream_procs`] by setting up the streaming
/// capabilities of the API, building a stream from the data and returning a
/// [`warp::sse`] reply.
pub async fn stream_procs(cache: ProcCache) -> Result<impl warp::Reply, Infallible> {
    Ok(sse::reply(
        warp::sse::keep_alive().stream(proc_sse_events(cache).await),
    ))
}

/// Builds the actual stream for [`stream_procs`].
///
/// See also: [`crate::proc::CacheInner::refresh`] for the other end of the
/// channel.
async fn proc_sse_events(cache: ProcCache) -> impl Stream<Item = Result<sse::Event, Infallible>> {
    // Get a receiver, thus switching the cache to stream mode. As it is moved
    // into the stream builder, it will be automatically dropped right after
    // the stream is stopped by the client, thus avoiding channel lagging and
    // enabling switching back to the normal "blocking" mode of the cache when
    // the channel's receiver count finally drops to zero.
    let mut rx = cache.read().await.subscribe();
    // First immediately emit the currently-cached data,
    stream::iter(cache.read().await.get().clone().into_iter())
        // then stream new data received from the channel.
        .chain(
            // https://docs.rs/tokio/latest/tokio/stream/index.html
            stream! {
                debug!("SSE: stream started.");
                while let Ok(Ok(proc_group)) = time::timeout(SSE_TOUT, async {
                    debug!("SSE: waiting for channel data...");
                    rx.recv().await
                })
                .await
                {
                    debug!("SSE: received {} new processes.", proc_group.len());
                    yield stream::iter(proc_group.into_iter());
                }
                debug!("SSE: stream ended.");
            }
            .flatten(),
        )
        // Unwrapping here *should* ***hopefully*** be fine here because the
        // data is known to be correct JSON-capable data at this point.
        .map(|proc| Ok(sse::Event::default().json_data(proc).unwrap()))
}
