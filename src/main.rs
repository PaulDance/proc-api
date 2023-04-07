//! Main module: server launching and integration testing.

mod proc;
use proc::ProcCache;
mod handlers;
mod routes;

/// Start the server on a default, non-configurable, local-only port.
#[tokio::main]
async fn main() {
    warp::serve(routes::all(ProcCache::default()))
        .run(([127, 0, 0, 1], 8080))
        .await;
}

/// Basic integration tests.
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

    /// Query the search endpoint without parameters: empty body in BAD REQUEST
    /// response.
    #[tokio::test]
    async fn test_search_procs_empty_noparams_is_badrequest() {
        let res = request()
            .method("GET")
            .path("/search")
            .reply(&routes::search_procs(ProcCache::default()))
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(res.body(), "");
    }

    /// Query the search endpoint without parameters even after refreshing
    /// the cache: empty body in BAD REQUEST response.
    #[tokio::test]
    async fn test_search_procs_refreshed_noparams_is_badrequest() {
        let cache = ProcCache::default();
        request()
            .method("POST")
            .path("/acquire_process_list")
            .reply(&routes::refresh_procs(Arc::clone(&cache)))
            .await;
        let res = request()
            .method("GET")
            .path("/search")
            .reply(&routes::search_procs(Arc::clone(&cache)))
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(res.body(), "");
    }

    /// Search for root processes without refreshing the cache first: empty
    /// JSON array in OK response.
    #[tokio::test]
    async fn test_search_procs_empty() {
        let res = request()
            .method("GET")
            .path("/search?uid=0")
            .reply(&routes::search_procs(ProcCache::default()))
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.body(), "[]");
    }

    /// Search for root processes by refreshing the cache first: non-empty JSON
    /// array in OK response.
    #[tokio::test]
    async fn test_search_procs_refreshed() {
        let cache = ProcCache::default();
        request()
            .method("POST")
            .path("/acquire_process_list")
            .reply(&routes::refresh_procs(Arc::clone(&cache)))
            .await;
        let res = request()
            .method("GET")
            .path("/search?uid=0")
            .reply(&routes::search_procs(Arc::clone(&cache)))
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
