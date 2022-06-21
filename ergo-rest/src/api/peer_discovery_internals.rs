#[cfg(target_arch = "wasm32")]
mod chrome;
mod non_chrome;

use std::time::Duration;

use bounded_integer::BoundedU16;
#[cfg(target_arch = "wasm32")]
pub(crate) use chrome::peer_discovery_inner_chrome;
pub(crate) use non_chrome::peer_discovery_inner;

use crate::{NodeConf, NodeError, PeerInfo};

use super::{build_client, set_req_headers};

/// GET on /peers/all endpoint
async fn get_peers_all(node: NodeConf) -> Result<Vec<PeerInfo>, NodeError> {
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join("peers/all").unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    let response = set_req_headers(rb, node).send().await?;
    Ok(response.json::<Vec<PeerInfo>>().await?)
}

struct PeerDiscoverySettings {
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    task_2_buffer_length: usize,
    global_timeout: Duration,
    timeout_of_individual_node_request: Duration,
}
