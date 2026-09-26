use super::Server;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

pub async fn measure_latency(host: &str, port: u16, timeout_duration: Duration) -> Option<u128> {
    let addr = format!("{}:{}", host, port);
    let start = Instant::now();

    match timeout(timeout_duration, TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => Some(start.elapsed().as_millis()),
        _ => None,
    }
}

pub async fn ping_and_sort_servers(mut servers: Vec<Server>) -> Vec<Server> {
    let timeout_duration = Duration::from_millis(1500);

    let tasks: Vec<_> = servers
        .iter()
        .map(|server| {
            let host = server.host.clone();
            let port = server.port;
            tokio::spawn(async move { measure_latency(&host, port, timeout_duration).await })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    for (i, res) in results.into_iter().enumerate() {
        if let Ok(ping_ms) = res {
            servers[i].ping_ms = ping_ms;
        }
    }

    servers.sort_by(|a, b| match (a.ping_ms, b.ping_ms) {
        (Some(p1), Some(p2)) => p1.cmp(&p2),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });

    servers
}
