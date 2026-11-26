//! Network socket opening and capped HTTP server startup.

use std::net::Ipv4Addr;
use std::sync::Arc;

use phairdlna_core::security::Limits;
use phairdlna_core::upnp::BaseUrl;
use tokio::net::TcpListener;

use super::super::http;
use super::super::limit::CappedListener;
use super::http_server::start_http;

/// Opens the socket of the media server and builds its base URL.
///
/// The base URL carries the port the operating system gave, not the one that
/// was asked for: the port `0` means "any free port", and a client that reads
/// the description must learn the port that is really open (`INT-022`,
/// `INT-006`).
pub async fn open_the_socket(
    host: Ipv4Addr,
    port: u16,
) -> Result<(TcpListener, std::net::SocketAddr, BaseUrl), String> {
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .await
        .map_err(|error| format!("the port {port} cannot be opened: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("the address of the server is not known: {error}"))?;
    let base_url = BaseUrl::new(&format!("http://{host}:{}", address.port()))
        .map_err(|error| format!("the base URL is not usable: {error}"))?;
    Ok((listener, address, base_url))
}

/// Starts the HTTP server with bounded connections and timeouts.
pub fn start_capped_http(
    listener: TcpListener,
    limits: &Limits,
    context: Arc<http::ServerContext>,
) -> super::http_server::HttpServer {
    let capped = CappedListener::new(
        listener,
        limits.connections,
        limits.read_timeout,
        limits.write_timeout,
    );
    start_http(capped, context)
}
