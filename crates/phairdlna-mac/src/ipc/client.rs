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

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::ipc::http::{HttpRequest, HttpResponse, Transport, TransportError, TransportKind};
use crate::ipc::json::{Json, JsonError};
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

/// Why the surface could not be asked.
///
/// The error carries the *kind* of the failure and the words of the system,
/// and no sentence: the sentence that the user reads comes from the resource
/// file of the language of the user (`UX-020`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientError {
    /// The surface cannot be reached.
    Unreachable {
        /// What the system said.
        reason: String,
    },
    /// The surface cannot be read.
    Unreadable {
        /// What the system said.
        reason: String,
    },
    /// The answer is not an answer of the protocol.
    Invalid {
        /// What was wrong with it.
        reason: String,
    },
    /// The answer is not the answer the client expects.
    Unexpected {
        /// What was wrong with it.
        reason: String,
    },
    /// The token of the installation was refused.
    Unauthorised,
    /// The surface refused the request.
    Refused {
        /// The number of the refusal.
        status: u16,
    },
    /// The token of the installation could not be read.
    Token {
        /// What the system said.
        reason: String,
    },
    /// The file of the token is not there: the server was never started.
    NoToken,
    /// The application is busy and the request was dropped.
    Busy,
}

impl ClientError {
    /// What was wrong, in the words of the system.
    ///
    /// The answer is an empty text when the failure has no words of its own.
    pub fn reason(&self) -> &str {
        match self {
            Self::Unreachable { reason }
            | Self::Unreadable { reason }
            | Self::Invalid { reason }
            | Self::Unexpected { reason }
            | Self::Token { reason } => reason,
            Self::Unauthorised | Self::Refused { .. } | Self::NoToken | Self::Busy => "",
        }
    }
}

impl From<TransportError> for ClientError {
    fn from(error: TransportError) -> Self {
        let reason = error.reason().to_owned();
        match error.kind() {
            TransportKind::Unreachable => Self::Unreachable { reason },
            TransportKind::Unreadable => Self::Unreadable { reason },
            TransportKind::Invalid => Self::Invalid { reason },
        }
    }
}

impl From<JsonError> for ClientError {
    fn from(error: JsonError) -> Self {
        Self::Unexpected {
            reason: error.to_string(),
        }
    }
}

/// Where the token of the installation comes from.
#[derive(Clone, Debug)]
enum TokenSource {
    /// A token that the caller knows.
    Fixed(String),
    /// The file of the token of the installation.
    ///
    /// The file is read before every request: the server may be started
    /// while the application runs, and the application must find it then.
    File(PathBuf),
}

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

    /// The address of the surface, as the browser may open it.
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

/// Reads the token of the installation out of its file.
///
/// The file must be readable by its owner alone: a token that another program
/// of the machine can read is no token at all (`SEC-071`).
pub fn read_token(path: &Path) -> Result<String, ClientError> {
    if !path.exists() {
        return Err(ClientError::NoToken);
    }
    check_private(path)?;
    let text = std::fs::read_to_string(path).map_err(|error| ClientError::Token {
        reason: error.to_string(),
    })?;
    let token = text.trim().to_owned();
    if token.is_empty() {
        return Err(ClientError::Token {
            reason: String::new(),
        });
    }
    Ok(token)
}

/// Refuses a token file that another program of the machine may read.
#[cfg(unix)]
fn check_private(path: &Path) -> Result<(), ClientError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).map_err(|error| ClientError::Token {
        reason: error.to_string(),
    })?;
    let mode = metadata.permissions().mode();
    if mode & 0o077 != 0 {
        return Err(ClientError::Token {
            reason: String::new(),
        });
    }
    Ok(())
}

/// The rule of the private file only exists on the systems that have file
/// permissions; elsewhere the token file is read as it is.
#[cfg(not(unix))]
fn check_private(_path: &Path) -> Result<(), ClientError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{HttpRequest, HttpResponse, HOST, METHOD_GET, PATH_STATUS};
    use crate::ipc::http::HTTP_VERSION;

    /// The bytes of a request carry the request line and the headers.
    #[test]
    fn a_request_is_built() {
        let host = HOST;
        let value = "127.0.0.1:8210";
        let request = HttpRequest::new(METHOD_GET, PATH_STATUS).with_header(host, value);
        let text = String::from_utf8(request.to_bytes()).expect("the bytes are text");
        let method = METHOD_GET;
        let path = PATH_STATUS;
        let version = HTTP_VERSION;
        assert!(
            text.starts_with(&format!("{method} {path} {version}\r\n")),
            "{text}"
        );
        assert!(text.contains(&format!("{host}: {value}\r\n")), "{text}");
        assert!(text.ends_with("\r\n\r\n"), "{text}");
    }

    /// An answer is read out of its bytes.
    #[test]
    fn an_answer_is_read() {
        let version = HTTP_VERSION;
        let length = "Content-Length";
        let ok = "OK";
        let bytes = format!("{version} 200 {ok}\r\n{length}: 2\r\n\r\n{{}}");
        let response = HttpResponse::parse(bytes.as_bytes()).expect("the answer is read");
        assert_eq!(response.status, 200);
        assert_eq!(response.body_text(), "{}");
        assert_eq!(response.header("content-length"), Some("2"));
    }

    /// The negative case: bytes that are not an answer are refused.
    #[test]
    fn bytes_that_are_not_an_answer_are_refused() {
        assert!(HttpResponse::parse(b"hello").is_err());
        assert!(HttpResponse::parse(b"HTTP/1.1\r\n\r\n").is_err());
    }
}
