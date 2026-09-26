use crate::app::{IperfConfig, TestDirection, TestMode};
use crate::event::AppEvent;
use anyhow::Result;
use regex::Regex;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

pub struct IperfRunner;

impl IperfRunner {
    pub async fn run(
        host: String,
        port: u16,
        config: IperfConfig,
        tx: mpsc::UnboundedSender<AppEvent>,
    ) -> Result<()> {
        match config.mode {
            TestMode::Download => Self::run_once(&host, port, &config, true, &tx).await?,
            TestMode::Upload => Self::run_once(&host, port, &config, false, &tx).await?,
            TestMode::Both => {
                Self::run_once(&host, port, &config, true, &tx).await?;
                Self::run_once(&host, port, &config, false, &tx).await?;
            }
        }

        let _ = tx.send(AppEvent::IperfFinished(Ok(())));
        Ok(())
    }

    async fn run_once(
        host: &str,
        port: u16,
        config: &IperfConfig,
        reverse: bool,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) -> Result<()> {
        let mut args = vec![
            "-c".to_string(),
            host.to_string(),
            "-p".to_string(),
            port.to_string(),
            "-t".to_string(),
            config.duration_secs.to_string(),
            "-P".to_string(),
            config.parallel_streams.to_string(),
            "-i".to_string(),
            "1".to_string(),
            "--forceflush".to_string(), // <--- Forza lo svuotamento istantaneo del buffer stdout
        ];

        if reverse {
            args.push("-R".to_string());
        }

        let _ = tx.send(AppEvent::IperfStatus(format!(
            "Avvio {} verso {}:{}...",
            if reverse { "download" } else { "upload" },
            host,
            port
        )));

        let child_result = Command::new("iperf3")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match child_result {
            Ok(child) => child,
            Err(e) => return Err(anyhow::anyhow!("Unable to start iperf3: {e}")),
        };

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Unable to capture iperf3 stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Unable to capture iperf3 stderr"))?;
        let stderr_task = tokio::spawn(async move {
            let mut output = String::new();
            BufReader::new(stderr).read_to_string(&mut output).await?;
            Ok::<String, std::io::Error>(output)
        });
        let mut reader = BufReader::new(stdout).lines();

        let re = Regex::new(
            r"\[\s*(\d+|SUM)\]\s+(\d+(?:\.\d+)?)-(\d+(?:\.\d+)?)\s+sec\s+[\d.]+\s+[KMG]?Bytes\s+([\d.]+)\s+([KMG]?bits/sec)",
        )?;

        // Stream output asynchronously in real time.
        while let Some(line) = reader
            .next_line()
            .await
            .map_err(|e| anyhow::anyhow!("Error reading iperf3 stdout: {e}"))?
        {
            if let Some(caps) = re.captures(&line) {
                let stream_id = &caps[1];
                let end_sec: f64 = caps[3].parse().unwrap_or(0.0);
                let speed_val: f64 = caps[4].parse().unwrap_or(0.0);
                let unit = &caps[5];

                if config.parallel_streams > 1 && stream_id != "SUM" {
                    continue;
                }

                let mbps = match unit {
                    "Kbits/sec" => speed_val / 1000.0,
                    "Mbits/sec" => speed_val,
                    "Gbits/sec" => speed_val * 1000.0,
                    _ => speed_val,
                };

                let _ = tx.send(AppEvent::IperfBandwidth {
                    direction: if reverse {
                        TestDirection::Download
                    } else {
                        TestDirection::Upload
                    },
                    timestamp_sec: end_sec,
                    mbps,
                });
            }
        }

        let status = child.wait().await?;
        let stderr_output = stderr_task
            .await
            .map_err(|e| anyhow::anyhow!("Error collecting iperf3 stderr: {e}"))??;
        if !status.success() {
            let details = stderr_output.trim();
            let message = if details.is_empty() {
                format!("iperf3 exited with code {:?}", status.code())
            } else {
                format!("iperf3 exited with code {:?}: {}", status.code(), details)
            };
            return Err(anyhow::anyhow!(message));
        }

        Ok(())
    }
}
