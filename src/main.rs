use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use textplots::{Chart, Plot, Shape};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub tests: Vec<benchmark::Test>,
}

async fn ordered_get(buffer_size: usize, urls: Vec<String>) -> Vec<(StatusCode, String)> {
    let client = Client::new();

    stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client.get(url).send().await.expect("No connection to the server.");
                (resp.status(), resp.text().await.expect("No response from server."))
            }
        })
        .buffered(buffer_size)
        .collect::<Vec<_>>().await
}

async fn unordered_get(buffer_size: usize, urls: Vec<String>) -> Vec<(StatusCode, String)> {
    let client = Client::new();

    stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client.get(url).send().await.expect("No connection to the server.");
                (resp.status(), resp.text().await.expect("No response from server."))
            }
        })
        .buffer_unordered(buffer_size)
        .collect::<Vec<_>>().await
}

trait ErrorHandler<T> {
   fn graceful_exit(self, msg: &str) -> T;
}

impl<T> ErrorHandler<T> for Result<T,Box<dyn Error>> {
    fn graceful_exit(self, msg: &str) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => {
                println!("{}\n\nAdditional information:\n  Error: {}", msg, err);
                if let Some(source) = err.source() {
                    println!("    Caused by: {}", source);
                }
                std::process::exit(1);
            }
        }
    }
}

impl<T> ErrorHandler<T> for Option<T> {
    fn graceful_exit(self, msg: &str) -> T {
        match self {
            Some(some) => some,
            None => {
                println!("Error: {}", msg);
                std::process::exit(1);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let file = File::open("fixed_requests_number_250.json")
        .map_err(|e| e.into())
        .graceful_exit("The 'fixed_requests_number_250.json' does not exist. Please add it.");

    let config: Config = serde_json::from_reader(file)
        .map_err(|e| e.into())
        .graceful_exit("The configuration file is incorrect.");

   {
        let results = benchmark::average_time(&config.tests, ordered_get, "Ordered buffer - 250 requests").await;

        let points: Vec<(f32,f32)> = config.tests.iter()
            .map(|t| t.concurrent_requests as f32)
            .zip(results.iter()
                .map(|r| r.0))
            .collect();

        let max = config.tests.iter().map(|t| t.concurrent_requests).max().unwrap() as f32;
        let min = config.tests.iter().map(|t| t.concurrent_requests).min().unwrap() as f32;

        println!("\ny = time[s], x = concurrent requests");
        Chart::new(200, 120, min, max)
            .lineplot(&Shape::Lines(&points))
            .nice();
    }
    {
        let results = benchmark::average_time(&config.tests, unordered_get, "Unrdered buffer - 250 requests").await;

        let points: Vec<(f32,f32)> = config.tests.iter()
            .map(|t| t.concurrent_requests as f32)
            .zip(results.iter()
                .map(|r| r.0))
            .collect();

        let max = config.tests.iter().map(|t| t.concurrent_requests).max().unwrap() as f32;
        let min = config.tests.iter().map(|t| t.concurrent_requests).min().unwrap() as f32;

        println!("\ny = time[s], x = concurrent requests");
        Chart::new(200, 120, min, max)
            .lineplot(&Shape::Lines(&points))
            .nice();
    }

    let file = File::open("fixed_concurrent_requests_50.json")
        .map_err(|e| e.into())
        .graceful_exit("The 'fixed_concurrent_requests_50.json' does not exist. Please add it.");

    let config: Config = serde_json::from_reader(file)
        .map_err(|e| e.into())
        .graceful_exit("The configuration file is incorrect.");

    {
        let results = benchmark::average_time(&config.tests, ordered_get, "Ordered buffer - 50 concurrent requests").await;

        let points: Vec<(f32,f32)> = config.tests.iter()
            .map(|t| t.requests_number as f32)
            .zip(results.iter()
                .map(|r| r.0))
            .collect();

        let max = config.tests.iter().map(|t| t.requests_number).max().unwrap() as f32;
        let min = config.tests.iter().map(|t| t.requests_number).min().unwrap() as f32;

        println!("\ny = time[s], x = requests number");
        Chart::new(200, 120, min, max)
            .lineplot(&Shape::Lines(&points))
            .nice();
    }
    {
        let results = benchmark::average_time(&config.tests, unordered_get, "Unrdered buffer - 50 concurrent requests").await;

        let points: Vec<(f32,f32)> = config.tests.iter()
            .map(|t| t.requests_number as f32)
            .zip(results.iter()
                .map(|r| r.0))
            .collect();

        let max = config.tests.iter().map(|t| t.requests_number).max().unwrap() as f32;
        let min = config.tests.iter().map(|t| t.requests_number).min().unwrap() as f32;

        println!("\ny = time[s], x = requests number");
        Chart::new(200, 120, min, max)
            .lineplot(&Shape::Lines(&points))
            .nice();
    }
}
