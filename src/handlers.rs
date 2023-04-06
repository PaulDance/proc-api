use std::convert::Infallible;

use warp::http::StatusCode;

use crate::proc::{Cache, ProcInfo};
use crate::routes::SearchQuery;

pub async fn list_processes(cache: Cache) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&*cache.read().await))
}

pub async fn refresh_processes(cache: Cache) -> Result<impl warp::Reply, Infallible> {
    match ProcInfo::collect_all() {
        Ok(ps) => {
            *cache.write().await = ps;
            Ok(StatusCode::OK)
        }
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn search_processes(
    query: SearchQuery,
    cache: Cache,
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
