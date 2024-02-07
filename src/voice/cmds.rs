use crate::voice::disconnect_handler::ChannelDisconnect;
use reqwest::Client as HttpClient;
use serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::Result as SerenityResult;
use songbird::input::Compose;
use songbird::{input::YoutubeDl, tracks::TrackHandle};
use tracing::{error, info};

pub struct VoiceHttpKey;

impl TypeMapKey for VoiceHttpKey {
    type Value = HttpClient;
}

struct TrackInfo;
impl TypeMapKey for TrackInfo {
    type Value = (String, String);
}

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild_id, channel_id) = {
        let guild = msg.guild(&ctx.cache).unwrap();

        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);
        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);
            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }
        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;
            handler.remove_all_global_events();
            info!("removing handlers");
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let song = args.rest().to_owned();
    let is_url = song.starts_with("http");

    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if manager.get(guild_id).is_none() {
        let channel_id = {
            let guild = msg.guild(&ctx.cache).unwrap();
            guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id)
        };

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                check_msg(msg.reply(ctx, "Not in a voice channel").await);

                return Ok(());
            }
        };

        let _handler = manager.join(guild_id, connect_to).await;
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let _dch = ChannelDisconnect::new(manager.clone(), ctx.http.clone(), guild_id)
            .register_handler(&handler_lock)
            .await;
        let mut handler = handler_lock.lock().await;

        // TODO: make this faster with less cloning
        let http_client = get_http_client(ctx).await;
        let mut source = if is_url {
            YoutubeDl::new(http_client, song)
        } else {
            YoutubeDl::new_search(http_client, song)
        };

        let handle = handler.enqueue_input(source.clone().into()).await;
        if let Ok(metadata) = source.aux_metadata().await {
            let url = match metadata.source_url {
                Some(url) => url,
                None => "https://en.wikipedia.org/wiki/HTTP_404".to_string(),
            };
            let title = match metadata.title {
                Some(title) => title,
                None => "Title".to_string(),
            };
            handle
                .typemap()
                .write()
                .await
                .insert::<TrackInfo>((title, url));
        }
        let embed = song_embed(&mut source, handler.queue().len()).await;

        let builder = CreateMessage::new().embed(embed);
        check_msg(msg.channel_id.send_message(&ctx.http, builder).await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        match handler.queue().pause() {
            Ok(s) => s,
            Err(why) => {
                error!("Err pausing source {:?}", why);

                return Ok(());
            }
        };

        check_msg(msg.channel_id.say(&ctx.http, "Paused song").await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        match handler.queue().resume() {
            Ok(s) => s,
            Err(why) => {
                error!("Err resuming source {:?}", why);

                return Ok(());
            }
        };

        check_msg(msg.channel_id.say(&ctx.http, "resume song").await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        handler.queue().stop();

        check_msg(
            msg.channel_id
                .say(&ctx.http, "stopping song and clearing queue")
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        match handler.queue().skip() {
            Ok(s) => s,
            Err(why) => {
                error!("Err skip source {:?}", why);

                return Ok(());
            }
        };

        check_msg(msg.channel_id.say(&ctx.http, "skipped song").await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let footer = CreateEmbedFooter::new("rauly.rs");
    let mut embed = CreateEmbed::new()
        .colour(0xeb984e)
        .title("Music Queue")
        .footer(footer);

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        let list = handler.queue().current_queue();
        for (i, track) in list.iter().take(10).enumerate() {
            let metadata = get_metadata(track).await;
            info!("Position: {}, Url: {} Title: {}", i, metadata.1, metadata.0);
            if i == 0 {
                embed = embed.field(
                    "Now Playing:".to_string(),
                    format!("[{}]({})", metadata.0, metadata.1),
                    false,
                );
            } else {
                embed = embed.field(
                    String::new(),
                    format!("**{}**. [{}]({})", i, metadata.0, metadata.1),
                    false,
                );
            }
        }

        let builder = CreateMessage::new().embed(embed);
        check_msg(msg.channel_id.send_message(&ctx.http, builder).await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

async fn get_metadata(track_handle: &TrackHandle) -> (String, String) {
    let typemap = track_handle.typemap().read().await;
    typemap
        .get::<TrackInfo>()
        .cloned()
        .expect("This shouldn't be empty")
}

async fn get_http_client(ctx: &Context) -> HttpClient {
    let data = ctx.data.read().await;
    data.get::<VoiceHttpKey>()
        .cloned()
        .expect("Guaranteed to exist in the typemap")
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}

async fn song_embed(current_track: &mut impl Compose, postion: usize) -> CreateEmbed {
    let footer = CreateEmbedFooter::new("rauly.rs");
    let mut embed = CreateEmbed::new().colour(0xeb984e);
    if let Ok(metadata) = current_track.aux_metadata().await {
        if let Some(title) = metadata.title {
            embed = embed.title(format!("rauly.rs | {}", title));
        }
        if let Some(artist) = metadata.artist {
            embed = embed.field("Artist", artist.to_string(), true);
        }
        if let Some(track) = metadata.track {
            embed = embed.field("Track", track.to_string(), true);
        }
        if let Some(track_url) = metadata.source_url {
            embed = embed.url(track_url);
        }
        if let Some(track_len) = metadata.duration {
            let track_min = (track_len.as_secs() / 60) % 60;
            let track_secs = track_len.as_secs() % 60;
            embed = embed.field(
                "Song Duration",
                format!("{}:{:0>2}", track_min, track_secs),
                false,
            );
        }
        if let Some(thumbnail) = metadata.thumbnail {
            embed = embed.image(thumbnail);
        }
    }

    embed
        .field("Position in Queue", format!("{}", postion), false)
        .footer(footer)
}
