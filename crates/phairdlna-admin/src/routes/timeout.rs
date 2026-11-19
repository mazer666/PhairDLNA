//! Request-handler timeout shared by every administration route.
//!
//! What: stop a request whose handler future exceeds the configured budget.
//! Why: keep the local API responsive without replacing socket read/write deadlines.
//! Example: a delayed handler returns `408 Request Timeout` after the budget.

use std::time::Duration;

use axum::http::StatusCode;
use axum::Router;
use tower_http::timeout::TimeoutLayer;

/// Applies the configured handler-future timeout to a router.
pub(super) fn apply_request_timeout<S>(router: Router<S>, timeout: Duration) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(TimeoutLayer::with_status_code(
        StatusCode::REQUEST_TIMEOUT,
        timeout,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use tower::ServiceExt;

    #[tokio::test]
    async fn a_slow_handler_returns_request_timeout() {
        let router = Router::new().route("/slow", get(slow_handler));
        let router = apply_request_timeout(router, Duration::from_millis(10));
        let request = Request::builder()
            .uri("/slow")
            .body(Body::empty())
            .expect("request is valid");

        let response = router.oneshot(request).await.expect("router answers");

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }

    async fn slow_handler() -> &'static str {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "late response"
    }
}
