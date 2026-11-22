//! Finding a game server on the local network (see common/hostchooser.py).

use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

pub const UDP_PORT: u16 = 8056;
const PING_MESSAGE: &[u8] = b"pclient-game-ping";
const PONG_MESSAGE: &str = "server-game-pong";

/// Send pings to the given hosts and return the first server that answers:
/// (address, game description).
pub fn pick(hosts: &[&str], rounds: usize) -> Option<(SocketAddr, String)> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    let _ = s.set_broadcast(true);
    s.set_read_timeout(Some(Duration::from_millis(200))).ok()?;
    for _ in 0..rounds {
        for host in hosts {
            eprintln!("* Looking for a server on {}... ", host);
            let target = if *host == "<broadcast>" {
                format!("255.255.255.255:{}", UDP_PORT)
            } else {
                format!("{}:{}", host, UDP_PORT)
            };
            let addrs: Vec<SocketAddr> = match target.to_socket_addrs() {
                Ok(a) => a.filter(|a| a.is_ipv4()).collect(),
                Err(_) => continue,
            };
            for addr in addrs {
                let _ = s.send_to(PING_MESSAGE, addr);
            }
            let end = Instant::now() + Duration::from_millis(1000);
            let mut buf = [0u8; 512];
            while Instant::now() < end {
                if let Ok((n, from)) = s.recv_from(&mut buf) {
                    let text = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let parts: Vec<&str> = text.split(':').collect();
                    if parts.len() >= 4 && parts[0] == PONG_MESSAGE {
                        if let Ok(port) = parts[3].parse::<u16>() {
                            let host = if parts[2].is_empty() {
                                from.ip().to_string()
                            } else {
                                parts[2].to_string()
                            };
                            let addr = format!("{}:{}", host, port)
                                .to_socket_addrs()
                                .ok()
                                .and_then(|mut a| a.find(|a| a.is_ipv4()))
                                .unwrap_or(SocketAddr::new(from.ip(), port));
                            eprintln!("* Picking {:?} at {}", parts[1], addr);
                            return Some((addr, parts[1].to_string()));
                        }
                    }
                }
            }
        }
    }
    None
}
