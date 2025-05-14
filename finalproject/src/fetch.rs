use reqwest::blocking::Client;
use std::time::{Duration, Instant};
use crate::status::WebsiteStatus;

pub struct Fetcher {
    client: Client,
    timeout: Duration,
    retries: u32,
}

impl Fetcher {
    pub fn new(timeout_secs: u64, retries: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("client build failed");
        Self { client, timeout: Duration::from_secs(timeout_secs), retries }
    }

    pub fn fetch(&self, url: String) -> WebsiteStatus {
        let mut attempts_left = self.retries + 1;

        loop {
            let start = Instant::now();
            let result = self.client.get(&url).send();
            let elapsed = start.elapsed();
            let ts = std::time::SystemTime::now();

            match result {
                Ok(resp) => {
                    let code = resp.status().as_u16();
                    return WebsiteStatus {
                        url,
                        action_status: Ok(code),
                        response_time: elapsed,
                        timestamp: ts,
                    };
                }
                Err(_e) if attempts_left > 1 => {
                    // ignore error, sleep, and retry
                    attempts_left -= 1;
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    return WebsiteStatus {
                        url,
                        action_status: Err(e.to_string()),
                        response_time: elapsed,
                        timestamp: ts,
                    };
                }
            }
        }
    }
}
