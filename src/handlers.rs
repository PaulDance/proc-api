use std::convert::Infallible;

use warp::http::StatusCode;

use crate::proc::ProcCache;
use crate::routes::SearchQuery;

pub async fn list_processes(cache: ProcCache) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&*cache.read().await.get()))
}

pub async fn refresh_processes(cache: ProcCache) -> Result<impl warp::Reply, Infallible> {
    Ok(cache
        .write()
        .await
        .refresh()
        .map_or(StatusCode::INTERNAL_SERVER_ERROR, |_| StatusCode::OK))
}

pub async fn search_processes(
    query: SearchQuery,
    cache: ProcCache,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    match query {
        SearchQuery {
            pid: None,
            username: None,
        } => Ok(Box::new(StatusCode::BAD_REQUEST)),
        _ => Ok(Box::new(warp::reply::json(
            &cache
                .read()
                .await
                .get()
                .iter()
                .filter(|&proc| {
                    (query.pid == None || query.pid == Some(proc.pid))
                        && query
                            .username
                            .as_ref()
                            .map(|username| username == proc.username.as_str())
                            .unwrap_or(true)
                })
                .collect::<Vec<_>>(),
        ))),
    }
}
