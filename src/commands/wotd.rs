// Import necessary modules and crates
use crate::{models::word, utils}; // Importing word module from models and utils module
use chrono::prelude::*; // Importing date and time related functionalities
use rand::prelude::*; // Importing random functionalities
use serenity::framework::standard::{macros::command, CommandResult}; // Importing necessary items from serenity
use serenity::model::prelude::*; // Importing necessary items from serenity
use serenity::prelude::*; // Importing necessary items from serenity

// Command for retrieving and displaying a word of the day
#[command]
pub async fn word(ctx: &Context, msg: &Message) -> CommandResult {
    // Get the current date and time in a specific format
    let dt = Utc::now().format("%B %d, %Y");

    // Choose a random color from the COLORS array defined in the utils module
    let color = utils::COLORS.choose(&mut rand::thread_rng()).unwrap();

    // Fetch a word of the day using the word::word() function from the word module
    let res = match word::word().await {
        Ok(r) => r, // If fetching the word is successful, store it in 'res'
        Err(_) => {
            // If there's an error parsing JSON, send an error message and return
            msg.channel_id
                .say(&ctx.http, "had a problem parsing JSON!")
                .await?;
            return Ok(());
        }
    };

    // Create an example sentence with the word highlighted using markdown
    let example = res.examples[0]
        .text
        .replace(&res.word, &format!("**{}**", res.word));

    // Send an embedded message to the channel with details about the word of the day
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{} | {}", res.word, dt)) // Set the title of the embed
                    .colour(*color) // Set the color of the embed
                    // Add fields for word type, definition, example, and note
                    .field(
                        "Word type",
                        format!("*{}*", res.definitions[0].part_of_speech),
                        false,
                    )
                    .field("Definition", &res.definitions[0].text, false)
                    .field("Example", example, false)
                    .field("Note", res.note, false)
                    .footer(|f| {
                        f.text("Word of the Day"); // Set the footer text
                        f
                    })
            })
        })
        .await?; // Await sending the message and handle any errors

    Ok(()) // Return Ok to indicate successful command execution
}
