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

use std::fmt;
use std::path::Path;

use crate::ipc::http::{HttpRequest, HttpResponse, Transport, TransportError};
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
/// The permissions the token file must have: the owner reads it, nobody else.
#[cfg(unix)]
const PRIVATE_FILE: u32 = 0o600;

/// The path of the state of the server.
const PATH_STATUS: &str = "/api/status";
/// The path of the diagnostics of the server.
const PATH_DIAGNOSTICS: &str = "/api/diagnostics";
/// The path of the scan of the library.
const PATH_RESCAN: &str = "/api/catalogue/rescan";

/// Why the surface could not be asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientError {
    /// The bytes did not travel.
    Transport(TransportError),
    /// The token of the installation was refused.
    Unauthorised,
    /// The surface refused the request.
    Refused {
        /// The number of the refusal.
        status: u16,
    },
    /// The answer was not the answer the client expects.
    Malformed {
        /// What was wrong with it.
        reason: String,
    },
    /// The token of the installation could not be read.
    Token {
        /// What was wrong with it.
        reason: String,
    },
}

impl fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(error) => write!(formatter, "{error}"),
            Self::Unauthorised => formatter.write_str("the token of the installation was refused"),
            Self::Refused { status } => {
                write!(formatter, "the surface refused the request: {status}")
            }
            Self::Malformed { reason } => write!(formatter, "the answer is not expected: {reason}"),
            Self::Token { reason } => write!(formatter, "the token cannot be read: {reason}"),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<TransportError> for ClientError {
    fn from(error: TransportError) -> Self {
        Self::Transport(error)
    }
}

impl From<JsonError> for ClientError {
    fn from(error: JsonError) -> Self {
        Self::Malformed {
            reason: error.to_string(),
        }
    }
}

/// The client of the administration surface.
#[derive(Clone, Debug)]
pub struct AdminClient<T> {
    /// The way the bytes travel.
    transport: T,
    /// The token of the installation.
    token: String,
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
            token: token.into(),
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
        let document = self.read(METHOD_POST, PATH_RESCAN)?;
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
        let response = self.send(method, path)?;
        match response.status {
            200..=299 => Ok(Json::parse(&response.body_text())?),
            401 => Err(ClientError::Unauthorised),
            status => Err(ClientError::Refused { status }),
        }
    }

    /// Builds the request, sends it and keeps the token of the defence.
    fn send(&mut self, method: &str, path: &str) -> Result<HttpResponse, ClientError> {
        let mut request = HttpRequest::new(method, path)
            .with_header(HOST, &self.transport.host())
            .with_header(AUTHORIZATION, &self.authorization())
            .with_header(ADMIN_HEADER, ADMIN_HEADER_VALUE)
            .with_header(ACCEPT, APPLICATION_JSON)
            .with_header(CONNECTION, CLOSE);
        if let Some(csrf) = &self.csrf {
            request = request.with_header(CSRF_HEADER, csrf);
        }
        let response = self.transport.send(&request)?;
        if let Some(csrf) = response.header(CSRF_ANSWER_HEADER) {
            self.csrf = Some(csrf.to_owned());
        }
        Ok(response)
    }

    /// The value of the header that carries the token.
    fn authorization(&self) -> String {
        let bearer = BEARER;
        let token = &self.token;
        format!("{bearer} {token}")
    }
}

/// Reads the token of the installation out of its file.
///
/// The file must be readable by its owner alone: a token that another program
/// of the machine can read is no token at all (`SEC-071`).
pub fn read_token(path: &Path) -> Result<String, ClientError> {
    check_private(path)?;
    let text = std::fs::read_to_string(path).map_err(|error| ClientError::Token {
        reason: error.to_string(),
    })?;
    let token = text.trim().to_owned();
    if token.is_empty() {
        return Err(ClientError::Token {
            reason: "the file holds no token".to_owned(),
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
        let expected = PRIVATE_FILE;
        return Err(ClientError::Token {
            reason: format!("the file is readable by others; expected {expected:o}"),
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
