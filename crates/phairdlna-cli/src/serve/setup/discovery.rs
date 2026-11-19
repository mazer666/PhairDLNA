//! The SSDP discovery socket and its task.

use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use phairdlna_core::config::Config;
use phairdlna_core::identity;
use tokio::task::JoinHandle;

use super::super::{log::Logger, ssdp, ServeOptions};

/// Opens the discovery socket and starts the tasks that use it.
pub async fn start_ssdp(
    options: &ServeOptions,
    identity: &identity::InstalledIdentity,
    config: &Config,
    host: Ipv4Addr,
    port: u16,
    logger: &Logger,
    discovery_available: Arc<AtomicBool>,
) -> (
    Option<Arc<ssdp::SsdpSocket>>,
    Option<tokio::sync::watch::Sender<bool>>,
    Option<JoinHandle<()>>,
) {
    if !options.ssdp || !config.server.discovery_enabled {
        return (None, None, None);
    }
    let server = phairdlna_core::ssdp::ServerIdentity {
        friendly_name: config.server.device_name.clone(),
        product: identity::PRODUCT_NAME.to_owned(),
        version: phairdlna_core::PROJECT_VERSION.to_owned(),
    };
    let multicast_port = options.ssdp_port.unwrap_or(config.server.ssdp_port);
    let advertise_port = options.ssdp_advertise_port.unwrap_or(multicast_port);
    let socket = match ssdp::SsdpSocket::bind(
        &identity.as_device_identity(),
        server,
        host,
        port,
        multicast_port,
        advertise_port,
        logger.for_component("ssdp"),
    )
    .await
    {
        Ok(socket) => socket,
        Err(error) => {
            logger.warn("the server cannot be discovered", &[("reason", &error)]);
            return (None, None, None);
        }
    };
    let (stop, stopped) = tokio::sync::watch::channel(false);
    let socket = Arc::new(socket);
    let task_socket = Arc::clone(&socket);
    let task = tokio::spawn(async move {
        discovery_available.store(true, Ordering::Release);
        task_socket.run(stopped).await;
        discovery_available.store(false, Ordering::Release);
    });
    (Some(socket), Some(stop), Some(task))
}
