//! The client of the administration surface for the diagnostics.
//!
//! What: the one request the command line sends to the administration surface
//!       of a running server.
//! Why: the report of a running server must say what the server does now, and
//!      only the server itself knows that. The client is written with the
//!      standard library alone, because a command that has to be installed on
//!      the machine of the user should not drag a web framework with it
//!      (`OPS-013`, `SEC-071`).
//! Example: the client asks `GET /api/diagnostics` on the loopback interface
//!          and reads the document of the answer.
//!
//! **The client never leaves the machine.** The request always names
//! `127.0.0.1`, and the token is read out of the file the owner of the
//! machine may read. The client never opens a connection to another host and
//! never writes the token anywhere (`SEC-071`, `SEC-094`).

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use phairdlna_core::config::Config;
use phairdlna_core::diagnostics::{self, Diagnostics};
use phairdlna_core::json::Json;

/// The address the surface is asked at.
const LOOPBACK: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// How long the client waits for the surface.
const TIMEOUT: Duration = Duration::from_secs(3);

/// The largest answer the client reads, in bytes.
const MAX_ANSWER: usize = 1024 * 1024;

/// The answer of the surface.
pub struct Answer {
    /// The facts of the server, as the surface reported them.
    pub diagnostics: Diagnostics,
}

/// Asks the surface for the diagnostics of the server.
///
/// The answer is the reason the server could not be asked, in words meant for
/// a person: the report of a stopped server says why it is stopped.
pub fn ask(config: &Config) -> Result<Answer, String> {
    if !config.security.admin_enabled {
        return Err("the administration surface is switched off in the configuration".to_owned());
    }
    let token = read_token(&config.security.admin_token_file)?;
    let body = request(config.security.admin_port, &token)?;
    let document = Json::parse(&body)
        .map_err(|error| format!("the answer of the surface cannot be read: {error}"))?;
    Ok(Answer {
        diagnostics: diagnostics::from_json(&document),
    })
}

/// Reads the token of the surface out of its file.
///
/// The file holds the secret of the installation. It is never written
/// anywhere else, and it never appears in an error message.
fn read_token(path: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("the token of the surface cannot be read: {error}"))?;
    let token = text.trim();
    if token.is_empty() {
        return Err("the token file of the surface is empty".to_owned());
    }
    Ok(token.to_owned())
}

/// Asks the surface one question and reads the answer.
///
/// The request carries the token and the custom header the surface demands,
/// and it asks for the connection to be closed after the answer, so that the
/// client knows where the answer ends.
fn request(port: u16, token: &str) -> Result<String, String> {
    let address = SocketAddr::from((LOOPBACK, port));
    let mut stream = TcpStream::connect_timeout(&address, TIMEOUT)
        .map_err(|error| format!("the server does not answer on the surface: {error}"))?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|error| format!("the answer of the surface cannot be read: {error}"))?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(|error| format!("the question of the surface cannot be sent: {error}"))?;
    stream
        .write_all(request_text(port, token).as_bytes())
        .map_err(|error| format!("the question of the surface cannot be sent: {error}"))?;
    let answer = read_answer(&mut stream)?;
    status_of(&answer)
}

/// The bytes of the request.
///
/// The header that names the host is the loopback interface, because the
/// surface answers only to the loopback interface (`SEC-TH-23`).
fn request_text(port: u16, token: &str) -> String {
    format!(
        "GET /api/diagnostics HTTP/1.1\r\n\
         Host: {LOOPBACK}:{port}\r\n\
         Authorization: Bearer {token}\r\n\
         X-Phair-Admin: 1\r\n\
         Accept: application/json\r\n\
         Connection: close\r\n\
         \r\n"
    )
}

/// Reads the whole answer of the surface.
///
/// The answer is read until the connection ends, and it is refused when it is
/// larger than the client accepts: an answer that does not stop would fill the
/// memory of the machine of the user.
fn read_answer(stream: &mut TcpStream) -> Result<String, String> {
    let mut answer: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let read = stream
            .read(&mut chunk)
            .map_err(|error| format!("the answer of the surface cannot be read: {error}"))?;
        if read == 0 {
            break;
        }
        answer.extend_from_slice(&chunk[..read]);
        if answer.len() > MAX_ANSWER {
            return Err("the answer of the surface is too large".to_owned());
        }
    }
    String::from_utf8(answer).map_err(|_| "the answer of the surface is not text".to_owned())
}

/// The body of the answer, when the surface answered with success.
///
/// A refusal of the surface is reported as such: the report of the owner must
/// not pretend that the server said something it did not say.
fn status_of(answer: &str) -> Result<String, String> {
    let (head, body) = answer
        .split_once("\r\n\r\n")
        .ok_or("the answer of the surface is not a whole answer")?;
    let status = head
        .lines()
        .next()
        .ok_or("the answer of the surface has no status line")?;
    let code = status
        .split_whitespace()
        .nth(1)
        .ok_or("the status line of the surface is not one")?;
    if code != "200" {
        return Err(format!("the surface refused the question ({status})"));
    }
    Ok(body.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The positive case: the request names the host, the token and the header.
    #[test]
    fn the_request_carries_what_the_surface_demands() {
        let text = request_text(8210, "abc");
        assert!(text.starts_with("GET /api/diagnostics HTTP/1.1\r\n"));
        assert!(text.contains("Host: 127.0.0.1:8210\r\n"));
        assert!(text.contains("Authorization: Bearer abc\r\n"));
        assert!(text.contains("X-Phair-Admin: 1\r\n"));
    }

    /// The positive case: the body of a successful answer is read.
    #[test]
    fn the_body_of_a_successful_answer_is_read() {
        let answer = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"a\":1}";
        assert_eq!(status_of(answer).expect("the answer is read"), "{\"a\":1}");
    }

    /// The negative case: a refusal of the surface is reported as a refusal.
    #[test]
    fn a_refusal_of_the_surface_is_reported() {
        let answer = "HTTP/1.1 401 Unauthorized\r\n\r\nThe request was refused.";
        let error = status_of(answer).expect_err("the refusal is reported");
        assert!(error.contains("401"), "{error}");
    }

    /// The negative case: an answer that is not an answer is refused.
    #[test]
    fn an_answer_that_is_not_an_answer_is_refused() {
        assert!(status_of("nonsense").is_err());
    }
}
