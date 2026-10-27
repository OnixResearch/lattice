//! Explicit, effect-free consumer of a live Tile pane metadata observation.
use std::io::Read;
use std::io::Write;
use std::time::Duration;

use lattice::compose_surface::ComposeError;
use lattice::compose_surface::ReceiptPublisher;
use lattice::compose_surface::TileSdkPanePort;
use lattice::compose_surface::activate;
use lattice::compose_surface::decode_request;
use lattice_compose_binding::ActivationReceipt;
use lattice_compose_binding::MAXIMUM_RECEIPT_BYTES;
use lattice_compose_binding::MAXIMUM_REQUEST_BYTES;
use lattice_workflow_contract::Identity;

struct StdoutReceipt;

impl ReceiptPublisher for StdoutReceipt {
    fn publish(&mut self, receipt: &ActivationReceipt) -> Result<(), ComposeError> {
        let mut bytes = serde_json::to_vec(receipt).map_err(|_| ComposeError::PublicationFailed)?;
        if bytes.len() > MAXIMUM_RECEIPT_BYTES {
            return Err(ComposeError::PublicationFailed);
        }
        bytes.push(b'\n');
        std::io::stdout().lock().write_all(&bytes).map_err(|_| ComposeError::PublicationFailed)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let usage = || std::io::Error::other("usage: compose-pane SOCKET DAEMON_BLAKE3_IDENTITY REQUEST_JSON");
    let socket = args.next().ok_or_else(usage)?;
    let daemon = args.next().ok_or_else(usage)?.into_string().map_err(|_| usage())?;
    let request_path = args.next().ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }
    let authority = Identity::new(daemon).map_err(|error| std::io::Error::other(error.to_string()))?;
    let mut bytes = Vec::new();
    std::fs::File::open(request_path)?
        .take((MAXIMUM_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let request = decode_request(&bytes)?;
    lattice_compose_binding::validate_request(&request).map_err(|error| std::io::Error::other(error.to_string()))?;
    // Connection errors intentionally omit the socket path and producer details.
    let client = tokio::time::timeout(Duration::from_secs(10), tile_sdk::TileClient::connect(socket))
        .await
        .map_err(|_| ComposeError::ObservationTimeout)?
        .map_err(|_| ComposeError::ObservationUnavailable)?;
    let mut observations = TileSdkPanePort::new(client, authority);
    activate(&mut observations, &mut StdoutReceipt, request).await?;
    Ok(())
}
