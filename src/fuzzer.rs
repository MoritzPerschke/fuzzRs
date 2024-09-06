use std::io::{self, BufRead};
use std::fs::File;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use reqwest::{Client, StatusCode};
use futures::stream::{self, StreamExt};
use tokio::runtime::Runtime;

// Request fuzzing work
async fn request_url(client: &Client, url: &str) -> Result<(String, StatusCode), reqwest::Error> {
    let res = client.get(url).send().await;
    let result = match res {
        Ok(res) => {
            Ok((url.to_string(), res.status()))
        }
        Err(err) => {
            Err(err)
        }
    };
    result
}

pub fn fuzz(wordlist: &str, host: &str, query_results: &Arc<Mutex<HashMap<String, StatusCode>>>) {

    let wordlist = File::open(wordlist);
    let wordlist = io::BufReader::new(wordlist.unwrap()).lines();

    // really no point in continuing execution if this doesn't work
    let rt = Runtime::new().expect("Failed to create Async runtime");

    rt.block_on(async {
        let client = Client::new();
        let _ = stream::iter(wordlist.into_iter().map(|word| {
            let client = client.clone();
            let query_results = query_results.clone();
            let url = format!("{}{}", &host, &word.unwrap());

            tokio::spawn(async move {
                // println!("Requesting...");
                let result = request_url(&client, &url).await;
                match result {
                    Ok(result) => {
                        // lock only panicks when current thread already holds mutex
                        let mut map = query_results.lock().unwrap(); 
                        map.insert(result.0, result.1);
                    }
                    _ => ()
                }
            })
        }))
        .buffer_unordered(100)
        .collect::<Vec<_>>()
        .await;
    });
    
}

