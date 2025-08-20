use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use anyhow::Result;
use tokio::net::TcpStream;

// Use a complete handshake for simplicity where a SYN port scanning could be implemented
pub async fn port_ping(socket_address: SocketAddr, timeout: u64) -> Result<u64> {
    let timeout = Duration::from_secs(timeout);
    let now_req = Instant::now();
    let _ = tokio::time::timeout(timeout, TcpStream::connect(&socket_address)).await??;
    Ok(now_req.elapsed().as_millis() as u64)
}
