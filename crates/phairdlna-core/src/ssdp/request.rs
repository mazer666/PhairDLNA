//! Parsing an incoming search, and building the answers.
//!
//! What: the pure half of the discovery protocol — reading an `M-SEARCH`
//!       datagram and writing the bytes of a reply or an advertisement.
//! Why: the half that can be tested without a socket is the half that decides
//!       whether the server answers at all, and whether the answer is correct
//!       (`INT-001` … `INT-014`).
//! Example: `parse_search` turns a datagram into a `Search`, and
//!       `build_response` turns a `Search` plus a target into the bytes that go
//!       back to the client.

use std::time::{SystemTime, UNIX_EPOCH};

use super::constants::{
    CACHE_CONTROL_MAX_AGE, DESCRIPTION_PATH, MAN_DISCOVER, MAX_MX_SECONDS, METHOD_SEARCH,
    MIN_MX_SECONDS, NTS_ALIVE, NTS_BYEBYE,
};
use super::targets::Target;

/// A parsed search, or the reason it was ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Search {
    /// The value of the `ST` header.
    pub search_target: String,
    /// The value of the `MX` header, in seconds.
    pub mx: u32,
    /// The value of the `MAN` header.
    pub man: String,
}

/// Why a datagram was not treated as a search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ignored {
    /// The datagram is not an `M-SEARCH`.
    NotASearch,
    /// The `MAN` header does not carry `"ssdp:discover"`.
    WrongMan,
    /// There is no `MX` header.
    NoMx,
    /// The `MX` header is out of range.
    BadMx,
    /// There is no `ST` header.
    NoSearchTarget,
}

/// Parses an `M-SEARCH` datagram.
///
/// A datagram that is not a valid search is **ignored**, not an error: SSDP runs on
/// an untrusted network, and a malformed datagram is a normal event (`INT-010`).
pub fn parse_search(datagram: &str) -> Result<Search, Ignored> {
    let mut lines = datagram.split("\r\n").flat_map(|line| line.split('\n'));
    let request_line = lines.next().unwrap_or_default().trim();
    if !request_line.starts_with(METHOD_SEARCH) {
        return Err(Ignored::NotASearch);
    }
    let headers = read_headers(lines);
    let man = headers
        .iter()
        .find(|(name, _)| name == "man")
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    if !man.contains(MAN_DISCOVER) {
        return Err(Ignored::WrongMan);
    }
    let mx = headers
        .iter()
        .find(|(name, _)| name == "mx")
        .map(|(_, value)| value.trim().parse::<u32>())
        .transpose()
        .map_err(|_| Ignored::BadMx)?
        .ok_or(Ignored::NoMx)?;
    if !(MIN_MX_SECONDS..=MAX_MX_SECONDS).contains(&mx) {
        return Err(Ignored::BadMx);
    }
    let search_target = headers
        .iter()
        .find(|(name, _)| name == "st")
        .map(|(_, value)| value.clone())
        .ok_or(Ignored::NoSearchTarget)?;
    Ok(Search {
        search_target,
        mx,
        man,
    })
}

/// Reads the headers of a request, lower-casing the names.
fn read_headers<'a, I>(lines: I) -> Vec<(String, String)>
where
    I: Iterator<Item = &'a str>,
{
    let mut headers = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }
    headers
}

/// The identity the server sends in its headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerIdentity {
    /// The name a client shows.
    pub friendly_name: String,
    /// The product name in the `SERVER` header.
    pub product: String,
    /// The product version in the `SERVER` header.
    pub version: String,
}

impl ServerIdentity {
    /// The value of the `SERVER` header.
    pub fn server_header(&self) -> String {
        format!(
            "{}/UPnP/1.1 {}/{}",
            std::env::consts::OS,
            self.product,
            self.version
        )
    }
}

