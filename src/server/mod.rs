pub mod fetcher;
pub mod pinger;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub port_display: String,
    pub options: String,
    pub bandwidth_gbps: String,
    pub continent: String,
    pub country: String,
    pub site: String,
    pub provider: String,
    pub ping_ms: Option<u128>,
}

impl Server {
    pub fn searchable_text(&self) -> String {
        format!(
            "{} {} {} {} {} {} {} {} {}",
            self.host,
            self.port_display,
            self.options,
            self.bandwidth_gbps,
            self.continent,
            self.country,
            self.site,
            self.provider,
            self.ping_ms
                .map_or_else(String::new, |ping| ping.to_string())
        )
        .to_lowercase()
    }
}
