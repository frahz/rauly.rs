use crate::voice::disconnect_handler::ChannelDisconnect;
use crate::{Context, Error};
use poise::ReplyHandle;
use reqwest::Client as HttpClient;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};
use serenity::prelude::TypeMapKey;
use songbird::{
    input::{Compose, YoutubeDl},
    tracks::TrackHandle,
    Songbird,
};
use std::sync::Arc;
use tracing::{debug, error, info};

pub struct VoiceHttpKey;

impl TypeMapKey for VoiceHttpKey {
    type Value = HttpClient;
}

struct TrackInfo;
impl TypeMapKey for TrackInfo {
    type Value = (String, String);
}

#[poise::command(
    slash_command,
    subcommands("join", "leave", "play", "pause", "resume", "stop", "skip", "info")
)]
pub async fn voice(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Make the bot join your current voice channel
#[poise::command(slash_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Err(why) = join_vc(ctx, manager).await {
        check_msg(ctx.reply(why).await);
    } else {
        check_msg(ctx.reply("Joined Voice Channel.").await);
    }

    Ok(())
}

/// Make the bot leave its current voice channel
#[poise::command(slash_command, guild_only)]
async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(ctx.say(format!("Failed: {:?}", e)).await);
        }
        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;
            handler.remove_all_global_events();
            info!("removing handlers");
        }

        check_msg(ctx.say("Left Voice Channel.").await);
    } else {
        check_msg(ctx.reply("Not in a voice channel.").await);
    }

    Ok(())
}

/// Play an audio track by providing a link or search query
#[poise::command(slash_command, guild_only)]
async fn play(ctx: Context<'_>, song: String) -> Result<(), Error> {
    ctx.defer().await?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let guild_id = ctx.guild_id().unwrap();

    if manager.get(guild_id).is_none() {
        if let Err(why) = join_vc(ctx, manager.clone()).await {
            check_msg(ctx.reply(why).await);
        }
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let _dch = ChannelDisconnect::new(manager.clone(), guild_id)
            .register_handler(&handler_lock)
            .await;
        let mut handler = handler_lock.lock().await;

        // TODO: make this faster with less cloning
        let http_client = get_http_client(&ctx).await;

        let is_url = song.starts_with("http");
        let mut source = if is_url {
            YoutubeDl::new(http_client, song)
        } else {
            YoutubeDl::new_search(http_client, song)
        };

        debug!("Source: {source:?}");
        let handle = handler.enqueue_input(source.clone().into()).await;
        if let Ok(metadata) = source.aux_metadata().await {
            debug!("metadata: {metadata:?}");
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

        let msg = poise::CreateReply::default().embed(embed);
        check_msg(ctx.send(msg).await);
    } else {
        check_msg(ctx.say("Not in a voice channel.").await);
    }

    Ok(())
}

/// Pauses the current audio track
#[poise::command(slash_command, guild_only)]
async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        if let Err(why) = handler.queue().pause() {
            error!("Err pausing source {:?}", why);
            return Ok(());
        }

        check_msg(ctx.say("Paused current audio track.").await);
    } else {
        check_msg(ctx.say("Not in a voice channel.").await);
    }

    Ok(())
}

/// Resumes the current audio track
#[poise::command(slash_command, guild_only)]
async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        if let Err(why) = handler.queue().resume() {
            error!("Err resuming source {:?}", why);
            return Ok(());
        }

        check_msg(ctx.say("Resumed the current audio track.").await);
    } else {
        check_msg(ctx.say("Not in a voice channel.").await);
    }

    Ok(())
}

/// Stops the song and clears the queue
#[poise::command(slash_command, guild_only)]
async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        handler.queue().stop();

        check_msg(ctx.say("stopping song and clearing queue").await);
    } else {
        check_msg(ctx.say("Not in a voice channel.").await);
    }

    Ok(())
}

/// Skips the current audio track
#[poise::command(slash_command, guild_only)]
async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        if let Err(why) = handler.queue().skip() {
            error!("Err skip source {:?}", why);
            return Ok(());
        }

        check_msg(ctx.say("Skipped audio track.").await);
    } else {
        check_msg(ctx.say("Not in a voice channel.").await);
    }

    Ok(())
}

/// Display a list of the current audio tracks
#[poise::command(slash_command, guild_only)]
async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
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

        // TODO: handle empty case
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

        let msg = poise::CreateReply::default().embed(embed);
        check_msg(ctx.send(msg).await);
    } else {
        check_msg(ctx.say("Not in a voice channel.").await);
    }

    Ok(())
}

async fn join_vc(ctx: Context<'_>, manager: Arc<Songbird>) -> Result<(), String> {
    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();

        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);
        (guild.id, channel_id)
    };
    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            return Err("User not in a voice channel.".to_string());
        }
    };

    if let Err(why) = manager.join(guild_id, connect_to).await {
        debug!("Failed to join vc: {}", why);
        return Err("Failed to join voice channel.".to_string());
    }

    return Ok(());
}

async fn get_metadata(track_handle: &TrackHandle) -> (String, String) {
    let typemap = track_handle.typemap().read().await;
    typemap
        .get::<TrackInfo>()
        .cloned()
        .expect("This shouldn't be empty")
}

async fn get_http_client(ctx: &Context<'_>) -> HttpClient {
    let data = ctx.serenity_context().data.read().await;
    data.get::<VoiceHttpKey>()
        .cloned()
        .expect("Guaranteed to exist in the typemap")
}

fn check_msg(result: Result<ReplyHandle, serenity::Error>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}

async fn song_embed(current_track: &mut impl Compose, postion: usize) -> CreateEmbed {
    let footer = CreateEmbedFooter::new("rauly.rs");
    let mut embed = CreateEmbed::new().colour(0xeb984e);
    if let Ok(metadata) = current_track.aux_metadata().await {
        info!("metadata: {metadata:?}");
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
