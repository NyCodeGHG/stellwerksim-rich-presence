use std::time::Duration;

use backon::{ConstantBuilder, Retryable};
use color_eyre::Result;
use stellwerksim::Plugin;
use tokio::{sync::mpsc::Sender, task::JoinHandle, time::Instant};

use crate::Event;

pub struct StsActor {
    plugin: Plugin,
    sender: Sender<Event>,
}

impl StsActor {
    pub async fn spawn(sender: Sender<Event>) -> Result<JoinHandle<Result<()>>> {
        let plugin = try_create_sts_plugin(Duration::from_secs(2)).await?;
        let actor = StsActor { plugin, sender };
        let handle = tokio::spawn(async move { actor.start().await });
        Ok(handle)
    }

    async fn start(mut self) -> Result<()> {
        loop {
            let now = Instant::now();
            match self.plugin.system_info().await {
                Err(stellwerksim::Error::Network(error)) => {
                    tracing::error!("Network error: {error}");
                    self.sender.send(Event::ClearPresence).await?;
                    // Try to reconnect
                    self.plugin = try_create_sts_plugin(Duration::from_secs(5)).await?;
                }
                Err(error) => {
                    tracing::error!("An error occured: {error}");
                }
                Ok(value) => {
                    self.sender.send(Event::UpdatePresence(value)).await?;
                }
            }
            tokio::time::sleep_until(now + Duration::from_secs(1)).await;
        }
    }
}

async fn try_create_sts_plugin(delay: Duration) -> Result<Plugin> {
    create_sts_plugin
        .retry(
            &ConstantBuilder::default()
                .with_delay(delay)
                .with_max_times(usize::MAX),
        )
        .await
}

async fn create_sts_plugin() -> Result<Plugin> {
    Ok(Plugin::builder()
        .name("Discord Rich Presence")
        .author("Marie Ramlow")
        .version(env!("CARGO_PKG_VERSION"))
        .description("Discord Rich Presence for StellwerkSim.")
        .connect()
        .await?)
}
