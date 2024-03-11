use argh::FromArgs;
use scanner::scan_ip_range;
use std::sync::Arc;
use tokio::{main, sync::mpsc};

mod network;
mod scanner;

#[derive(FromArgs)]
/// IP range scanner
struct Args {
    #[argh(option, short = 'b')]
    /// base ip
    base_ip: String,
    #[argh(option, short = 'p', default = "vec![80]", from_str_fn(parse_ports))]
    ///  "Ports to check (comma-separated)"
    port: Vec<u16>,
}

fn parse_ports(s: &str) -> Result<Vec<u16>, String> {
    s.split(',')
        .map(|p| {
            p.parse::<u16>()
                .map_err(|e| format!("Failed to parse port number: {}", e))
        })
        .collect()
}

#[main]
async fn main() {
    let args: Args = argh::from_env();

    let start_ip: u8 = 1;
    let end_ip: u8 = 255;

    let (sender, mut receiver) = mpsc::channel(255);
    let sender = Arc::new(sender);

    scan_ip_range(args.base_ip, start_ip, end_ip, args.port, sender.clone()).await;

    drop(sender); // Drop sender to signal that no more messages will be sent

    while let Some(message) = receiver.recv().await {
        println!("{}", message);
    }
}
