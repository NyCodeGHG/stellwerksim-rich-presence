use std::{future, time::Duration};

use backon::{ConstantBuilder, Retryable};
use color_eyre::Result;
use stellwerksim::Plugin;
use stellwerksim_rich_presence::Event;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let (sender, receiver) = channel(1);
    spawn_sts_plugin(sender.clone()).await?;
    spawn_discord_handler(receiver).await?;
    future::pending::<()>().await;
    Ok(())
}

async fn spawn_sts_plugin(sender: Sender<Event>) -> Result<JoinHandle<()>> {
    let plugin = create_sts_plugin
        .retry(
            &ConstantBuilder::default()
                .with_delay(Duration::from_secs(1))
                .with_max_times(usize::MAX),
        )
        .await?;
    let handle = tokio::spawn(async move {
        let mut plugin = plugin;
        let mut system_info = None;
        let sender = sender;
        loop {
            match plugin.system_info().await {
                Err(stellwerksim::Error::Network(error)) => {
                    tracing::error!("Network Error: {error}");
                    sender.send(Event::ClearPresence).await.expect("uwu");
                    system_info = None;
                    plugin = create_sts_plugin
                        .retry(
                            &ConstantBuilder::default()
                                .with_delay(Duration::from_secs(1))
                                .with_max_times(usize::MAX),
                        )
                        .await
                        .expect("Failed to reconnect");
                }
                Ok(value) => {
                    if let Some(info) = &system_info {
                        if info != &value {
                            system_info = Some(value.clone());
                            sender
                                .send(Event::UpdatePresence(value))
                                .await
                                .expect("uwu");
                        }
                    } else {
                        system_info = Some(value.clone());
                        sender
                            .send(Event::UpdatePresence(value))
                            .await
                            .expect("uwu");
                    }
                }
                _ => {}
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    Ok(handle)
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

async fn spawn_discord_handler(receiver: Receiver<Event>) -> Result<()> {
    stellwerksim_rich_presence::presence::PresenceActor::spawn(receiver).await?;
    Ok(())
}