/// Builds the bytes of the reply to a search.
///
/// The reply is unicast to the requester (`INT-003`) and carries the headers the
/// device architecture requires (`INT-001`).
pub fn build_response(target: &Target, server: &ServerIdentity, location: &str) -> String {
    let mut response = String::from("HTTP/1.1 200 OK\r\n");
    response.push_str(&format!(
        "CACHE-CONTROL: max-age={CACHE_CONTROL_MAX_AGE}\r\n"
    ));
    response.push_str(&format!("DATE: {}\r\n", http_date_now()));
    response.push_str("EXT:\r\n");
    response.push_str(&format!("LOCATION: {location}\r\n"));
    response.push_str(&format!("SERVER: {}\r\n", server.server_header()));
    response.push_str(&format!("ST: {}\r\n", target.nt));
    response.push_str(&format!("USN: {}\r\n", target.usn));
    response.push_str("\r\n");
    response
}

/// Builds the bytes of an advertisement.
///
/// `alive` announces the device; `byebye` withdraws it (`INT-004`).
pub fn build_advertisement(
    target: &Target,
    server: &ServerIdentity,
    location: &str,
    alive: bool,
) -> String {
    let mut advertisement = String::from("NOTIFY * HTTP/1.1\r\n");
    advertisement.push_str(&format!(
        "CACHE-CONTROL: max-age={CACHE_CONTROL_MAX_AGE}\r\n"
    ));
    advertisement.push_str(&format!("DATE: {}\r\n", http_date_now()));
    advertisement.push_str(&format!("LOCATION: {location}\r\n"));
    advertisement.push_str(&format!("NT: {}\r\n", target.nt));
    advertisement.push_str(&format!(
        "NTS: {}\r\n",
        if alive { NTS_ALIVE } else { NTS_BYEBYE }
    ));
    advertisement.push_str(&format!("SERVER: {}\r\n", server.server_header()));
    advertisement.push_str(&format!("USN: {}\r\n", target.usn));
    advertisement.push_str("\r\n");
    advertisement
}

/// The location URL of the device description, for an interface address.
pub fn location_url(host: &str, port: u16) -> String {
    format!("http://{host}:{port}{DESCRIPTION_PATH}")
}

/// The delay before answering a search, in milliseconds.
///
/// The specification asks for a uniform random delay inside `[0, MX)` seconds. The
/// randomness comes from the caller, so that this function stays pure and
/// testable (`INT-008`).
pub fn reply_delay_millis(mx: u32, random_millis: u64) -> u64 {
    let window = u64::from(mx) * 1000;
    if window == 0 {
        return 0;
    }
    random_millis % window
}

/// The current time in the HTTP date format.
fn http_date_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or_default();
    format_http_date(seconds)
}

/// Formats a Unix timestamp as an HTTP date.
///
/// The format is fixed by RFC 9110: `Sun, 06 Nov 1994 08:49:37 GMT`.
pub fn format_http_date(seconds: u64) -> String {
    const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let days = seconds / 86_400;
    let time_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    // The epoch, 1970-01-01, was a Thursday - which is index 3 in a week that
    // starts on Monday.
    let weekday = ((days + 3) % 7) as usize;
    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
        DAYS[weekday],
        day,
        MONTHS[(month - 1) as usize],
        year,
        time_of_day / 3600,
        (time_of_day % 3600) / 60,
        time_of_day % 60
    )
}

