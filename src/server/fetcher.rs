use super::Server;
use serde::Deserialize;

#[derive(Deserialize)]
struct RemoteServer {
    #[serde(rename = "IP/HOST")]
    host: String,
    #[serde(rename = "PORT")]
    port: String,
    #[serde(rename = "OPTIONS", default)]
    options: String,
    #[serde(rename = "GB/S", default)]
    bandwidth_gbps: String,
    #[serde(rename = "CONTINENT", default)]
    continent: String,
    #[serde(rename = "COUNTRY", default)]
    country: String,
    #[serde(rename = "SITE", default)]
    site: String,
    #[serde(rename = "PROVIDER", default)]
    provider: String,
}

impl RemoteServer {
    fn into_server(self) -> Server {
        let port = self
            .port
            .split('-')
            .next()
            .and_then(|port| port.trim().parse().ok())
            .unwrap_or(5201);

        Server {
            host: self.host,
            port,
            port_display: self.port,
            options: self.options,
            bandwidth_gbps: self.bandwidth_gbps,
            continent: self.continent,
            country: self.country,
            site: self.site,
            provider: self.provider,
            ping_ms: None,
        }
    }
}

pub async fn fetch_public_servers() -> Vec<Server> {
    let url = "https://export.iperf3serverlist.net/listed_iperf3_servers.json";

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();

    if let Ok(client) = client {
        if let Ok(response) = client.get(url).send().await {
            if let Ok(response) = response.error_for_status() {
                if let Ok(remote_servers) = response.json::<Vec<RemoteServer>>().await {
                    return remote_servers
                        .into_iter()
                        .map(RemoteServer::into_server)
                        .collect();
                }
            }
        }
    }

    fallback_servers()
}

pub fn fallback_servers() -> Vec<Server> {
    vec![
        RemoteServer {
            host: "speedtest.milkywan.fr".to_string(),
            port: "9200-9240".to_string(),
            options: String::new(),
            bandwidth_gbps: "40".to_string(),
            continent: "Europe".to_string(),
            country: "FR".to_string(),
            site: "Croissy-Beaubourg".to_string(),
            provider: "MilkyWan".to_string(),
        }
        .into_server(),
        RemoteServer {
            host: "iperf3.moji.fr".to_string(),
            port: "5200-5240".to_string(),
            options: "-R,-6".to_string(),
            bandwidth_gbps: "100".to_string(),
            continent: "Europe".to_string(),
            country: "FR".to_string(),
            site: "Paris".to_string(),
            provider: "moji".to_string(),
        }
        .into_server(),
        RemoteServer {
            host: "ping.online.net".to_string(),
            port: "5200-5209".to_string(),
            options: "-R,-u".to_string(),
            bandwidth_gbps: "100".to_string(),
            continent: "Europe".to_string(),
            country: "FR".to_string(),
            site: "Vitry-sur-Seine".to_string(),
            provider: "Scaleway".to_string(),
        }
        .into_server(),
        RemoteServer {
            host: "speedtest.init7.net".to_string(),
            port: "5201-5204".to_string(),
            options: "-R,-6,-u".to_string(),
            bandwidth_gbps: "20".to_string(),
            continent: "Europe".to_string(),
            country: "CH".to_string(),
            site: "Winterthur".to_string(),
            provider: "Init7".to_string(),
        }
        .into_server(),
        RemoteServer {
            host: "speedtest.nocix.net".to_string(),
            port: "5201-5205".to_string(),
            options: "-R,-6".to_string(),
            bandwidth_gbps: "200".to_string(),
            continent: "North America".to_string(),
            country: "US".to_string(),
            site: "Kansas City".to_string(),
            provider: "NOCIX".to_string(),
        }
        .into_server(),
    ]
}
