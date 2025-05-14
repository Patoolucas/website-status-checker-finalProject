use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct WebsiteStatus {
    pub url: String,
    // start the variables
    pub action_status: Result<u16, String>,
    pub response_time: Duration,
    pub timestamp: SystemTime,
}

impl WebsiteStatus {
    //render the string 
    pub fn to_line(&self) -> String {
        let status = match &self.action_status {
            Ok(code) => format!("HTTP {}", code),
            Err(e)   => format!("ERROR {e}"),
        };
        format!(
            "[{:?}] {} — {} — {} ms",
            self.timestamp,
            self.url,
            status,
            self.response_time.as_millis()
        )
    }

    // build a json to prepare the status
    pub fn to_json(&self) -> String {
        let status_str = match &self.action_status {
            Ok(code) => code.to_string(),
            Err(e)   => format!("\"{e}\""),
        };
        format!(
            r#"{{"url":"{}","status":{},"rt_ms":{},"ts":{}}}"#,
            self.url,
            status_str,
            self.response_time.as_millis(),
            self.timestamp.duration_since(SystemTime::UNIX_EPOCH)
                           .unwrap()
                           .as_secs()
        )
    }
}
