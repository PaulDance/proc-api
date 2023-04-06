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
            .reply(&routes::list_processes(ProcCache::default()))
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
            .reply(&routes::refresh_processes(Arc::clone(&cache)))
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.body(), "");
        assert!(!cache.read().await.is_empty());
    }

    /// Refresh processes, then fetch them: non-empty JSON array in OK response.
    #[tokio::test]
    async fn test_list_procs_refreshed() {
        let cache = ProcCache::default();
        request()
            .method("POST")
            .path("/acquire_process_list")
            .reply(&routes::refresh_processes(Arc::clone(&cache)))
            .await;
        let res = request()
            .method("GET")
            .path("/processes")
            .reply(&routes::list_processes(Arc::clone(&cache)))
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        assert!(
            !serde_json::from_str::<Vec<ProcInfo>>(str::from_utf8(&**res.body()).unwrap())
                .unwrap()
                .is_empty()
        );
    }
}
