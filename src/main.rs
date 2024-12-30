mod commands;
mod models;
mod utils;
mod voice;

use crate::voice::cmds::VoiceHttpKey;
use reqwest::Client as HttpClient;
use serenity::{
    all::{GuildId, RoleId},
    async_trait,
    gateway::ShardManager,
    model::{event::ResumedEvent, gateway::Ready, guild::Member},
    prelude::*,
};
use songbird::SerenityInit;
use std::{env, sync::Arc};
use tracing::{error, info};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: serenity::client::Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: serenity::client::Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn guild_member_addition(&self, ctx: serenity::client::Context, mut _member: Member) {
        let guild_id = env::var("GUILD_ID")
            .expect("Guild ID")
            .parse::<GuildId>()
            .expect("GUILD_ID as u64");

        let role_id = env::var("ROLE_ID")
            .expect("Role ID")
            .parse::<RoleId>()
            .expect("ROLE_ID as u64");

        let user_id = _member.user.id;

        if _member.guild_id != guild_id {
            return;
        }

        match ctx
            .http
            .add_member_role(guild_id, user_id, role_id, Some("Default Role"))
            .await
        {
            Ok(_) => info!("added role to new member"),
            Err(e) => {
                error!("error adding role to member: {}", e);
                return;
            }
        };
    }
}

#[tokio::main]
async fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenvy::dotenv().ok();

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create the framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::quotes::quote(),
                commands::math::multiply(),
                commands::wotd::word(),
                commands::guild::get_guild(),
                voice::cmds::voice_cmds(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .register_songbird()
        .event_handler(Handler)
        .type_map_insert::<VoiceHttpKey>(HttpClient::new())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
