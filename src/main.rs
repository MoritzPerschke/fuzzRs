use std::env;
use std::fs::File;
use std::io::{self, BufRead};

use tokio::task;
use reqwest::{Client, StatusCode};
use futures::stream::{self, StreamExt};

async fn fuzz_url(client: &Client, url: &str) -> Result<(String, StatusCode), reqwest::Error> {
    let res = client.get(url).send().await;
    let result = match res {
        Ok(res) => {
            if res.status() == StatusCode::OK{
                println!("Status: {} returned by {}", res.status(), &url);
            }
            Ok((url.to_string(), res.status()))
        }
        Err(err) => {
            Err(err)
        }
    };

    result
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let wordlist = &args[1];
    let wordlist = File::open(wordlist);
    let wordlist = io::BufReader::new(wordlist.unwrap()).lines();

    let host = &args[2];

    let client = Client::new();

    let tasks = stream::iter(wordlist.into_iter().map(|word| {
        let client = client.clone();
        let url = format!("{}{}", &host, &word.unwrap());

        task::spawn(async move {
            fuzz_url(&client, &url).await;
        })
    }))
    .buffer_unordered(500);

    tasks.collect::<Vec<_>>().await;
    // let responses: Vec<Result<Result<(StatusCode, String), reqwest::Error>, tokio::task::JoinError>> = tasks.collect::<Vec<_>>().await;
    // let resposes: Vec<Result<(String, StatusCode), _>> = tasks.collect()::<Vec<_>>().await;

    println!("Done...");

}
