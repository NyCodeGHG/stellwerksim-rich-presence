use color_eyre::Result;
use stellwerksim_rich_presence::{presence::PresenceActor, sts::StsActor};
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let (sender, receiver) = channel(1);

    StsActor::spawn(sender.clone()).await?;
    PresenceActor::spawn(receiver).await?;

    tracing::info!("Registering Ctrl+C handler");
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to register Ctrl+C Handler");

    tracing::info!("Exiting.");
    Ok(())
}
