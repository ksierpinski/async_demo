use futures::Future;
use reqwest::{StatusCode};
use serde::{Deserialize, Serialize};
use std::time::{Instant, Duration};
use std::{thread};

/// Structure describe single test round.
#[derive(Serialize, Deserialize)]
pub struct Test {
    /// Test description
    pub label: String,
    /// HTTP request GET method to be tested
    pub url_get: String,
    /// Multipler of URL GET in single test
    pub requests_number: usize,
    /// Define pool size of requests to be tested
    pub concurrent_requests: usize,
    /// Repeats of this test instance
    pub repeats: usize,
    /// Delay between repeats, defined in seconds
    pub delay_s: u64,
}

impl std::fmt::Display for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}\nurl_get: {}\nrequests_number: {}\nconcurrent_requests: {}\nrepeats: {}\ndelay: {}s",
            self.label, self.url_get, self.requests_number, self.concurrent_requests, self.repeats, self.delay_s)
    }
}

/// A function to proceed series of test.
/// Returns the vector of tuples represents the (mean, standard deviation) execution time, definded in seconds
pub async fn average_time<F, Fut>(tests: &Vec<Test>, async_func: F, title: &str) -> Vec<(f32,f32)>
where
    F: Fn(usize,Vec<String>) -> Fut,
    Fut: Future<Output = Vec<(StatusCode, String)>>
{
    println!("\n🌊🌊🌊 {} 🌊🌊🌊", title);
    let mut ret = Vec::new();
    for (test_idx, test) in tests.iter().enumerate() {
        println!("\n🚀Test{} - {}", test_idx+1, test);

        let mut results = Vec::new();
        for idx in 1..=test.repeats {
            if idx != 1 && test_idx != 0 {
                thread::sleep(Duration::from_secs(test.delay_s));
            }

            let urls: Vec<String> = vec![test.url_get.clone(); test.requests_number];

            let now = Instant::now();
            let resp = async_func(test.concurrent_requests, urls).await;
            let time = now.elapsed().as_secs_f32();

            resp.iter()
                .for_each(|r|
                    if !r.0.is_success() {
                        let err = format!("{}", r.0);
                        println!("Error: {}", err);
                        std::process::exit(1);
                    }
                );

            println!("  [{}/{}] time: {}s", idx, test.repeats, time);
            results.push(time);
        }

        let (mean, std_dev) = statistic(&results)
            .expect("Repeats equal zero");

        println!("SUMMARY: {}±{}s", mean, std_dev);

        ret.push((mean, std_dev));
    }

    ret
}

fn statistic(data: &[f32]) -> Option<(f32,f32)> {
    if data.len() == 0 {
        return None;
    }

    let count = data.len() as f32;
    let mean = data.iter().sum::<f32>() / count;

    let variance = data.iter()
        .map(|value| {
            let diff = mean - *value;
            diff * diff
        })
        .sum::<f32>() / count;

    let std_deviation = variance.sqrt();

    Some((mean, std_deviation))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stats_basic() {
        let data = [10.0, 12.0, 23.0, 23.0, 16.0, 23.0, 21.0, 16.0];
        let (mean, std_dev) = statistic(&data).unwrap();

        assert_eq!(18.0, mean);
        assert_eq!(4.8989797, std_dev);
    }

    #[test]
    fn stats_none() {
        let data = [];
        assert!(statistic(&data).is_none());
    }
}

