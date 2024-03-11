use std::{net::IpAddr, sync::Arc};
use tokio::{sync::mpsc, task};

use crate::network::is_host_reachable;

pub async fn scan_ip_range(
    base_ip: String,
    start_ip: u8,
    end_ip: u8,
    ports: Vec<u16>,
    sender: Arc<mpsc::Sender<String>>,
) {
    for i in start_ip..=end_ip {
        let sender = Arc::clone(&sender);
        let ip = format!("{}{}", base_ip, i);
        let ports = ports.clone();
        task::spawn(async move {
            if let Ok(ip_addr) = ip.parse::<IpAddr>() {
                for port in &ports {
                    let sender = Arc::clone(&sender);
                    let ip = ip.clone();
                    let port = *port;
                    task::spawn(async move {
                        if is_host_reachable(ip_addr, port).await.is_err() {
                            return;
                        }

                        if let Err(e) = sender.send(format!("{}:{} is connected", ip, port)).await {
                            println!("{e}")
                        }
                    });
                }
            } else {
                sender
                    .send(format!("Invalid IP address {}: {}", ip, "Invalid format"))
                    .await
                    .ok();
            }
        });
    }
}
