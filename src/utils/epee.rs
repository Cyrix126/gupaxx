use futures::stream;
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::OnceLock,
    time::{Duration, Instant},
};
use tokio::time::timeout;
use tower::{Service, ServiceExt, make::Shared, util::MapErr};

use cuprate_p2p_core::{
    BroadcastMessage, ClearNet, Network, NetworkZone,
    client::{
        ConnectRequest, Connector, HandshakerBuilder, InternalPeerID,
        handshaker::builder::{DummyAddressBook, DummyCoreSyncSvc, DummyProtocolRequestHandler},
    },
};
use cuprate_wire::{BasicNodeData, common::PeerSupportFlags};

use crate::components::node::TIMEOUT_P2POOL_NODE_PING;

static CONNECTOR: OnceLock<
    Connector<
        ClearNet,
        DummyAddressBook,
        DummyCoreSyncSvc,
        MapErr<Shared<DummyProtocolRequestHandler>, fn(Infallible) -> tower::BoxError>,
        fn(InternalPeerID<<ClearNet as NetworkZone>::Addr>) -> stream::Pending<BroadcastMessage>,
    >,
> = OnceLock::new();

pub async fn ping_epee(addr: SocketAddr) -> Result<u128, tower::BoxError> {
    let mut connector;

    // get the connector or create it
    if let Some(con) = CONNECTOR.get() {
        connector = con.clone();
    } else {
        init_connector();
        connector = CONNECTOR.get().unwrap().clone();
    }

    let now_req = Instant::now();
    // The connection.
    _ = timeout(
        Duration::from_millis(TIMEOUT_P2POOL_NODE_PING as u64),
        connector
            .ready()
            .await?
            .call(ConnectRequest { addr, permit: None }),
    )
    .await??;
    return Ok(now_req.elapsed().as_millis());
}

pub fn init_connector() {
    let handshaker = HandshakerBuilder::<ClearNet>::new(BasicNodeData {
        my_port: 0,
        network_id: Network::Mainnet.network_id(),
        peer_id: rand::random(),
        support_flags: PeerSupportFlags::FLUFFY_BLOCKS,
        rpc_port: 0,
        rpc_credits_per_hash: 0,
    })
    .build();

    let connector = Connector::new(handshaker);

    let _ = CONNECTOR.get_or_init(|| connector.clone());
}
