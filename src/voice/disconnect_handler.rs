use once_cell::sync::Lazy;
use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::RwLock;
use serenity::prelude::*;
use songbird::{Call, Event, EventContext, EventHandler, Songbird};
use std::{sync::Arc, time::Duration};
use tracing::info;

static HANDLER_ADDED: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));
const TIMEOUT_SECS: u64 = 420;

#[derive(Clone)]
pub struct ChannelDisconnect {
    manager: Arc<Songbird>,
    guild_id: GuildId,
}

impl ChannelDisconnect {
    pub fn new(manager: Arc<Songbird>, guild_id: GuildId) -> Self {
        Self { manager, guild_id }
    }

    pub async fn register_handler(&self, handler_lock: &Arc<Mutex<Call>>) {
        if !*HANDLER_ADDED.read().await {
            info!("Register handler for disconnect");
            let mut ha = HANDLER_ADDED.write().await;
            *ha = true;
            let mut handler = handler_lock.lock().await;
            handler.add_global_event(
                Event::Periodic(Duration::from_secs(TIMEOUT_SECS), None),
                self.clone(),
            );
        } else {
            info!("No handler");
        }
    }

    async fn disconnect(&self) {
        let should_close = match self.manager.get(self.guild_id) {
            None => false,
            Some(handler_lock) => {
                let handler = handler_lock.lock().await;
                handler.queue().is_empty()
            }
        };

        if should_close {
            info!("Leaving voice channel.");
            let _dc = self.manager.remove(self.guild_id).await;
            info!("Left voice channel.");
            {
                *HANDLER_ADDED.write().await = false;
            }
        }
    }
}

#[async_trait]
impl EventHandler for ChannelDisconnect {
    async fn act(&self, _: &EventContext<'_>) -> Option<Event> {
        info!("Checking if bot is active");
        self.disconnect().await;
        None
    }
}
