use iroh::{discovery::DiscoveryEvent, Endpoint};
use n0_future::StreamExt;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{info, warn};

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static SHUTDOWN_SENDER: OnceLock<Arc<Mutex<broadcast::Sender<()>>>> = OnceLock::new();

fn initialize_logging() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Use RUST_LOG env var if set, otherwise use default filter
        // Default: info for mdns_peer, debug for swarm_discovery (to see mDNS activity)
        let default_filter = "mdns_peer=info,swarm_discovery=debug,iroh=info";

        let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| default_filter.to_string());

        tracing_subscriber::fmt().with_env_filter(filter).init();
    });
}

/// Initialize with a given peer identifier
fn start_peer(identifier: &'static str) -> bool {
    initialize_logging();

    info!("{} starting...", identifier);

    // Create tokio runtime if needed
    let rt = RUNTIME
        .get_or_init(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"));

    // Create shutdown channel if needed
    let shutdown_sender = SHUTDOWN_SENDER.get_or_init(|| {
        let (tx, _) = broadcast::channel(1);
        Arc::new(Mutex::new(tx))
    });

    let shutdown_rx = shutdown_sender.lock().unwrap().subscribe();

    rt.spawn(async move {
        match run_peer(identifier, shutdown_rx).await {
            Ok(_) => info!("{} completed successfully", identifier),
            Err(e) => warn!("{} error: {}", identifier, e),
        }
    });

    true
}

/// Start peer with given identifier (for iOS)
#[no_mangle]
pub extern "C" fn peer_start(identifier: *const std::os::raw::c_char) -> bool {
    if identifier.is_null() {
        warn!("peer_start called with null identifier");
        return false;
    }

    let c_str = unsafe { std::ffi::CStr::from_ptr(identifier) };
    let id = match c_str.to_str() {
        Ok(s) => s,
        Err(e) => {
            warn!("Invalid UTF-8 in identifier: {}", e);
            return false;
        }
    };

    // Convert to static string (leaks but OK for app lifecycle)
    let static_id: &'static str = Box::leak(id.to_string().into_boxed_str());
    start_peer(static_id)
}

/// Legacy name for backwards compatibility (defaults to "bob")
#[no_mangle]
pub extern "C" fn bob_start() -> bool {
    start_peer("bob")
}

/// Stop the peer
#[no_mangle]
pub extern "C" fn peer_stop() {
    info!("Stopping peer...");

    if let Some(sender) = SHUTDOWN_SENDER.get() {
        let _ = sender.lock().unwrap().send(());
        info!("Shutdown signal sent");
    } else {
        warn!("Peer was never started");
    }
}

async fn run_peer(
    identifier: &str,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    info!("Creating endpoint with mDNS discovery...");

    // Create endpoint with mDNS discovery and user data
    let user_data = identifier.parse()?;
    let endpoint = Endpoint::builder()
        .discovery_local_network()
        .user_data_for_discovery(user_data)
        .bind()
        .await?;

    let node_id = endpoint.node_id();
    info!("{} node ID: {}", identifier, node_id);

    info!("Listening for peers via mDNS discovery...");

    // Subscribe to discovery events to see user_data
    let my_node_id = node_id;
    let mut discovery_stream = endpoint.discovery_stream();
    let mut discovery_shutdown = shutdown_rx.resubscribe();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                event = discovery_stream.next() => {
                    match event {
                        Some(Ok(DiscoveryEvent::Discovered(item))) => {
                            let discovered_node_id = item.node_id();

                            // Skip self-discovery (not interesting)
                            if discovered_node_id == my_node_id {
                                continue;
                            }

                            // Check user_data to definitively identify the peer
                            let user_data = item.node_info().data.user_data();

                            info!("Peer discovered:");
                            info!("  Node ID: {}", discovered_node_id);
                            info!("  User data: {:?}", user_data);
                            info!("  Source: {}", item.provenance());

                            if let Some(ref data) = user_data {
                                info!("[[[ SUCCESS ]]]: Discovered peer '{}'!", data);
                            } else {
                                info!("  Note: No user_data (legacy iroh peer or different app)");
                            }
                        }
                        Some(Ok(DiscoveryEvent::Expired(node_id))) => {
                            info!("Peer expired: {}", node_id);
                        }
                        Some(Err(e)) => {
                            warn!("Discovery error: {}", e);
                        }
                        None => break,
                    }
                }
                _ = discovery_shutdown.recv() => {
                    info!("Discovery task shutting down...");
                    break;
                }
            }
        }
    });

    // Show periodic summary
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let remotes: Vec<_> = endpoint.remote_info_iter().collect();
                if remotes.is_empty() {
                    warn!("No peers discovered yet");
                } else {
                    info!("Total peers in routing table: {}", remotes.len());
                }
            }
            _ = shutdown_rx.recv() => {
                info!("Peer shutting down...");
                // Close endpoint gracefully
                endpoint.close().await;
                info!("Peer shutdown complete");
                break;
            }
        }
    }

    Ok(())
}

/// Legacy name for backwards compatibility
#[no_mangle]
pub extern "C" fn bob_stop() {
    peer_stop()
}

/// Run as desktop binary (used by alice/bob CLI wrappers)
pub async fn run_desktop() -> anyhow::Result<()> {
    initialize_logging();

    // Get identifier from env var or default to "bob"
    let identifier = std::env::var("PEER_ID").unwrap_or_else(|_| "bob".to_string());
    info!("Running as: {}", identifier);

    // For desktop, create a shutdown channel that listens for Ctrl+C
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    // Spawn Ctrl+C handler
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, shutting down...");
                let _ = shutdown_tx.send(());
            }
            Err(err) => {
                warn!("Unable to listen for Ctrl+C: {}", err);
            }
        }
    });

    run_peer(&identifier, shutdown_rx).await
}

// Note: The binary entry point is in src/main.rs
