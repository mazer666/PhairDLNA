//! The media HTTP server task and its shutdown handles.

use std::sync::Arc;

use tokio::sync::oneshot;

use super::super::{http, limit::CappedListener};

/// The HTTP server of the daemon, and the way to stop it.
///
/// The server reports its end through a channel of its own: the daemon has to
/// know when the server stops, both to drain the open transfers and to notice
/// a server that died on its own.
pub struct HttpServer {
    /// Asks the server to stop.
    pub(in crate::serve) stop: oneshot::Sender<()>,
    /// Carries the result of the server when it ends.
    pub(in crate::serve) ended: oneshot::Receiver<Result<(), std::io::Error>>,
    /// Tells the daemon that the server ended, whoever waits for it.
    ///
    /// The daemon watches this channel while it waits for a signal, and it
    /// keeps the handle of the server for the shutdown. The two must not
    /// fight over the same handle: a shutdown that cannot stop the server
    /// cuts every open transfer.
    pub(in crate::serve) finished: tokio::sync::watch::Receiver<bool>,
}

/// Starts the HTTP server in its own task.
pub fn start_http(listener: CappedListener, context: Arc<http::ServerContext>) -> HttpServer {
    let (stop, stopped) = oneshot::channel::<()>();
    let (report, ended) = oneshot::channel::<Result<(), std::io::Error>>();
    let (done, finished) = tokio::sync::watch::channel(false);
    let app = http::router(context);
    // The service knows the address of the peer: the access-control list is
    // checked against the address of the socket and not against a header
    // (`CFG-009`).
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<super::super::limit::PeerAddress>(),
    )
    .with_graceful_shutdown(async {
        let _ = stopped.await;
    });
    tokio::spawn(async move {
        let _ = report.send(server.await);
        let _ = done.send(true);
    });
    HttpServer {
        stop,
        ended,
        finished,
    }
}
