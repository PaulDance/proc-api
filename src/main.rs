//! TODO: write docs.

mod proc;
use proc::ProcCache;
mod handlers;
mod routes;

#[tokio::main]
async fn main() {
    warp::serve(routes::all(ProcCache::default()))
        .run(([127, 0, 0, 1], 8080))
        .await;
}

#[cfg(test)]
mod tests {
    use std::str;
    use std::sync::Arc;

    use warp::http::StatusCode;
    use warp::test::request;

    use super::*;
    use proc::ProcInfo;

    /// Fetch processes without refreshing them first: empty JSON array in OK
    /// response.
    #[tokio::test]
    async fn test_list_procs_empty() {
        let res = request()
            .method("GET")
            .path("/processes")
            .reply(&routes::list_procs(ProcCache::default()))
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.body(), "[]");
    }

    /// Refresh processes: empty OK response, non-empty cache.
    #[tokio::test]
    async fn test_refresh_procs() {
        let cache = ProcCache::default();
        let res = request()
            .method("POST")
            .path("/acquire_process_list")
            .reply(&routes::refresh_procs(Arc::clone(&cache)))
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.body(), "");
        assert!(!cache.read().await.get().is_empty());
    }

    /// Refresh processes, then fetch them: non-empty JSON array in OK response.
    #[tokio::test]
    async fn test_list_procs_refreshed() {
        let cache = ProcCache::default();
        request()
            .method("POST")
            .path("/acquire_process_list")
            .reply(&routes::refresh_procs(Arc::clone(&cache)))
            .await;
        let res = request()
            .method("GET")
            .path("/processes")
            .reply(&routes::list_procs(Arc::clone(&cache)))
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        assert!(
            !serde_json::from_str::<Vec<ProcInfo>>(str::from_utf8(&**res.body()).unwrap())
                .unwrap()
                .is_empty()
        );
    }

    // TODO: write tests for the streaming parts:
    //  * Open stream, refresh, observe at least two lines.
    //  * Refresh, open stream, observe some data.
    //  * Refresh, open stream, observe some data, refresh, observe again.
    //  * Open stream, refresh, observe, close stream, observe, refresh, observe.
    //  * Refresh, observe, open stream, refresh, observe, close stream, observe, refresh, observe.
}
