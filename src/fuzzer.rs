use std::fs::{self};
use std::sync::{Arc, Mutex};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, StatusCode, Version};
use tokio::runtime::Runtime;

use crate::gui::AppState;

// TODO: make configurable
const BATCH_SIZE: usize = 10;
#[derive(Debug, Default, Clone)]
pub struct Data {
    pub status: StatusCode,
    pub version: Version,
    pub headers: HeaderMap,
    pub content_length: u64,
    pub url: String,
    pub text: String,
}

/* do the one request */
async fn request_url(client: &Client, url: &str, _method: Method) -> Result<Data, reqwest::Error> {
    match client.get(url).send().await {
        Ok(response) => {
            let mut ret = Data {
                status: response.status(),
                version: response.version(),
                headers: response.headers().clone(),
                content_length: u64::default(),
                url: response.url().to_string(),
                text: String::default(),
            };
            match &response.content_length() {
                Some(len) => ret.content_length = *len,
                _ => {}
            }
            match &response.text().await {
                Ok(text) => ret.text = text.to_string(),
                _ => {}
            }
            Ok(ret)
        }
        Err(err) => Err(err)
    }
}

async fn process_batch(client: &Client, target: &String, words: Vec<String>) -> Vec<Data>{
    let mut handles = vec![];
    let results: Arc<Mutex<Vec<Data>>> = Arc::default();

    for word in words {
        let client = client.clone();
        let url = target.replace("FUZZ", &word);
        let results = results.clone();

        // spawn a tokio task for every word in the batch for concurrent processing
        let handle = tokio::spawn(async move {
            let result = request_url(&client, &url, Method::GET).await;
            if result.is_ok(){
                let data = result.unwrap();
                let mut results = results.lock().unwrap();
                results.push(data);
            }
        });
        handles.push(handle);
    }
    // wait for all tasks to finish
    futures::future::join_all(handles).await;

    let results = results.lock().unwrap();
    results.to_vec()
}

pub fn fuzz(gui_params: &mut AppState) {

    // read entire wordlist to memory, this might not be smart
    let wordlist = fs::read_to_string(&gui_params.wordlist).expect("Failed to find wordlist");
    let wordlist: Vec<String> = wordlist.lines().map(|line| line.to_string()).collect();
    let batches: Vec<Vec<String>> = wordlist
        .chunks(BATCH_SIZE)
        .map(|chunk| chunk.to_vec())
        .collect();

    // really no point in continuing execution if this doesn't work
    let rt = Runtime::new().expect("Failed to create Async runtime");
    rt.block_on(async {
        let client = Client::new();

        let test_str: String = rand::thread_rng().sample_iter(&Alphanumeric).take(15).map(char::from).collect();
        let test_url = format!("{}{}", &gui_params.target, &test_str);
        let _ = match client.get(&test_url).send().await {
            Err(err) => {
                Err(err)
            }
            _ => Ok(())
        };
        for batch in batches{
            let batch_results: Vec<Data> = process_batch(&client, &gui_params.target, batch).await;
            gui_params.query_results.extend(batch_results);
        };
    });
}

