//! Main module: CLI parsing, server launching, and integration testing.

use std::env;
use std::net::IpAddr;

use clap::Parser;
use warp::Filter;
#[macro_use]
extern crate log;

mod proc;
use proc::ProcCache;
mod handlers;
mod routes;

/// `"proc_api"`
const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

/// Launch an HTTP server exposing the current host's process information.
#[derive(clap::Parser, Debug, PartialEq, Eq)]
#[command(author, version, about, long_about)]
pub struct CliArgs {
    /// The interface address to bind the server socket to.
    #[arg(short, long, default_value = "127.0.0.1")]
    pub addr: IpAddr,
    /// The port number to listen on.
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
}

/// Start the server on the given address and port.
#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    // Use INFO as a default.
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", format!("{CRATE_NAME}=info"));
    }

    pretty_env_logger::init();
    warp::serve(routes::all(&ProcCache::default()).with(warp::log(CRATE_NAME)))
        .run((args.addr, args.port))
        .await;
}

/// Basic integration tests.
#[cfg(test)]
mod tests {
    use std::str;
    use std::sync::Arc;

    use tokio::sync::Barrier;
    use tokio::time::{self, Duration};
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

    /// Start the stream, wait for the test timout: no data received.
    #[tokio::test]
    async fn test_stream_procs_empty() {
        let cache = ProcCache::default();

        let stream = {
            let cache = Arc::clone(&cache);

            tokio::spawn(async move {
                request()
                    .method("GET")
                    .path("/data")
                    .reply(&routes::stream_procs(cache))
                    .await
            })
        };

        assert!(cache.read().await.get().is_empty());

        let res = tokio::join!(stream).0.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(str::from_utf8(res.body()).unwrap().lines().take(2).count() == 0);
    }

    /// Start the stream, refresh processes, wait for the timeout: at least one
    /// process received.
    #[tokio::test]
    async fn test_stream_procs_refreshed_after() {
        let cache = ProcCache::default();
        let sync = Arc::new(Barrier::new(2));

        let stream = {
            let cache = Arc::clone(&cache);
            let sync = Arc::clone(&sync);

            tokio::spawn(async move {
                let filter = routes::stream_procs(cache);
                let fut = request().method("GET").path("/data").reply(&filter);
                sync.wait().await;
                fut.await
            })
        };

        sync.wait().await;
        assert_eq!(
            request()
                .method("POST")
                .path("/acquire_process_list")
                .reply(&routes::refresh_procs(Arc::clone(&cache)))
                .await
                .status(),
            StatusCode::OK
        );
        assert!(!cache.read().await.get().is_empty());

        let res = tokio::join!(stream).0.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(str::from_utf8(res.body()).unwrap().lines().take(2).count() == 2);
    }

    /// Refresh the processes first, then open the stream, wait for the timeout:
    /// at least one process is observed.
    #[tokio::test]
    async fn test_stream_procs_refreshed_first() {
        let cache = ProcCache::default();

        assert_eq!(
            request()
                .method("POST")
                .path("/acquire_process_list")
                .reply(&routes::refresh_procs(Arc::clone(&cache)))
                .await
                .status(),
            StatusCode::OK
        );

        let cache_len = cache.read().await.get().len();
        assert!(cache_len != 0);

        let stream = {
            let cache = Arc::clone(&cache);

            tokio::spawn(async move {
                request()
                    .method("GET")
                    .path("/data")
                    .reply(&routes::stream_procs(cache))
                    .await
            })
        };

        assert!(cache.read().await.get().len() == cache_len);

        let res = tokio::join!(stream).0.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(str::from_utf8(res.body()).unwrap().lines().take(2).count() == 2);
    }

