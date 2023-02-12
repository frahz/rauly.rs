use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentProvider {
    name: String,
    id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    source: String,
    pub text: String,
    note: Option<String>,
    pub part_of_speech: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Example {
    url: String,
    title: String,
    pub text: String,
    id: Option<u32>,
}

pub async fn word() -> Result<Response> {
    info!("Sending Word of the Day");
    let wordnik_api = env::var("WORDNIK_API_KEY").expect("wordnik api key");
    let url = format!(
        "https://api.wordnik.com/v4/words.json/wordOfTheDay?api_key={}",
        wordnik_api
    );

    match reqwest::get(url).await?.json::<Response>().await {
        Ok(r) => Ok(r),
        Err(e) => {
            error!("Problem parsing JSON: {:?}", e);
            Err(e.into())
        }
    }

}
