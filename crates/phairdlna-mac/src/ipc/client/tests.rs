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