    /// Refresh the processes first, then open the stream, refresh again, wait
    /// for the timeout: at least one process is observed after each action.
    #[tokio::test]
    async fn test_stream_procs_refreshed_first_and_after() {
        let cache = ProcCache::default();
        let sync = Arc::new(Barrier::new(2));

        assert_eq!(
            request()
                .method("POST")
                .path("/acquire_process_list")
                .reply(&routes::refresh_procs(Arc::clone(&cache)))
                .await
                .status(),
            StatusCode::OK
        );
        assert!(!cache.read().await.get().is_empty());

        let stream = {
            let cache = Arc::clone(&cache);
            let sync = Arc::clone(&sync);

            tokio::spawn(async move {
                let filter = routes::stream_procs(cache);
                let fut = request().method("GET").path("/data").reply(&filter);
                sync.wait().await;
                fut.await
            })
        };

        sync.wait().await;
        time::sleep(Duration::from_millis(500)).await;
        assert_eq!(
            request()
                .method("POST")
                .path("/acquire_process_list")
                .reply(&routes::refresh_procs(Arc::clone(&cache)))
                .await
                .status(),
            StatusCode::OK
        );
        assert!(!cache.read().await.get().is_empty());

        let res = tokio::join!(stream).0.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(str::from_utf8(res.body()).unwrap().lines().take(2).count() == 2);
    }

    /// Open the stream, refresh processes, wait for the timeout, refresh again:
    /// observe at least one process each time.
    #[tokio::test]
    async fn test_stream_procs_close_refreshed_after() {
        let cache = ProcCache::default();
        let sync = Arc::new(Barrier::new(2));

        let stream = {
            let cache = Arc::clone(&cache);
            let sync = Arc::clone(&sync);

            tokio::spawn(async move {
                let filter = routes::stream_procs(cache);
                let fut = request().method("GET").path("/data").reply(&filter);
                sync.wait().await;
                fut.await
            })
        };

        sync.wait().await;
        assert_eq!(
            request()
                .method("POST")
                .path("/acquire_process_list")
                .reply(&routes::refresh_procs(Arc::clone(&cache)))
                .await
                .status(),
            StatusCode::OK
        );
        assert!(!cache.read().await.get().is_empty());

        let res = tokio::join!(stream).0.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(str::from_utf8(res.body()).unwrap().lines().take(2).count() == 2);

        assert_eq!(
            request()
                .method("POST")
                .path("/acquire_process_list")
                .reply(&routes::refresh_procs(Arc::clone(&cache)))
                .await
                .status(),
            StatusCode::OK
        );
        assert!(!cache.read().await.get().is_empty());
    }

    /// Repeat multiple times: open a stream, refresh processes, close the
    /// stream, refresh, open another stream and close it right after. At least
    /// one process should be observed at each step and iteration.
    #[tokio::test]
    async fn test_stream_procs_close_reopen_refreshed() {
        let cache = ProcCache::default();
        let sync = Arc::new(Barrier::new(2));

        for _ in 0..3 {
            let stream = {
                let cache = Arc::clone(&cache);
                let sync = Arc::clone(&sync);

                tokio::spawn(async move {
                    let filter = routes::stream_procs(cache);
                    let fut = request().method("GET").path("/data").reply(&filter);
                    sync.wait().await;
                    fut.await
                })
            };

            sync.wait().await;
            assert_eq!(
                request()
                    .method("POST")
                    .path("/acquire_process_list")
                    .reply(&routes::refresh_procs(Arc::clone(&cache)))
                    .await
                    .status(),
                StatusCode::OK
            );
            assert!(!cache.read().await.get().is_empty());

            let res = tokio::join!(stream).0.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            assert!(str::from_utf8(res.body()).unwrap().lines().take(2).count() == 2);

            assert_eq!(
                request()
                    .method("POST")
                    .path("/acquire_process_list")
                    .reply(&routes::refresh_procs(Arc::clone(&cache)))
                    .await
                    .status(),
                StatusCode::OK
            );
            assert!(!cache.read().await.get().is_empty());

            let stream = {
                let cache = Arc::clone(&cache);

                tokio::spawn(async move {
                    request()
                        .method("GET")
                        .path("/data")
                        .reply(&routes::stream_procs(cache))
                        .await
                })
            };

            let res = tokio::join!(stream).0.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            assert!(str::from_utf8(res.body()).unwrap().lines().take(2).count() == 2);
        }
    }
}
