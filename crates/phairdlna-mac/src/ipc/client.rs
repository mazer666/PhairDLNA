//! The client of the administration surface.
//!
//! What: the request that is built, the answer that is read and the snapshot
//!       that the menu bar shows.
//! Why: the menu bar application holds no state of the server of its own. It
//!      asks the surface what the server does and shows the answer, so that
//!      the application and the server can never disagree (`MB-001`,
//!      `MB-005`).
//! Example: a client that knows the port of the surface and the token of the
//!          installation asks `GET /api/status` and reads the number of the
//!          streams that are running.
//!
//! **The surface is on the loopback interface.** Every request of this module
//! names `127.0.0.1` and carries the token of the installation. The module
//! never opens a connection to another host, and it never reads the token out
//! of a place other than the file the owner of the machine may read
//! (`SEC-071`, `SEC-094`).

use std::path::PathBuf;
use std::time::Duration;

mod errors;
mod token;

pub use errors::ClientError;
pub use token::read_token;
use token::TokenSource;

use crate::ipc::http::{HttpRequest, HttpResponse, Transport};
use crate::ipc::json::Json;
use crate::ipc::snapshots::{
    number_of, roots_of, texts_of, transfers_of, DiagnosticsSnapshot, RescanReport, StatusSnapshot,
};

/// The name of the header that names the host of the request.
const HOST: &str = "Host";
/// The name of the header that carries the token.
const AUTHORIZATION: &str = "Authorization";
/// The word that introduces the token.
const BEARER: &str = "Bearer";
/// The name of the header that a form in a browser cannot send.
const ADMIN_HEADER: &str = "X-Phair-Admin";
/// The value the header must carry.
const ADMIN_HEADER_VALUE: &str = "1";
/// The name of the header that a request that changes something carries.
const CSRF_HEADER: &str = "X-Phair-Admin-Csrf";
/// The name of the header of an answer that carries the value of the token.
///
/// The surface names the header in lower case, and the name of a header does
/// not matter in its case.
const CSRF_ANSWER_HEADER: &str = "x-phairdlna-admin-csrf";
/// The name of the header that names the length of a body.
const CONTENT_LENGTH: &str = "Content-Length";
/// The name of the header that names the wanted answer.
const ACCEPT: &str = "Accept";
/// The answer the client asks for.
const APPLICATION_JSON: &str = "application/json";
/// The name of the header that says that the connection ends with the answer.
const CONNECTION: &str = "Connection";
/// The value of that header.
const CLOSE: &str = "close";
/// The method of a request that only reads.
const METHOD_GET: &str = "GET";
/// The method of a request that changes something.
const METHOD_POST: &str = "POST";
/// The path of the state of the server.
const PATH_STATUS: &str = "/api/status";
/// The path of the diagnostics of the server.
const PATH_DIAGNOSTICS: &str = "/api/diagnostics";
/// The path of the scan of the library.
const PATH_RESCAN: &str = "/api/catalogue/rescan";

/// How long the scan of the library may take, in seconds.
///
/// The scan reads every file of the library; the surface answers when it is
/// done, and the answer may take a while (`UX-013`).
const RESCAN_TIMEOUT: Duration = Duration::from_secs(120);

/// The client of the administration surface.
#[derive(Clone, Debug)]
pub struct AdminClient<T> {
    /// The way the bytes travel.
    transport: T,
    /// Where the token of the installation comes from.
    token: TokenSource,
    /// The token of the "cross-site request forgery" defence.
    ///
    /// The surface hands the value out with every answer, and the client sends
    /// it back with the request that changes something (`OPS-023`).
    csrf: Option<String>,
}

impl<T: Transport> AdminClient<T> {
    /// Builds the client of a surface.
    pub fn new(transport: T, token: impl Into<String>) -> Self {
        Self {
            transport,
            token: TokenSource::Fixed(token.into()),
            csrf: None,
        }
    }

    /// Builds the client of a surface whose token lives in a file.
    ///
    /// The file is read before every request. A file that is not there means
    /// that the server was never started: the client says so and sends
    /// nothing (`MB-001`).
    pub fn with_token_file(transport: T, path: PathBuf) -> Self {
        Self {
            transport,
            token: TokenSource::File(path),
            csrf: None,
        }
    }

    /// The loopback base address of the administration API.
    pub fn address(&self) -> String {
        format!("http://{}", self.transport.address())
    }

    /// The way the bytes travel.
    ///
    /// The answer is the transport itself, so that the tests of the client
    /// can look at the requests it sent.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// The state of the server.
    pub fn status(&mut self) -> Result<StatusSnapshot, ClientError> {
        let document = self.read(METHOD_GET, PATH_STATUS)?;
        Ok(StatusSnapshot {
            lifecycle_state: document
                .get("lifecycle_state")
                .and_then(Json::as_str)
                .map(str::to_owned),
            lifecycle_reason: document
                .get("lifecycle_reason")
                .and_then(Json::as_str)
                .map(str::to_owned),
            device_name: document
                .get("device_name")
                .and_then(Json::as_str)
                .unwrap_or_default()
                .to_owned(),
            device_address: document
                .get("device_address")
                .and_then(Json::as_str)
                .map(str::to_owned),
            discovery_available: document.get("discovery_available").and_then(Json::as_bool),
            uptime_seconds: number_of(&document, "uptime_seconds"),
            active_streams: number_of(&document, "active_streams"),
            catalogue_objects: number_of(&document, "catalogue_objects"),
            roots: roots_of(&document),
        })
    }

