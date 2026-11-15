//! The discovery socket of the daemon.
//!
//! What: the multicast socket that advertises the server and answers the
//!       searches of the clients.
//! Why: a client finds a DLNA server by asking the network for it. The socket
//!       is the only place where the server is *seen*; everything else it
//!       serves (`F-01`, `INT-001` … `INT-019`).
//! Example: the socket sends `NOTIFY * HTTP/1.1` with `NTS: ssdp:alive` every
//!       few minutes and answers an `M-SEARCH` with a unicast `HTTP/1.1 200 OK`.
//!
//! **The rules of the socket.**
//!
//! - It sends **only** the advertisements and the answers (`SEC-091`).
//! - It answers a search **only** for the targets the server advertises, and
//!   the answer goes back to the address the search came from (`INT-010`).
//! - A datagram it cannot read is **ignored**, not an error: the socket is on
//!   an untrusted network.
//! - It never blocks the server: the discovery runs beside the media delivery,
//!   never in front of it.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use phairdlna_core::ssdp::{
    advertised_targets, build_advertisement, build_response, location_url, matching_targets,
    parse_search, reply_delay_millis, DeviceIdentity, ServerIdentity, Target,
    ADVERTISEMENT_INTERVAL_SECONDS, MULTICAST_GROUP, MULTICAST_TTL,
};
use tokio::net::UdpSocket;

use super::log::Logger;

/// The number of advertisements the server sends when it starts.
///
/// The specification asks for an advertisement when the server comes up; many
/// clients listen a little later, so the first advertisements are repeated.
const START_ADVERTISEMENTS: usize = 3;

/// The time between the first advertisements.
const START_ADVERTISEMENT_GAP: Duration = Duration::from_millis(500);

/// The discovery socket of the server.
pub struct SsdpSocket {
    socket: Arc<UdpSocket>,
    advertise_port: u16,
    /// The number of answers that may wait at the same time.
    answers: Arc<tokio::sync::Semaphore>,
    identity: DeviceIdentity,
    targets: Vec<Target>,
    server: ServerIdentity,
    location: String,
    pub(super) logger: Logger,
}

