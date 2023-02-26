use std::thread;

use color_eyre::Result;
use stellwerksim_rich_presence::{presence::PresenceActor, sts::StsActor, tray::TrayActor, Event};
use tokio::sync::mpsc::channel;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    thread::spawn(|| tokio::runtime::Runtime::new()?.block_on(presence()));
    TrayActor::spawn();
    Ok(())
}

async fn presence() -> Result<()> {
    let (sender, receiver) = channel(1);
    let plugin = StsActor::spawn(sender.clone()).await?;
    let discord = PresenceActor::spawn(receiver).await?;
    tracing::info!("Registering Ctrl+C handler");
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to register Ctrl+C Handler");
    tracing::info!("Exiting Gracefully.");
    sender.send(Event::Exit).await?;
    let _ = plugin.await?;
    discord.await?;
    Ok(())
}
