use anyhow::Result; // Import the Result type from the `anyhow` crate
use serde::{Deserialize, Serialize}; // Import traits for serialization and deserialization
use std::env; // Import the standard library's environment module for environment variables
use tracing::{error, info}; // Import logging functionalities

// Structs representing the deserialized response from the Wordnik API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(rename = "_id")]
    id: String,
    pub word: String,
    content_provider: ContentProvider,
    pub definitions: Vec<Definition>,
    publish_date: String,
    pub examples: Vec<Example>,
    pdd: String,
    html_extra: Option<String>,
    pub note: String,
}

// Struct representing content provider information
#[derive(Debug, Serialize, Deserialize)]
pub struct ContentProvider {
    name: String,
    id: u32,
}

// Struct representing the definition of a word
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    source: String,
    pub text: String,
    note: Option<String>,
    pub part_of_speech: String,
}

// Struct representing an example of the word's usage
#[derive(Debug, Serialize, Deserialize)]
pub struct Example {
    url: String,
    title: String,
    pub text: String,
    id: Option<u32>,
}

// Function to fetch the Word of the Day from the Wordnik API
pub async fn word() -> Result<Response> {
    info!("Sending Word of the Day");

    // Fetch the Wordnik API key from an environment variable
    let wordnik_api = env::var("WORDNIK_API_KEY").expect("Wordnik API key not found");

    // Construct the URL for the Word of the Day endpoint
    let url = format!(
        "https://api.wordnik.com/v4/words.json/wordOfTheDay?api_key={}",
        wordnik_api
    );

    // Make a GET request to the Wordnik API and deserialize the JSON response
    match reqwest::get(url).await?.json::<Response>().await {
        Ok(r) => Ok(r), // If successful, return the deserialized response
        Err(e) => {
            error!("Problem parsing JSON: {:?}", e);
            Err(e.into()) // If there's an error parsing JSON, log it and return an error
        }
    }
}
