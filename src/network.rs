use std::net::{IpAddr, SocketAddr};
use tokio::{
    net::TcpStream,
    time::{timeout, Duration},
};

pub async fn is_host_reachable(ip_addr: IpAddr, port: u16) -> Result<(), String> {
    const CONNECTION_TIMEOUT: u64 = 100;

    let socket_addr = SocketAddr::new(ip_addr, port);

    match timeout(
        Duration::from_millis(CONNECTION_TIMEOUT),
        TcpStream::connect(socket_addr),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err("Connection timed out".to_string()),
    }
}