/// Converts a day count since the epoch into a calendar date.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_prime + 2) / 5 + 1) as u32;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::{
        build_advertisement, build_response, format_http_date, location_url, parse_search,
        reply_delay_millis, Ignored, ServerIdentity,
    };
    use crate::ssdp::targets::{advertised_targets, matching_targets, DeviceIdentity};

    /// A search that a real client would send.
    const SEARCH: &str = "M-SEARCH * HTTP/1.1\r\n\
         HOST: 239.255.255.250:1900\r\n\
         MAN: \"ssdp:discover\"\r\n\
         MX: 3\r\n\
         ST: ssdp:all\r\n\
         USER-AGENT: macOS/14.0 UPnP/1.0\r\n\r\n";

    /// The identity used by the tests.
    fn identity() -> DeviceIdentity {
        DeviceIdentity::new("11111111-2222-3333-4444-555555555555", "Living room")
    }

    /// The server identity used by the tests.
    fn server() -> ServerIdentity {
        ServerIdentity {
            friendly_name: "Living room".to_owned(),
            product: "PhairDLNA".to_owned(),
            version: "0.0.0".to_owned(),
        }
    }

    /// A well-formed search is understood.
    #[test]
    fn a_valid_search_is_parsed() {
        let search = parse_search(SEARCH).expect("the search should parse");
        assert_eq!(search.search_target, "ssdp:all");
        assert_eq!(search.mx, 3);
    }

    /// The negative case: a datagram that is not a search is ignored.
    #[test]
    fn another_method_is_ignored() {
        assert_eq!(
            parse_search("NOTIFY * HTTP/1.1\r\n\r\n"),
            Err(Ignored::NotASearch)
        );
    }

    /// The negative case: a search without the discovery token is ignored.
    #[test]
    fn a_wrong_man_is_ignored() {
        let datagram = SEARCH.replace("\"ssdp:discover\"", "\"something:else\"");
        assert_eq!(parse_search(&datagram), Err(Ignored::WrongMan));
    }

    /// The negative case: a search without `MX` is ignored, as the
    /// specification demands.
    #[test]
    fn a_search_without_mx_is_ignored() {
        let datagram: String = SEARCH
            .lines()
            .filter(|line| !line.starts_with("MX:"))
            .collect::<Vec<_>>()
            .join("\r\n");
        assert_eq!(parse_search(&datagram), Err(Ignored::NoMx));
    }

    /// The boundary case: `MX` outside 1…5 is ignored.
    #[test]
    fn an_out_of_range_mx_is_ignored() {
        let too_small = SEARCH.replace("MX: 3", "MX: 0");
        let too_large = SEARCH.replace("MX: 3", "MX: 6");
        assert_eq!(parse_search(&too_small), Err(Ignored::BadMx));
        assert_eq!(parse_search(&too_large), Err(Ignored::BadMx));
    }

    /// The reply carries every header the device architecture requires.
    #[test]
    fn the_reply_carries_the_required_headers() {
        let target = &matching_targets(&identity(), "ssdp:all")[0];
        let reply = build_response(target, &server(), &location_url("192.168.1.20", 8200));
        assert!(reply.starts_with("HTTP/1.1 200 OK\r\n"));
        for header in [
            "CACHE-CONTROL",
            "DATE",
            "EXT",
            "LOCATION",
            "SERVER",
            "ST",
            "USN",
        ] {
            assert!(reply.contains(&format!("{header}:")), "{header} is missing");
        }
        assert!(reply.contains("CACHE-CONTROL: max-age=1800"));
        assert!(reply.contains("LOCATION: http://192.168.1.20:8200/description.xml"));
    }

    /// The advertisement says whether it is alive or a goodbye.
    #[test]
    fn the_advertisement_carries_its_nts() {
        let target = &advertised_targets(&identity())[0];
        let alive = build_advertisement(
            target,
            &server(),
            "http://192.168.1.20:8200/description.xml",
            true,
        );
        let bye = build_advertisement(
            target,
            &server(),
            "http://192.168.1.20:8200/description.xml",
            false,
        );
        assert!(alive.contains("NTS: ssdp:alive"));
        assert!(bye.contains("NTS: ssdp:byebye"));
        assert!(alive.starts_with("NOTIFY * HTTP/1.1"));
    }

    /// The reply delay stays inside the window the client asked for.
    #[test]
    fn the_reply_delay_stays_inside_the_window() {
        assert_eq!(reply_delay_millis(3, 0), 0);
        assert_eq!(reply_delay_millis(3, 2_999), 2_999);
        assert_eq!(reply_delay_millis(3, 3_000), 0);
        assert_eq!(reply_delay_millis(5, 4_999), 4_999);
    }

    /// The `SERVER` header names the platform the core was built for.
    #[test]
    fn the_server_header_names_the_platform() {
        let header = server().server_header();
        assert!(header.contains(std::env::consts::OS));
        assert!(header.contains("UPnP/1.1"));
        assert!(header.contains("PhairDLNA"));
    }

    /// The HTTP date is the format the specification fixes.
    #[test]
    fn the_http_date_is_the_specified_format() {
        assert_eq!(
            format_http_date(784_111_777),
            "Sun, 06 Nov 1994 08:49:37 GMT"
        );
    }
}