impl SsdpSocket {
    /// Binds the socket and joins the multicast group.
    ///
    /// The socket binds the SSDP port on every address of the machine and joins
    /// the group, so that the advertisements of the server reach the clients on
    /// the local network (`INT-004`).
    pub async fn bind(
        identity: &DeviceIdentity,
        server: ServerIdentity,
        host: Ipv4Addr,
        port: u16,
        multicast_port: u16,
        advertise_port: u16,
        logger: Logger,
    ) -> Result<Self, String> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, multicast_port))
            .await
            .map_err(|error| format!("the SSDP port {multicast_port} cannot be opened: {error}"))?;
        socket
            .join_multicast_v4(MULTICAST_GROUP, Ipv4Addr::UNSPECIFIED)
            .map_err(|error| format!("the multicast group cannot be joined: {error}"))?;
        // The advertisements must not leave the local link (`INT-016`).
        socket
            .set_multicast_ttl_v4(u32::from(MULTICAST_TTL))
            .map_err(|error| format!("the multicast time to live cannot be set: {error}"))?;
        let location = location_url(&host.to_string(), port);
        Ok(Self {
            socket: Arc::new(socket),
            advertise_port,
            answers: Arc::new(tokio::sync::Semaphore::new(MAX_WAITING_ANSWERS)),
            identity: identity.clone(),
            targets: advertised_targets(identity),
            server,
            location,
            logger,
        })
    }

    /// The targets the server advertises.
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    /// Sends the advertisements that say the server is here.
    ///
    /// The start sends the advertisement a few times in a row, so that a client
    /// that just joined the group hears it; every later cycle sends it once
    /// (`INT-004`).
    pub async fn advertise_alive(&self) -> Result<(), String> {
        self.advertise_rounds(START_ADVERTISEMENTS).await
    }

    /// Sends the advertisement of every target a number of times.
    async fn advertise_rounds(&self, rounds: usize) -> Result<(), String> {
        for round in 0..rounds {
            for target in &self.targets {
                self.send(build_advertisement(
                    target,
                    &self.server,
                    &self.location,
                    true,
                ))
                .await?;
            }
            if round + 1 < rounds {
                tokio::time::sleep(START_ADVERTISEMENT_GAP).await;
            }
        }
        self.logger.info(
            "advertising",
            &[("targets", &self.targets.len().to_string())],
        );
        Ok(())
    }

    /// Sends the advertisements that say the server is gone.
    pub async fn advertise_byebye(&self) -> Result<(), String> {
        for target in &self.targets {
            self.send(build_advertisement(
                target,
                &self.server,
                &self.location,
                false,
            ))
            .await?;
        }
        self.logger.info("the advertisements are withdrawn", &[]);
        Ok(())
    }

    /// Runs the discovery until the socket is closed.
    ///
    /// Three things happen at the same time: the server advertises itself when
    /// it comes up and then every `max-age/4` seconds (`INT-004`), it answers
    /// the searches that arrive (`INT-001`), and it stops when the daemon says
    /// so (`OPS-008`).
    pub async fn run(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        if let Err(error) = self.advertise_alive().await {
            self.logger
                .warn("the advertisement could not be sent", &[("reason", &error)]);
        }
        let mut ticker = tokio::time::interval(advertisement_interval());
        ticker.tick().await;
        let mut buffer = vec![0u8; 2048];
        loop {
            tokio::select! {
                received = self.socket.recv_from(&mut buffer) => {
                    match received {
                        Ok((read, from)) => self.answer_in_background(&buffer[..read], from),
                        Err(error) => {
                            self.logger.warn("a datagram could not be read", &[("reason", &error.to_string())]);
                        }
                    }
                }
                _ = ticker.tick() => {
                    // One round per cycle: the interval between two cycles is
                    // a quarter of the lifetime of the advertisement, and the
                    // advertisement is not repeated inside the cycle.
                    if let Err(error) = self.advertise_rounds(1).await {
                        self.logger.warn("the advertisement could not be sent", &[("reason", &error)]);
                    }
                }
                _ = shutdown.changed() => return,
            }
        }
    }

    /// Answers one datagram in a task of its own.
    ///
    /// The answer is delayed by up to `MX` seconds (`INT-008`); a waiting
    /// answer must not hold up the searches that arrive in the meantime.
    ///
    /// The number of waiting answers is bounded: a host that floods the group
    /// with searches must not make the server open an endless number of tasks
    /// (`PROJECT-STANDARDS.md` §2.4). A search that arrives when the bound is
    /// reached is dropped.
    fn answer_in_background(&self, datagram: &[u8], from: SocketAddr) {
        let Ok(permit) = Arc::clone(&self.answers).try_acquire_owned() else {
            self.logger.warn(
                "a search was dropped: too many answers are waiting",
                &[("bound", &MAX_WAITING_ANSWERS.to_string())],
            );
            return;
        };
        let text = String::from_utf8_lossy(datagram).into_owned();
        let identity = self.identity.clone();
        let server = self.server.clone();
        let location = self.location.clone();
        let logger = self.logger.clone();
        let socket = Arc::clone(&self.socket);
        tokio::spawn(async move {
            let _permit = permit;
            let Ok(search) = parse_search(&text) else {
                return;
            };
            let matching = matching_targets(&identity, &search.search_target);
            if matching.is_empty() {
                return;
            }
            let delay = reply_delay_millis(search.mx, jitter());
            tokio::time::sleep(Duration::from_millis(delay)).await;
            for target in matching {
                let answer = build_response(&target, &server, &location);
                if let Err(error) = socket.send_to(answer.as_bytes(), from).await {
                    logger.warn(
                        "an answer could not be sent",
                        &[("reason", &error.to_string())],
                    );
                }
            }
        });
    }

    /// Sends a datagram to the multicast group.
    ///
    /// The advertisements go to the port the specification names, which is the
    /// port the socket listens on. A test may name another one, so that it can
    /// listen for the advertisements without competing with the server for the
    /// port (`INT-004`).
    async fn send(&self, message: impl AsRef<str>) -> Result<(), String> {
        self.send_to(message, multicast_address(self.advertise_port))
            .await
    }

    /// Sends a datagram to one address.
    async fn send_to(&self, message: impl AsRef<str>, to: SocketAddr) -> Result<(), String> {
        self.socket
            .send_to(message.as_ref().as_bytes(), to)
            .await
            .map(|_| ())
            .map_err(|error| format!("the datagram could not be sent: {error}"))
    }
}

/// The address of the multicast group.
///
/// The port is the one the socket listens on: the specification names 1900, and
/// a test may name another one to stay out of the way of the network.
fn multicast_address(port: u16) -> SocketAddr {
    SocketAddr::new(MULTICAST_GROUP.into(), port)
}

/// A small random number, in milliseconds.
///
/// The answer to a search may be delayed by any moment up to the `MX` of the
/// search, so that two servers do not answer at the same time (`INT-012`).
fn jitter() -> u64 {
    // The value comes from the hash of a random seed, not from the clock: two
    // servers that answer at the same moment must not answer at the same
    // millisecond.
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u32(std::process::id());
    hasher.finish() % 250
}

/// The number of answers that may wait at the same time.
///
/// The bound keeps a flood of searches from opening an endless number of
/// tasks; a search that arrives when the bound is reached is dropped
/// (`PROJECT-STANDARDS.md` §2.4).
const MAX_WAITING_ANSWERS: usize = 32;

/// The time between two advertisements.
pub fn advertisement_interval() -> Duration {
    Duration::from_secs(u64::from(ADVERTISEMENT_INTERVAL_SECONDS))
}
