use std::fs::File;
use std::io::{self, Write};

use reqwest::Client;
use futures_util::StreamExt;
use tokio::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DogImage {
    fileSizeBytes: u64,
    url: String,
}


#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_url = "https://random.dog/woof.json";

    let api_response = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .get(api_url)
        .send().await?;

    let dog_img_json:DogImage = api_response.json().await?;

    let url: String = dog_img_json.url;

    let file_name = url.split('/').last().unwrap_or("downloaded_file");

    let cloned_url = url.clone();
    let client = Client::new();
    let res = client
        .get(&cloned_url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &cloned_url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &cloned_url))?;

    let mut file = File::create(file_name).or(Err(format!("Failed to create file '{}'", file_name)))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    let mut last_tick = Instant::now();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = downloaded + (chunk.len() as u64);
        downloaded = new;


        if last_tick.elapsed() >= Duration::from_secs(1) {
            print_file_size(downloaded, total_size);
            last_tick = Instant::now();
        }

        io::stdout().flush().unwrap(); 
    }

    println!();

    println!("Downloaded {} to {}", url, file_name);

    Ok(())
}

fn print_file_size(downloaded: u64, total_size: u64) {
    print!(
        "\rDownloading [{}/{}]",
        format_size(downloaded),
        format_size(total_size)
    );
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * KB;
    const GB: u64 = MB * KB;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}