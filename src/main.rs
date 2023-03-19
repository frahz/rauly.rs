mod commands;
mod models;
mod utils;
mod voice;

use std::{collections::HashSet, env, sync::Arc};

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{
        event::ResumedEvent,
        gateway::Ready,
        guild::Member,
    },
    prelude::*,
};
use songbird::SerenityInit;
use tracing::{error, info};

use crate::commands::{guild::*, math::*, quotes::*, wotd::*};
use crate::voice::*;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn guild_member_addition(&self, ctx: Context, mut _member: Member) {
        let guild_id = env::var("GUILD_ID")
            .expect("Guild ID")
            .parse::<u64>()
            .expect("GUILD_ID as u64");

        let role_id = env::var("ROLE_ID")
            .expect("Role ID")
            .parse::<u64>()
            .expect("ROLE_ID as u64");

        let user_id = _member.user.id.as_u64().to_owned();

        if _member.guild_id.as_u64() != &guild_id {
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

#[group]
#[commands(multiply, quote, word, get_guild)]
#[sub_groups(voice)]
struct General;

#[group]
#[prefix = "v"]
#[commands(join, leave, play, pause, resume, search, skip, stop, info)]
struct Voice;

#[tokio::main]
async fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("~"))
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .register_songbird()
        .event_handler(Handler)
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
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
