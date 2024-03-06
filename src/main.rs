use argh::FromArgs;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::Duration;

#[derive(FromArgs)]
/// IP range scanner
struct Args {
    #[argh(positional)]
    /// start and end IP addresses in the format "start_ip-end_ip"
    ip_range: String,
    #[argh(positional)]
    /// ports to check
    port: Vec<u16>,
}

#[tokio::main]
async fn main() {
    let args: Args = argh::from_env();

    let (start_ip, end_ip) = parse_ip_range(&args.ip_range).unwrap_or_else(|e| {
        eprintln!("Error parsing IP range: {}", e);
        std::process::exit(1);
    });

    let start_ip: u8 = 1;
    let end_ip: u8 = 255;
    let base_ip = "200.200.1.";

    let ports = vec![3100, 3200, 3201];

    let (sender, mut receiver) = mpsc::channel(255);

    let sender = Arc::new(sender);

    for i in start_ip..=end_ip {
        let sender = sender.clone();
        let ip = format!("{}{}", base_ip, i);
        let ports = ports.clone();
        tokio::spawn(async move {
            match ip.parse::<IpAddr>() {
                Ok(ip_addr) => {
                    for &port in &ports {
                        let socket_addr = SocketAddr::new(ip_addr, port);
                        let sender = sender.clone();
                        let ip = ip.clone();
                        tokio::spawn(async move {
                            match is_host_reachable(socket_addr).await {
                                Ok(()) => {
                                    sender
                                        .send(format!("{}:{} is connected", ip, port))
                                        .await
                                        .unwrap();
                                }
                                Err(e) => {
                                    /*  sender
                                        .send(format!("{}:{} connection error: {}", ip, port, e))
                                        .await
                                        .unwrap();

                                    */
                                }
                            }
                        });
                    }
                }
                Err(e) => {
                    sender
                        .send(format!("Invalid IP address {}: {}", ip, e))
                        .await
                        .unwrap();
                }
            }
        });
    }

    drop(sender); // Drop sender to signal that no more messages will be sent

    while let Some(message) = receiver.recv().await {
        println!("{}", message);
    }
}

fn parse_ip_range(range: &str) -> Result<(IpAddr, IpAddr), &'static str> {
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return Err("Invalid IP range format. Should be 'start_ip-end_ip'");
    }
    let start_ip = IpAddr::from_str(parts[0]).map_err(|_| "Invalid start IP address")?;
    let end_ip = IpAddr::from_str(parts[1]).map_err(|_| "Invalid end IP address")?;
    Ok((start_ip, end_ip))
}

async fn is_host_reachable(socket_addr: SocketAddr) -> Result<(), String> {
    const CONNECTION_TIME: u64 = 100;

    match tokio::time::timeout(
        Duration::from_millis(CONNECTION_TIME),
        TcpStream::connect(socket_addr),
    )
    .await
    {
        Ok(Ok(mut stream)) => {
            stream.write_all(b"GET / HTTP/1.0\r\n\r\n").await.unwrap();
            stream.shutdown().await.unwrap(); // Shutdown the stream after use
            Ok(())
        }
        Ok(Err(e)) => Err(format!("Connection error: {}", e)),
        Err(_) => Err("Connection timed out".to_string()),
    }
}
