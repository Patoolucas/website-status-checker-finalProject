mod status;
mod pool;
mod fetch;

use status::WebsiteStatus;
use pool::ThreadPool;

use std::fs::File;
use std::io::{self, BufRead};
use std::sync::mpsc;
use std::env;

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: website_checker [--file sites.txt] [URL ...] \
               [--workers N] [--timeout S] [--retries N]");
    std::process::exit(2);
}

fn main() -> io::Result<()> {
    // ---------- CLI parsing ----------
    let mut args = env::args().skip(1);
    let mut urls: Vec<String> = Vec::new();
    let mut file_opt: Option<String> = None;

    // default worker count = logical CPUs (std 1.78+, no extra crate)
    let mut workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let mut timeout_secs = 5_u64;
    let mut retries = 0_u32;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file"    => file_opt = args.next(),
            "--workers" => workers = args.next()
                                         .and_then(|v| v.parse().ok())
                                         .unwrap_or_else(|| { print_usage_and_exit(); }),
            "--timeout" => timeout_secs = args.next()
                                         .and_then(|v| v.parse().ok())
                                         .unwrap_or_else(|| { print_usage_and_exit(); }),
            "--retries" => retries = args.next()
                                         .and_then(|v| v.parse().ok())
                                         .unwrap_or_else(|| { print_usage_and_exit(); }),
            _ if arg.starts_with("--") => print_usage_and_exit(),
            _ => urls.push(arg),
        }
    }

    // ---------- URLs from --file ----------
    if let Some(path) = file_opt {
        let file = File::open(path)?;
        for line in io::BufReader::new(file).lines() {
            let l = line?;
            let trimmed = l.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                urls.push(trimmed.to_string());
            }
        }
    }

    if urls.is_empty() {
        print_usage_and_exit();
    }

    // ---------- Concurrency setup ----------
    let (result_tx, result_rx) = mpsc::channel::<WebsiteStatus>();
    let pool = ThreadPool::new(workers);
    let fetcher = std::sync::Arc::new(fetch::Fetcher::new(timeout_secs, retries));

    // submit one job per URL
    for url in urls.clone() {
        let tx_clone = result_tx.clone();
        let fetch_clone = std::sync::Arc::clone(&fetcher);
        pool.submit(move || {
            let status = fetch_clone.fetch(url);
            println!("{}", status.to_line());       // live output
            tx_clone.send(status).unwrap();         // send to aggregator
        });
    }
    drop(result_tx); // close channel when all jobs queued

    // ---------- Collect & write JSON ----------
    let mut json = String::from("[");
    let mut first = true;
    for status in result_rx {
        if !first { json.push(','); } else { first = false; }
        json.push_str(&status.to_json());
    }
    json.push(']');
    std::fs::write("status.json", json)?;

    Ok(())
}

