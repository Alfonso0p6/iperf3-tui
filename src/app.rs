use crate::server::Server;

#[derive(Debug, Clone, PartialEq)]
pub enum ActivePanel {
    InputSettings,
    ServerList,
    LiveGraph,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputField {
    Host,
    Port,
    Duration,
    Streams,
    Mode,
}

impl InputField {
    pub fn next(&self) -> Self {
        match self {
            Self::Host => Self::Port,
            Self::Port => Self::Duration,
            Self::Duration => Self::Streams,
            Self::Streams => Self::Mode,
            Self::Mode => Self::Host,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Host => Self::Mode,
            Self::Port => Self::Host,
            Self::Duration => Self::Port,
            Self::Streams => Self::Duration,
            Self::Mode => Self::Streams,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestMode {
    Download,
    Upload,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestDirection {
    Download,
    Upload,
}

impl TestMode {
    pub fn next(self) -> Self {
        match self {
            Self::Download => Self::Upload,
            Self::Upload => Self::Both,
            Self::Both => Self::Download,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Download => "Download",
            Self::Upload => "Upload",
            Self::Both => "Download + Upload",
        }
    }
}

#[derive(Debug, Clone)]
pub struct IperfConfig {
    pub duration_secs: u64,
    pub parallel_streams: u8,
    pub mode: TestMode,
}

pub struct AppState {
    pub running: bool,
    pub is_testing: bool,
    pub is_loading_servers: bool,
    pub active_panel: ActivePanel,
    pub active_field: InputField,

    pub custom_host: String,
    pub custom_port: String,
    pub duration_input: String,
    pub streams_input: String,

    pub config: IperfConfig,

    pub servers: Vec<Server>,
    pub selected_server_idx: usize,
    pub server_search: String,
    pub server_search_active: bool,

    pub bandwidth_samples: Vec<(f64, f64)>,
    pub download_samples: Vec<(f64, f64)>,
    pub upload_samples: Vec<(f64, f64)>,
    pub current_mbps: f64,
    pub peak_mbps: f64,
    pub avg_mbps: f64,
    pub download_current_mbps: f64,
    pub download_peak_mbps: f64,
    pub download_avg_mbps: f64,
    pub upload_current_mbps: f64,
    pub upload_peak_mbps: f64,
    pub upload_avg_mbps: f64,
    pub current_status: String,
    pub max_bandwidth_found: f64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            running: true,
            is_testing: false,
            is_loading_servers: true,
            active_panel: ActivePanel::ServerList,
            active_field: InputField::Host,

            custom_host: String::from("ping.online.net"),
            custom_port: String::from("5201"),
            duration_input: String::from("10"),
            streams_input: String::from("1"),

            config: IperfConfig {
                duration_secs: 10,
                parallel_streams: 1,
                mode: TestMode::Both,
            },

            servers: Vec::new(),
            selected_server_idx: 0,
            server_search: String::new(),
            server_search_active: false,

            bandwidth_samples: Vec::new(),
            download_samples: Vec::new(),
            upload_samples: Vec::new(),
            current_mbps: 0.0,
            peak_mbps: 0.0,
            avg_mbps: 0.0,
            download_current_mbps: 0.0,
            download_peak_mbps: 0.0,
            download_avg_mbps: 0.0,
            upload_current_mbps: 0.0,
            upload_peak_mbps: 0.0,
            upload_avg_mbps: 0.0,
            current_status: String::from("Loading server list..."),
            max_bandwidth_found: 100.0,
        }
    }

    pub fn sync_config_from_inputs(&mut self) {
        if let Ok(d) = self.duration_input.parse::<u64>() {
            if d > 0 {
                self.config.duration_secs = d;
            }
        }
        if let Ok(s) = self.streams_input.parse::<u8>() {
            if s > 0 {
                self.config.parallel_streams = s;
            }
        }
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::InputSettings => ActivePanel::ServerList,
            ActivePanel::ServerList => ActivePanel::LiveGraph,
            ActivePanel::LiveGraph => ActivePanel::InputSettings,
        };
    }

    pub fn previous_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::InputSettings => ActivePanel::LiveGraph,
            ActivePanel::ServerList => ActivePanel::InputSettings,
            ActivePanel::LiveGraph => ActivePanel::ServerList,
        };
    }

    pub fn update_selected_server_to_inputs(&mut self) {
        if !self.servers.is_empty() && self.selected_server_idx < self.servers.len() {
            let s = &self.servers[self.selected_server_idx];
            self.custom_host = s.host.clone();
            self.custom_port = s.port.to_string();
        }
    }

