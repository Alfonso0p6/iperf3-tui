use crate::{app::TestDirection, server::Server};
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Tick,
    Key(KeyEvent),
    IperfBandwidth {
        direction: TestDirection,
        timestamp_sec: f64,
        mbps: f64,
    },
    IperfStatus(String),
    IperfFinished(Result<(), String>),
    ServersLoaded(Vec<Server>),
}

pub struct EventHandler {
    sender: mpsc::UnboundedSender<AppEvent>,
    receiver: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let event_sender = sender.clone();

        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if event_sender.send(AppEvent::Tick).is_err() {
                            break;
                        }
                    }
                    maybe_event = reader.next() => {
                        match maybe_event {
                            Some(Ok(CrosstermEvent::Key(key))) => {
                                if event_sender.send(AppEvent::Key(key)).is_err() {
                                    break;
                                }
                            }
                            Some(Err(_)) | None => break,
                            _ => {}
                        }
                    }
                }
            }
        });

        Self { sender, receiver }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.sender.clone()
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.receiver.recv().await
    }
}
