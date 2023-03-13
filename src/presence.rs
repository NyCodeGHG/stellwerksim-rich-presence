use std::time::{Duration, SystemTime};

use backon::{ConstantBuilder, Retryable};
use color_eyre::Result;
use discord_sdk::{
    activity::{ActivityBuilder, Assets, Button, WithButtons},
    wheel::{
        UserState::{Connected, Disconnected},
        Wheel,
    },
    Discord, DiscordApp, Subscriptions,
};
use stellwerksim::protocol::SystemInfo;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

use crate::Event;

pub struct PresenceActor {
    discord: Discord,
    last: Option<SystemInfo>,
    receiver: Receiver<Event>,
}

const APP_ID: i64 = 1076213464592289902;

impl PresenceActor {
    pub async fn spawn(receiver: Receiver<Event>) -> Result<JoinHandle<()>> {
        let discord = PresenceActor::create_discord
            .retry(
                &ConstantBuilder::default()
                    .with_delay(Duration::from_secs(2))
                    .with_max_times(usize::MAX),
            )
            .await?;
        let actor = PresenceActor {
            discord,
            receiver,
            last: None,
        };
        let handle = tokio::spawn(async move {
            actor.start().await;
        });
        Ok(handle)
    }

    async fn start(mut self) {
        loop {
            let Some(event) = self.receiver.recv().await else {
                return;
            };
            tracing::debug!(event = ?event);
            match event {
                Event::UpdatePresence(ref presence) => {
                    if let Some(ref last) = self.last {
                        if last == presence {
                            continue;
                        }
                    }
                    self.last = Some(presence.clone());
                    tracing::trace!(presence = ?presence, "Updating presence");
                    self.discord
                        .update_activity(build_activity_builder(presence))
                        .await
                        .expect("Failed");
                }
                Event::ClearPresence => {
                    self.last = None;
                    self.discord.clear_activity().await.expect("Failed");
                }
                Event::Exit => {
                    self.discord.clear_activity().await.expect("Failed");
                    break;
                }
            }
        }
    }

    async fn create_discord() -> Result<Discord> {
        let (wheel, handler) = Wheel::new(Box::new(|err| {
            tracing::error!(error = ?err, "error");
        }));
        let discord = Discord::new(
            DiscordApp::PlainId(APP_ID),
            Subscriptions::empty(),
            Box::new(handler),
        )?;
        wheel.user().0.changed().await?;
        match &*wheel.user().0.borrow() {
            Connected(user) => {
                tracing::info!(
                    "Connected to discord! User: {}#{}",
                    user.username,
                    user.discriminator.unwrap_or(0)
                );
            }
            Disconnected(error) => {
                panic!("Failed: {error}");
            }
        };
        Ok(discord)
    }
}

fn build_activity_builder(presence: &SystemInfo) -> ActivityBuilder<WithButtons> {
    ActivityBuilder::new()
        .assets(Assets {
            large_image: Some("stellwerksim".to_string()),
            large_text: Some(format!("StellwerkSim Build {}", presence.build)),
            small_image: None,
            small_text: None,
        })
        .details(if presence.online {
            format!("Spielt {}", presence.name)
        } else {
            format!("Übt {}", presence.name)
        })
        .instance(true)
        .start_timestamp(SystemTime::now())
        .with_buttons()
        .button(Button {
            label: "Stellwerk".to_string(),
            url: format!(
                "https://www.stellwerksim.de/anlagen.php#stellwerk={}",
                presence.aid
            ),
        })
        .button(Button {
            label: "StellwerkSim Rich Presence".to_string(),
            url: "https://github.com/NyCodeGHG/stellwerksim-rich-presence".to_string(),
        })
}
