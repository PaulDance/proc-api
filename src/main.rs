mod proc;
use proc::Cache;
mod handlers;
mod routes;

#[tokio::main]
async fn main() {
    warp::serve(routes::all(Cache::default()))
        .run(([127, 0, 0, 1], 8080))
        .await;
}

#[cfg(test)]
mod tests {
    use std::str;

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
            .reply(&routes::list_processes(Cache::default()))
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.body(), "[]");
    }

    /// Refresh processes: empty OK response, non-empty cache.
    #[tokio::test]
    async fn test_refresh_procs() {
        let cache = Cache::default();
        let res = request()
            .method("POST")
            .path("/acquire_process_list")
            .reply(&routes::refresh_processes(cache.clone()))
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.body(), "");
        assert!(!cache.lock().await.is_empty());
    }

    /// Refresh processes, then fetch them: non-empty JSON array in OK response.
    #[tokio::test]
    async fn test_list_procs_refreshed() {
        let cache = Cache::default();
        request()
            .method("POST")
            .path("/acquire_process_list")
            .reply(&routes::refresh_processes(cache.clone()))
            .await;
        let res = request()
            .method("GET")
            .path("/processes")
            .reply(&routes::list_processes(cache.clone()))
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        assert!(
            !serde_json::from_str::<Vec<ProcInfo>>(str::from_utf8(&**res.body()).unwrap())
                .unwrap()
                .is_empty()
        );
    }
}