    pub fn filtered_server_indices(&self) -> Vec<usize> {
        let query = self.server_search.trim().to_lowercase();
        self.servers
            .iter()
            .enumerate()
            .filter(|(_, server)| query.is_empty() || server.searchable_text().contains(&query))
            .map(|(index, _)| index)
            .collect()
    }

    pub fn normalize_server_selection(&mut self) {
        let visible = self.filtered_server_indices();
        if visible.is_empty() {
            self.selected_server_idx = 0;
        } else if !visible.contains(&self.selected_server_idx) {
            self.selected_server_idx = visible[0];
            self.update_selected_server_to_inputs();
        }
    }

    pub fn next_server(&mut self) {
        let visible = self.filtered_server_indices();
        if !visible.is_empty() {
            let position = visible
                .iter()
                .position(|&index| index == self.selected_server_idx)
                .unwrap_or(0);
            self.selected_server_idx = visible[(position + 1) % visible.len()];
            self.update_selected_server_to_inputs();
        }
    }

    pub fn previous_server(&mut self) {
        let visible = self.filtered_server_indices();
        if !visible.is_empty() {
            let position = visible
                .iter()
                .position(|&index| index == self.selected_server_idx)
                .unwrap_or(0);
            self.selected_server_idx = if position == 0 {
                visible[visible.len() - 1]
            } else {
                visible[position - 1]
            };
            self.update_selected_server_to_inputs();
        }
    }

    pub fn get_selected_target(&self) -> (String, u16) {
        let port = self.custom_port.parse::<u16>().unwrap_or(5201);
        (self.custom_host.trim().to_string(), port)
    }

    pub fn add_sample(&mut self, direction: TestDirection, timestamp_sec: f64, mbps: f64) {
        self.bandwidth_samples.push((timestamp_sec, mbps));
        self.current_mbps = mbps;
        if mbps > self.peak_mbps {
            self.peak_mbps = mbps;
        }
        if mbps > self.max_bandwidth_found {
            self.max_bandwidth_found = mbps * 1.25;
        }
        let total: f64 = self.bandwidth_samples.iter().map(|(_, m)| m).sum();
        if !self.bandwidth_samples.is_empty() {
            self.avg_mbps = total / self.bandwidth_samples.len() as f64;
        }

        let (samples, current, peak, average) = match direction {
            TestDirection::Download => (
                &mut self.download_samples,
                &mut self.download_current_mbps,
                &mut self.download_peak_mbps,
                &mut self.download_avg_mbps,
            ),
            TestDirection::Upload => (
                &mut self.upload_samples,
                &mut self.upload_current_mbps,
                &mut self.upload_peak_mbps,
                &mut self.upload_avg_mbps,
            ),
        };
        samples.push((timestamp_sec, mbps));
        *current = mbps;
        *peak = (*peak).max(mbps);
        *average = samples.iter().map(|(_, value)| value).sum::<f64>() / samples.len() as f64;
    }

    pub fn reset_test_stats(&mut self) {
        self.bandwidth_samples.clear();
        self.download_samples.clear();
        self.upload_samples.clear();
        self.current_mbps = 0.0;
        self.peak_mbps = 0.0;
        self.avg_mbps = 0.0;
        self.download_current_mbps = 0.0;
        self.download_peak_mbps = 0.0;
        self.download_avg_mbps = 0.0;
        self.upload_current_mbps = 0.0;
        self.upload_peak_mbps = 0.0;
        self.upload_avg_mbps = 0.0;
        self.max_bandwidth_found = 100.0;
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, TestDirection, TestMode};

    #[test]
    fn test_mode_cycles_through_all_options() {
        assert_eq!(TestMode::Download.next(), TestMode::Upload);
        assert_eq!(TestMode::Upload.next(), TestMode::Both);
        assert_eq!(TestMode::Both.next(), TestMode::Download);
    }

    #[test]
    fn throughput_stats_are_separate_by_direction() {
        let mut app = AppState::new();

        app.add_sample(TestDirection::Download, 1.0, 100.0);
        app.add_sample(TestDirection::Upload, 1.0, 50.0);

        assert_eq!(app.download_current_mbps, 100.0);
        assert_eq!(app.download_peak_mbps, 100.0);
        assert_eq!(app.upload_current_mbps, 50.0);
        assert_eq!(app.upload_peak_mbps, 50.0);
        assert_eq!(app.download_avg_mbps, 100.0);
        assert_eq!(app.upload_avg_mbps, 50.0);
    }
}
