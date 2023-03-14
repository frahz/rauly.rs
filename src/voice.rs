use serenity::builder::CreateEmbed;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::Result as SerenityResult;
use songbird::{input::Restartable, tracks::TrackHandle};
use tracing::{error, info};

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

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
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

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

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Must provide a URL to a video or audio")
                    .await,
            );

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Must provide a valid URL")
                .await,
        );

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

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

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match Restartable::ytdl(url, true).await {
            Ok(source) => source,
            Err(why) => {
                error!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                return Ok(());
            }
        };

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

#[command]
#[only_in(guilds)]
#[aliases("se")]
async fn search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let song = args.rest();

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

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

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match Restartable::ytdl_search(song, true).await {
            Ok(source) => source,
            Err(why) => {
                error!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                return Ok(());
            }
        };

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

#[command]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

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
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

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
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

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
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

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
//TODO: Send message with queue info
#[command]
#[only_in(guilds)]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        let list = handler.queue().current_queue();
        for track in list {
            let metadata = track.metadata();
            info!(
                "Artist: {} Track: {}",
                metadata.artist.as_ref().unwrap(),
                metadata.track.as_ref().unwrap()
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

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}

fn song_embed(current_track: TrackHandle, postion: usize) -> CreateEmbed {
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
        .field("Position in Queue", format!("{}", postion), false)
        .image(thumbnail)
        .footer(|f| {
            f.text("rauly.rs");
            f
        })
        .to_owned();
    embed
}