    /// What the server has to say about itself.
    pub fn diagnostics(&mut self) -> Result<DiagnosticsSnapshot, ClientError> {
        let document = self.read(METHOD_GET, PATH_DIAGNOSTICS)?;
        Ok(DiagnosticsSnapshot {
            warnings: texts_of(&document, "warnings"),
            transfers: transfers_of(&document),
            last_scan: document
                .get("last_scan")
                .and_then(Json::as_str)
                .map(str::to_owned),
        })
    }

    /// Fetches the raw JSON string of the diagnostics report.
    pub fn raw_diagnostics(&mut self) -> Result<String, ClientError> {
        let response =
            self.send_with_timeout(METHOD_GET, PATH_DIAGNOSTICS, self.transport.timeout())?;
        match response.status {
            200..=299 => Ok(response.body_text()),
            401 => Err(ClientError::Unauthorised),
            status => Err(ClientError::Refused { status }),
        }
    }

    /// Reads the library again.
    ///
    /// The request changes something and therefore carries the token of the
    /// "cross-site request forgery" defence. The client reads the value out of
    /// an answer of the surface when it has none (`OPS-023`).
    pub fn rescan(&mut self) -> Result<RescanReport, ClientError> {
        if self.csrf.is_none() {
            let _ = self.status()?;
        }
        let document = self.read_with_timeout(METHOD_POST, PATH_RESCAN, RESCAN_TIMEOUT)?;
        Ok(RescanReport {
            rescanned: document
                .get("rescanned")
                .and_then(Json::as_bool)
                .unwrap_or(false),
            objects: number_of(&document, "objects"),
            duration_seconds: number_of(&document, "duration_seconds"),
            message: document
                .get("message")
                .and_then(Json::as_str)
                .unwrap_or_default()
                .to_owned(),
        })
    }

    /// Asks the surface and reads its answer.
    fn read(&mut self, method: &str, path: &str) -> Result<Json, ClientError> {
        self.read_with_timeout(method, path, self.transport.timeout())
    }

    /// Asks the surface and reads its answer, with a time of its own.
    ///
    /// The scan of the library reads every file of the library and may take
    /// longer than the answer of a request that only reads (`UX-013`).
    fn read_with_timeout(
        &mut self,
        method: &str,
        path: &str,
        timeout: Duration,
    ) -> Result<Json, ClientError> {
        let response = self.send_with_timeout(method, path, timeout)?;
        match response.status {
            200..=299 => Ok(Json::parse(&response.body_text())?),
            401 => Err(ClientError::Unauthorised),
            status => Err(ClientError::Refused { status }),
        }
    }

    /// Builds the request, sends it and keeps the token of the defence.
    fn send_with_timeout(
        &mut self,
        method: &str,
        path: &str,
        timeout: Duration,
    ) -> Result<HttpResponse, ClientError> {
        let mut request = HttpRequest::new(method, path)
            .with_timeout(timeout)
            .with_header(HOST, &self.transport.host())
            .with_header(ADMIN_HEADER, ADMIN_HEADER_VALUE)
            .with_header(ACCEPT, APPLICATION_JSON)
            .with_header(CONNECTION, CLOSE);
        if let Some(authorization) = self.authorization()? {
            request = request.with_header(AUTHORIZATION, &authorization);
        }
        if let Some(csrf) = &self.csrf {
            request = request.with_header(CSRF_HEADER, csrf);
        }
        if method == METHOD_POST {
            // The request that changes something has no body; the length says
            // so, so that the surface does not wait for one.
            request = request.with_header(CONTENT_LENGTH, "0");
        }
        let response = self.transport.send(&request)?;
        if let Some(csrf) = response.header(CSRF_ANSWER_HEADER) {
            self.csrf = Some(csrf.to_owned());
        }
        Ok(response)
    }

    /// The value of the header that carries the token.
    ///
    /// The answer is `None` when there is no token: a request without a token
    /// is refused by the surface, and a request with an empty token would be
    /// refused as well.
    fn authorization(&self) -> Result<Option<String>, ClientError> {
        let token = match &self.token {
            TokenSource::Fixed(token) => token.clone(),
            TokenSource::File(path) => read_token(path)?,
        };
        if token.is_empty() {
            return Ok(None);
        }
        let bearer = BEARER;
        Ok(Some(format!("{bearer} {token}")))
    }
}

#[cfg(test)]
mod tests;
