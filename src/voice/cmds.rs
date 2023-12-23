use crate::voice::disconnect_handler::ChannelDisconnect; // Importing a module for handling voice channel disconnections
use serenity::builder::CreateEmbed; // Importing a builder for creating rich embeds in Discord
use serenity::framework::standard::{macros::command, Args, CommandResult}; // Importing Serenity's command-related items
use serenity::model::prelude::*; // Importing Serenity's model functionalities
use serenity::prelude::*; // Importing Serenity's prelude
use serenity::Result as SerenityResult; // Renaming Serenity's Result type for disambiguation
use songbird::{input::Restartable, tracks::TrackHandle}; // Importing Songbird's audio-related items
use tracing::{error, info}; // Importing logging functionalities

// Command for joining a voice channel
#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    // Retrieve guild information
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    // Retrieve the ID of the channel the author is in
    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    // Check if the author is in a voice channel; if not, reply and return
    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);
            return Ok(());
        }
    };

    // Retrieve the Songbird manager
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // Join the voice channel
    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

// Command for leaving a voice channel
#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    // Retrieve guild information
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    // Retrieve the Songbird manager
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // Check if a handler exists for the guild
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        // Attempt to remove the handler
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }
        // If a handler exists, remove global events and log
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

// Command for playing audio in a voice channel
#[command]
#[only_in(guilds)]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Retrieve the song from the command arguments
    let song = args.rest().to_owned();
    let is_url = song.starts_with("http");

    // Retrieve guild information
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    // Retrieve the Songbird manager
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // Check if a handler exists for the guild; if not, join the author's voice channel
    let has_handler = manager.get(guild_id).is_some();
    if !has_handler {
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                check_msg(msg.reply(ctx, "Not in a voice channel").await);
                return Ok(());
            }
        };

        let _handler = manager.join(guild_id, connect_to).await;
    }

    // If a handler exists, continue with playing the song
    if let Some(handler_lock) = manager.get(guild_id) {
        // Create a ChannelDisconnect to handle voice channel disconnections
        let _dch = ChannelDisconnect::new(manager.clone(), ctx.http.clone(), guild_id)
            .register_handler(&handler_lock)
            .await;
        let mut handler = handler_lock.lock().await;

        // Determine the source of the song (URL or search query)
        let resolved_source = match is_url {
            true => Restartable::ytdl(song, true).await,
            false => Restartable::ytdl_search(song, true).await,
        };

        // If the source is resolved, enqueue the song
        let source = match resolved_source {
            Ok(source) => source,
            Err(why) => {
                error!("Err starting source: {:?}", why);
                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);
                return Ok(());
            }
        };

        // Enqueue the song and create an embed with song details
        let current = handler.enqueue_source(source.into());
        let embed = song_embed(current, handler.queue().len());
        check_msg(
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.0 = embed.0;
                        e
                    })
                })
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

// Command for pausing audio playback in a voice channel
#[command]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    // Retrieve guild information
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    // Retrieve the Songbird manager
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // If a handler exists for the guild, pause the audio playback
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

// Command for resuming audio playback in a voice channel
#[command]
#[only_in(guilds)]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    // Retrieve guild information
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    // Retrieve the Songbird manager
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // If a handler exists for the guild, resume the audio playback
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

// Command for stopping audio playback and clearing the queue in a voice channel
#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    // Retrieve guild information
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    // Retrieve the Songbird manager
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // If a handler exists for the guild, stop the audio playback and clear the queue
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

// Command for skipping to the next track in a voice channel
#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    // Retrieve guild information
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    // Retrieve the Songbird manager
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // If a handler exists for the guild, skip to the next track
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

// Command for displaying information about the current music queue in a voice channel
#[command]
#[only_in(guilds)]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    // Retrieve guild information
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    // Retrieve the Songbird manager
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // If a handler exists for the guild, display information about the music queue
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        let list = handler.queue().current_queue();
        for track in list {
            let metadata = track.metadata();
            info!(
                "Artist: {} Track: {}",
                metadata.artist.clone().unwrap_or("<Unknown>".to_string()),
                metadata.track.clone().unwrap_or("<Unknown>".to_string())
            );
        }

        check_msg(msg.channel_id.say(&ctx.http, "printing queue info").await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

// Function for handling SerenityResult<Message> and logging errors if any
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}

// Function for creating an embedded message with song details
fn song_embed(current_track: TrackHandle, position: usize) -> CreateEmbed {
    // Retrieve metadata about the current song
    let dummy = "<Unknown>".to_string();
    let dummy_duration = std::time::Duration::new(60, 0);

    let artist = current_track
        .metadata()
        .artist
        .as_ref()
        .unwrap_or_else(|| &dummy);
    let track = current_track
        .metadata()
        .track
        .as_ref()
        .unwrap_or_else(|| &dummy);
    let track_url = current_track
        .metadata()
        .source_url
        .as_ref()
        .unwrap_or_else(|| &dummy);
    let track_len = current_track
        .metadata()
        .duration
        .as_ref()
        .unwrap_or_else(|| &dummy_duration);
    let track_min = (track_len.as_secs() / 60) % 60;
    let track_secs = track_len.as_secs() % 60;
    let thumbnail = current_track
        .metadata()
        .thumbnail
        .as_ref()
        .unwrap_or_else(|| &dummy);

    // Create an embedded message with song details
    let embed = CreateEmbed::default()
        .title("rauly.rs | music")
        .colour(0xeb984e)
        .url(track_url)
        .field("Artist", format!("{}", artist), true)
        .field("Track", format!("{}", track), true)
        .field(
            "Song Duration",
            format!("{}:{:0>2}", track_min, track_secs),
            false,
        )
        .field("Position in Queue", format!("{}", position), false)
        .image(thumbnail)
        .footer(|f| {
            f.text("rauly.rs");
            f
        })
        .to_owned();
    embed
}
